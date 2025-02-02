use std::{path::Path, sync::Arc};

use indexmap::IndexMap;
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_till, take_until, take_while, take_while1},
    character::complete::{char, digit1},
    combinator::{map, map_res, opt},
    error::ErrorKind,
    multi::{many0, many1},
    sequence::{delimited, pair, preceded, tuple},
    FindSubstring, IResult, InputLength, Parser, Slice,
};
use regex::Regex;

use super::{
    super::{Instance, KeyValue, LocalAST, Model, Subckt, Token, Unknwon},
    ast,
    manager::ParseManager,
};
use crate::{
    file::{EndReason, FileId, LocatedSpan, Pos, Span},
    lexer::{Data, DataFile, DataFiles, DataValues, ParseErrorInner, PnameColNum, Value, AST},
};

#[inline]
pub(super) fn space(i: LocatedSpan) -> IResult<LocatedSpan, LocatedSpan> {
    take_while(|c: char| matches!(c, '\t' | '\r' | ' '))(i)
}
#[inline]
pub(super) fn space1(i: LocatedSpan) -> IResult<LocatedSpan, LocatedSpan> {
    take_while1(|c: char| matches!(c, '\t' | '\r' | ' '))(i)
}
#[inline]
pub(super) fn space_newline(i: LocatedSpan) -> IResult<LocatedSpan, LocatedSpan> {
    take_while(|c: char| matches!(c, '\t' | '\r' | ' ' | '\n'))(i)
}

#[inline]
pub(super) fn multiline_sep<'a, T>(
    f: fn(LocatedSpan<'a>) -> IResult<LocatedSpan<'a>, T>,
) -> impl FnMut(LocatedSpan<'a>) -> IResult<LocatedSpan<'a>, T> {
    alt((
        preceded(space1, f),
        map(
            tuple((comment_space_newline, char('+'), space, f)),
            |(_, _, _, t)| t,
        ),
    ))
}

#[inline]
pub(super) fn loss_sep(i: LocatedSpan) -> IResult<LocatedSpan, LocatedSpan> {
    alt((
        map(
            tuple((comment_space_newline, char('+'), space)),
            |(_, _, s)| s,
        ),
        space,
    ))(i)
}

#[inline]
pub(super) fn key(i: LocatedSpan) -> IResult<LocatedSpan, Span> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        |s: LocatedSpan| s.into(),
    )(i)
}

enum Unit {
    Factor(f64),
    DB,
}

// hspice/index.htm#page/hspice_5/about_hspice_measurement_system.htm#wwID0EOGJM
#[inline]
fn unit(i: LocatedSpan) -> IResult<LocatedSpan, Unit> {
    map_res(take_while1(char::is_alphanumeric), |s: LocatedSpan| match s
        .fragment()
        .to_uppercase()
        .as_str()
    {
        "T" => Ok(Unit::Factor(1e+12)),
        "G" => Ok(Unit::Factor(1e+9)),
        "ME" | "MEG" | "X" | "Z" => Ok(Unit::Factor(1e+6)),
        "K" => Ok(Unit::Factor(1e+3)),
        "MI" | "MIL" => Ok(Unit::Factor(25.4e-6)),
        "U" => Ok(Unit::Factor(1e-6)),
        "N" => Ok(Unit::Factor(1e-9)),
        "P" => Ok(Unit::Factor(1e-12)),
        "F" => Ok(Unit::Factor(1e-15)),
        "A" => Ok(Unit::Factor(1e-18)),
        "DB" => Ok(Unit::DB),
        "MIN" => Ok(Unit::Factor(60.0)),
        "HR" => Ok(Unit::Factor(3600.0)),
        "DAY" => Ok(Unit::Factor(86400.0)),
        "YR" => Ok(Unit::Factor(31536000.0)),
        _ => Err(nom::Err::Error(nom::error::Error::new(i, ErrorKind::Float))),
    })(i)
}

#[inline]
pub(super) fn _float(i: LocatedSpan) -> IResult<LocatedSpan, f64> {
    match fast_float2::parse_partial(i) {
        Ok((f, pos)) => Ok((i.slice(pos..), f)),
        Err(_) => Err(nom::Err::Error(nom::error::Error::new(i, ErrorKind::Float))),
    }
}

#[inline]
pub(super) fn float_unit(i: LocatedSpan) -> IResult<LocatedSpan, f64> {
    map(pair(_float, opt(unit)), |(f, u)| match u {
        Some(Unit::Factor(u)) => f * u,
        Some(Unit::DB) => 10.0_f64.powf(f / 20.0),
        None => f,
    })(i)
}

#[inline]
pub(super) fn key_str(i: LocatedSpan) -> IResult<LocatedSpan, (&str, Span)> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        |s: LocatedSpan| {
            let _s: &str = s.fragment();
            (_s, s.into())
        },
    )(i)
}

#[inline]
pub(super) fn path(i: LocatedSpan) -> IResult<LocatedSpan, Span> {
    alt((
        unquote,
        map(
            take_while1(|c: char| c.is_alphanumeric() || !c.is_whitespace()),
            |s: LocatedSpan| s.into(),
        ),
    ))(i)
}

#[inline]
pub(super) fn integer(i: LocatedSpan) -> IResult<LocatedSpan, usize> {
    map_res(digit1, |s: LocatedSpan| s.parse())(i)
}

#[inline]
pub(super) fn path_str(i: LocatedSpan) -> IResult<LocatedSpan, &str> {
    alt((
        unquote_str,
        map(
            take_while1(|c: char| c.is_alphanumeric() || !c.is_whitespace()),
            |s: LocatedSpan| {
                let _s: &str = s.fragment();
                _s
            },
        ),
    ))(i)
}

#[inline]
pub(super) fn hierarchical_node_char(i: LocatedSpan) -> IResult<LocatedSpan, (u8, Span)> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '.'),
        |s: LocatedSpan| (s.fragment().as_bytes()[0], s.into()),
    )(i)
}

#[inline]
pub(super) fn hierarchical_node(i: LocatedSpan) -> IResult<LocatedSpan, Span> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '.'),
        |s: LocatedSpan| s.into(),
    )(i)
}

#[inline]
pub(super) fn formula(i: LocatedSpan) -> IResult<LocatedSpan, Span> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || "/_.+-*^:".contains(c)),
        |s: LocatedSpan| s.into(),
    )(i)
}

#[inline]
pub(super) fn unquote(i: LocatedSpan) -> IResult<LocatedSpan, Span> {
    map(
        delimited(char('\''), take_till(|c| c == '\''), take(1_usize)),
        |s: LocatedSpan| s.into(),
    )(i)
}

#[inline]
pub(super) fn unquote_str(i: LocatedSpan) -> IResult<LocatedSpan, &str> {
    map(
        delimited(char('\''), take_till(|c| c == '\''), take(1_usize)),
        |s: LocatedSpan| {
            let _s: &str = s.fragment();
            _s
        },
    )(i)
}

#[inline]
pub(super) fn comment(i: LocatedSpan) -> IResult<LocatedSpan, LocatedSpan> {
    delimited(tag("*"), take_till(|c| c == '\n'), take(1_usize))(i)
}

#[inline]
pub(super) fn comment_space_newline(i: LocatedSpan) -> IResult<LocatedSpan, LocatedSpan> {
    preceded(many0(pair(space_newline, comment)), space_newline)(i)
}

#[inline]
pub(super) fn value(i: LocatedSpan) -> IResult<LocatedSpan, Value> {
    match unquote(i) {
        Ok((i, s)) => return Ok((i, Value::Expr(s))),
        Err(_) => {}
    }
    match (float_unit(i), formula(i)) {
        (Ok((i_num, num)), Ok((i_formula, formula))) => {
            // when the len of rest of formula is less than num
            // means there is a non-quote expression, like `2+2`
            if i_formula.len() < i_num.len() {
                Ok((i_formula, Value::Expr(formula)))
            } else {
                Ok((i_num, Value::Num(num)))
            }
        }
        (Ok((i_num, num)), Err(_)) => Ok((i_num, Value::Num(num))),
        (Err(_), Ok((i_formula, formula))) => Ok((i_formula, Value::Expr(formula))),
        (Err(e), Err(_)) => Err(e),
    }
}

#[inline]
pub(super) fn equal<'a, T>(
    f: fn(LocatedSpan<'a>) -> IResult<LocatedSpan<'a>, T>,
) -> impl FnMut(LocatedSpan<'a>) -> IResult<LocatedSpan<'a>, T> {
    map(tuple((space, char('='), space, f)), |(_, _, _, v)| v)
}

#[inline]
pub(super) fn key_value(i: LocatedSpan) -> IResult<LocatedSpan, KeyValue> {
    map(pair(key, equal(value)), |(k, v)| KeyValue { k, v })(i)
}

#[inline]
pub(super) fn token(i: LocatedSpan) -> IResult<LocatedSpan, Token> {
    alt((map(key_value, Token::KV), map(value, Token::Value)))(i)
}

#[cfg(test)]
pub(super) fn span(s: &str) -> LocatedSpan {
    LocatedSpan::new(s)
}

pub fn many0_dummyfirst<I, O, E, F>(mut f: F) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + InputLength,
    F: Parser<I, O, E>,
    O: Default,
    E: nom::error::ParseError<I>,
{
    move |mut i: I| {
        let mut acc = Vec::with_capacity(4);
        acc.push(O::default());
        loop {
            let len = i.input_len();
            match f.parse(i.clone()) {
                Err(nom::Err::Error(_)) => return Ok((i, acc)),
                Err(e) => return Err(e),
                Ok((i1, o)) => {
                    // infinite loop check: the parser must always consume
                    if i1.input_len() == len {
                        return Err(nom::Err::Error(E::from_error_kind(i, ErrorKind::Many0)));
                    }

                    i = i1;
                    acc.push(o);
                }
            }
        }
    }
}

#[inline]
pub(super) fn ports_params(i: LocatedSpan) -> IResult<LocatedSpan, (Vec<Span>, Vec<KeyValue>)> {
    map(
        pair(
            many1(multiline_sep(hierarchical_node)),
            opt(pair(
                equal(value),
                many0_dummyfirst(multiline_sep(key_value)),
            )),
        ),
        |(mut ports, _params)| match _params {
            Some((first_value, mut params)) => {
                let first_key = ports.pop().unwrap();
                params[0] = KeyValue {
                    k: first_key,
                    v: first_value,
                };
                (ports, params)
            }
            None => (ports, Vec::new()),
        },
    )(i)
}

#[inline]
pub(super) fn instance(i: LocatedSpan) -> IResult<LocatedSpan, Instance> {
    map(
        tuple((hierarchical_node_char, ports_params)),
        |((first_char, name), (ports, params))| Instance {
            name,
            instance_type: first_char.into(),
            ports,
            params,
        },
    )(i)
}

/// ``` spice
/// .MODEL mname ModelType ([level=val]
/// + [keyword1=val1][keyword2=val2]
/// + [keyword3=val3][LOT distribution value]
/// + [DEV distribution value]...)
/// ```
#[inline]
pub(super) fn model(i: LocatedSpan) -> IResult<LocatedSpan, Model> {
    map(
        tuple((
            multiline_sep(key),
            multiline_sep(key_str),
            alt((
                many1(multiline_sep(key_value)),
                map(
                    tuple((
                        loss_sep,
                        char('('),
                        loss_sep,
                        opt(pair(key_value, many0_dummyfirst(multiline_sep(key_value)))),
                        loss_sep,
                        char(')'),
                    )),
                    |(_, _, _, v, _, _)| {
                        if let Some((first, mut vec)) = v {
                            vec[0] = first;
                            vec
                        } else {
                            Vec::new()
                        }
                    },
                ),
            )),
        )),
        |(name, model_type_ctx, params)| Model {
            name,
            model_type: model_type_ctx.into(),
            params,
        },
    )(i)
}

#[derive(Debug, Clone, Copy)]
struct EndLib;
impl FindSubstring<EndLib> for LocatedSpan<'_> {
    #[inline]
    fn find_substring(&self, _: EndLib) -> Option<usize> {
        use std::sync::LazyLock;
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new("(?i)\\.endl").unwrap());
        RE.find(self.fragment()).map(|m| m.start())
    }
}
impl InputLength for EndLib {
    #[inline]
    fn input_len(&self) -> usize {
        5
    }
}

#[inline]
pub(super) fn lib(i: LocatedSpan) -> IResult<LocatedSpan, Option<(&str, String)>> {
    alt((
        // include lib section
        map(
            tuple((path_str, space1, key_str)),
            |(path_str, _, (section_str, _))| Some((path_str, section_str.to_lowercase())),
        ),
        // skip to `.endl`
        map(
            tuple((take_until(EndLib), take(5usize), opt(pair(space1, key)))),
            |_| None,
        ),
    ))(i)
}
#[inline]
pub(super) fn data(mut i: LocatedSpan) -> IResult<LocatedSpan, Data> {
    #[inline]
    fn enddata(i: LocatedSpan) -> IResult<LocatedSpan, ()> {
        map_res(pair(char('.'), key_str), |(_, (key, _))| {
            if key.to_uppercase().as_str() == "ENDDATA" {
                Ok(())
            } else {
                // TODO: error information?
                Err(())
            }
        })(i)
    }
    #[inline]
    fn data_files(i: LocatedSpan) -> IResult<LocatedSpan, DataFiles> {
        #[inline]
        fn file(i: LocatedSpan) -> IResult<LocatedSpan, Span> {
            map_res(
                tuple((multiline_sep(key_str), equal(path))),
                |((key, _), path)| {
                    if key.to_uppercase().as_str() == "FILE" {
                        Ok(path)
                    } else {
                        Err(())
                    }
                },
            )(i)
        }
        #[inline]
        fn out(i: LocatedSpan) -> IResult<LocatedSpan, Span> {
            map_res(
                tuple((multiline_sep(key_str), equal(path))),
                |((key, _), path)| {
                    if key.to_uppercase().as_str() == "OUT" {
                        Ok(path)
                    } else {
                        Err(())
                    }
                },
            )(i)
        }
        #[inline]
        fn pname_col_num(i: LocatedSpan) -> IResult<LocatedSpan, PnameColNum> {
            map_res(
                tuple((multiline_sep(key_str), equal(integer))),
                |((pname_str, pname), col_num)| {
                    let binding = pname_str.to_uppercase();
                    let s = binding.as_str();
                    if s != "FILE" && s != "OUT" {
                        Ok(PnameColNum { pname, col_num })
                    } else {
                        Err(())
                    }
                },
            )(i)
        }
        map(
            tuple((
                many1(map(
                    pair(file, many1(pname_col_num)),
                    |(file, pname_col_num)| DataFile {
                        file,
                        pname_col_num,
                    },
                )),
                opt(out),
                space_newline,
                enddata,
            )),
            |(files, out, _, _)| DataFiles { files, out },
        )(i)
    }
    let name;
    (i, name) = multiline_sep(key)(i)?;
    let first;
    let first_str;
    (i, (first_str, first)) = multiline_sep(key_str)(i)?;
    match first_str.to_uppercase().as_str() {
        "MER" => {
            return data_files(i).and_then(|(i, data_files)| {
                Ok((
                    i,
                    Data {
                        name,
                        values: DataValues::MER(data_files),
                    },
                ))
            })
        }
        "LAM" => {
            return data_files(i).and_then(|(i, data_files)| {
                Ok((
                    i,
                    Data {
                        name,
                        values: DataValues::LAM(data_files),
                    },
                ))
            })
        }
        _ => {}
    }
    let mut params = vec![first];
    loop {
        match multiline_sep(float_unit)(i) {
            Ok((_i, first_n)) => {
                return map(
                    tuple((
                        many0_dummyfirst(multiline_sep(float_unit)),
                        space_newline,
                        enddata,
                    )),
                    |(mut values, _, _)| {
                        values[0] = first_n;
                        values
                    },
                )(_i)
                .and_then(|(i, values)| {
                    Ok((
                        i,
                        Data {
                            name,
                            values: DataValues::InlineNum { params, values },
                        },
                    ))
                });
            }
            Err(_) => {
                let param;
                let param_str;
                (i, (param_str, param)) = multiline_sep(key_str)(i)?;
                if param_str.to_uppercase().as_str() == "DATAFORM" {
                    return map(
                        tuple((many1(multiline_sep(value)), space_newline, enddata)),
                        |(values, _, _)| values,
                    )(i)
                    .and_then(|(i, values)| {
                        Ok((
                            i,
                            Data {
                                name,
                                values: DataValues::InlineExpr { params, values },
                            },
                        ))
                    });
                } else {
                    params.push(param);
                }
            }
        }
    }
}
#[inline]
pub(super) fn subckt<'a>(
    i: LocatedSpan<'a>,
    loaded: &IndexMap<FileId, Option<Pos>>,
    manager: &Arc<ParseManager>,
    work_dir: &Path,
) -> IResult<LocatedSpan<'a>, Subckt> {
    let ast_subckt = |i: LocatedSpan<'a>| -> IResult<LocatedSpan<'a>, AST> {
        ast(manager.clone(), loaded.clone(), work_dir.to_path_buf(), i)
    };
    map(
        tuple((
            space1,
            hierarchical_node,
            ports_params,
            space_newline,
            ast_subckt,
        )),
        |(_, name, (ports, params), _, ast)| Subckt {
            name,
            ports,
            params,
            ast,
        },
    )(i)
}

#[inline]
pub(super) fn local_ast<'a>(
    mut i: LocatedSpan<'a>,
    loaded: &IndexMap<FileId, Option<Pos>>,
    manager: &Arc<ParseManager>,
    work_dir: &Path,
) -> IResult<LocatedSpan<'a>, (LocalAST, EndReason<'a>)> {
    let mut ast = LocalAST::default();
    loop {
        log::trace!("\n{:?}", i.fragment());
        (i, _) = comment_space_newline(i)?;
        match char('.')(i) {
            Err(nom::Err::Error(_)) => match instance(i) {
                Ok((_i, inst)) => {
                    i = _i;
                    ast.instance.push(inst);
                }
                Err(e) => {
                    if i.is_empty() {
                        return Ok((i, (ast, EndReason::End)));
                    } else {
                        return Err(e);
                    }
                }
            },
            Err(e) => return Err(e),
            Ok((_i, _)) => {
                i = _i;
                let cmd;
                let cmd_str;
                (i, (cmd_str, cmd)) = key_str(i)?;
                match cmd_str.to_lowercase().as_str() {
                    "lib" => {
                        let lib_info;
                        (i, (_, lib_info)) = pair(space1, lib)(i)?;
                        if let Some((file_name_str, section_str)) = lib_info {
                            return Ok((
                                i,
                                (
                                    ast,
                                    EndReason::Include {
                                        file_name: Path::new(file_name_str),
                                        section: Some(section_str),
                                    },
                                ),
                            ));
                        }
                    }
                    "inc" | "include" => {
                        let file_name;
                        (i, (_, file_name)) = pair(space1, path_str)(i)?;
                        return Ok((
                            i,
                            (
                                ast,
                                EndReason::Include {
                                    file_name: Path::new(file_name),
                                    section: None,
                                },
                            ),
                        ));
                    }
                    "model" => {
                        let _model;
                        (i, _model) = model(i)?;
                        ast.model.push(_model);
                    }
                    "subckt" => {
                        let _subckt;
                        (i, _subckt) = subckt(i, loaded, manager, work_dir)?;
                        ast.subckt.push(_subckt);
                    }
                    "data" => {
                        let _data;
                        (i, _data) = data(i)?;
                        ast.data.push(_data);
                    }
                    "option" => {
                        let option;
                        (i, option) = many1(multiline_sep(token))(i)?;
                        ast.option.extend(option);
                    }
                    "param" | "parameter" => {
                        let param;
                        (i, param) = many1(multiline_sep(key_value))(i)?;
                        ast.param.extend(param);
                    }
                    "ends" => {
                        (i, _) = opt(pair(space1, key))(i)?;
                        return Ok((i, (ast, EndReason::End)));
                    }
                    "end" => {
                        return Ok((i, (ast, EndReason::End)));
                    }
                    _ => {
                        #[cfg(not(test))]
                        ast.errors.push(ParseErrorInner::Unknown(cmd).record(i));
                        #[cfg(test)]
                        ast.errors
                            .push(ParseErrorInner::Unknown(cmd.clone()).record(i));
                        let tokens;
                        (i, tokens) = many0(multiline_sep(token))(i)?;
                        ast.unknwon.push(Unknwon { cmd, tokens })
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    macro_rules! assert_ctx {
        ($i_res:expr, $ctx:expr $(,)?) => {
            assert_eq!($ctx, $i_res.unwrap().1.ctx);
        };
    }
    use std::path::PathBuf;

    // macro_rules! assert_kv {
    //     ($i_res:expr, $k:expr, $v:expr $(,)?) => {{
    //         let kv = $i_res.unwrap().1;
    //         assert_eq!($k, kv.k.ctx);
    //         assert_eq!($v, kv.v.ctx);
    //     }};
    // }
    // macro_rules! assert_token {
    //     ($i_res:expr, $k:expr, $v:expr $(,)?) => {{
    //         let token = $i_res.unwrap().1;
    //         if let Token::KV(kv) = token {
    //             assert_eq!($k, kv.k.ctx);
    //             assert_eq!($v, kv.v.ctx);
    //         } else {
    //             panic!("should be key-value!")
    //         }
    //     }};
    //     ($i_res:expr, $word:expr $(,)?) => {{
    //         let token = $i_res.unwrap().1;
    //         if let Token::Value(word) = token {
    //             assert_eq!($word, word.ctx);
    //         } else {
    //             panic!("should be word!")
    //         }
    //     }};
    // }
    use super::*;
    #[test]
    fn test_key() {
        assert_eq!(1.233, float_unit(span("1.233 ")).unwrap().1);
        assert_eq!(1.233e-6, float_unit(span("1.233u ")).unwrap().1);
        assert_ctx!(key(span("iw_ww ")), "iw_ww");
        assert_ctx!(key(span("iw.ww ")), "iw");
        assert_ctx!(hierarchical_node(span("iw.ww ")), "iw.ww");
        assert_ctx!(formula(span("8.00000000e-01+2")), "8.00000000e-01+2");
        assert_ctx!(unquote(span("'8.00000000e-01 + 2'")), "8.00000000e-01 + 2");
        // assert_kv!(
        //     key_value(span("a='8.00000000e-01 + 2'")),
        //     "a",
        //     "8.00000000e-01 + 2"
        // );
        // assert!(formula(span("'8.00000000e-01 + 2'")).is_err());
        // assert_token!(token(span("a")), "a");
        // assert_token!(
        //     token(span("a='8.00000000e-01 + 2'")),
        //     "a",
        //     "8.00000000e-01 + 2"
        // );
        crate::text_diff("'2+2'", value(span("2+2")).unwrap().1.to_string().as_str());
        crate::text_diff(
            "R1 node1 node2 var1='key1'",
            instance(span("R1 node1 node2 var1=key1"))
                .unwrap()
                .1
                .to_string()
                .as_str(),
        );
        crate::text_diff(
            "R1 node1 node2 var1='key1' var2='key2'",
            instance(span("R1 node1 node2 var1=key1 var2=key2"))
                .unwrap()
                .1
                .to_string()
                .as_str(),
        );
        let (manager, _) = ParseManager::new();
        let work_dir = PathBuf::new();
        let loaded = IndexMap::new();
        crate::text_diff(
            r#"
.MODEL nch_mac NMOS
+ level=2 version=4.5
.SUBCKT INV0SR_12TH40 I ZN VDD VSS
XX0 ZN I VSS VPW NHVT11LL_CKT W=0.00000031 L=0.00000004
XX3 ZN I VDD VNW PHVT11LL_CKT W=0.00000027 L=0.00000004
.ENDS INV0SR_12TH40
R1 node1 node2 var1='key1' var2='key2'
R2 node1 node2 var1='key1' var2='key2'
.mode 'nch_mac' 'nmos' level=2"#,
            local_ast(
                span(
                    r#"R1 node1 node2 var1=key1 var2=key2
        R2 node1 node2 var1=key1 
        +var2=key2
        .model nch_mac nmos level=2
        + version=4.5
        .SUBCKT INV0SR_12TH40 I ZN VDD VSS
XX0 ZN I VSS VPW NHVT11LL_CKT W=310.00n L=40.00n
XX3 ZN I VDD VNW PHVT11LL_CKT W=270.00n L=40.00n
.ENDS INV0SR_12TH40
.mode 'nch_mac' 'nmos' level=2
        "#,
                ),
                &loaded,
                &manager,
                &work_dir,
            )
            .unwrap()
            .1
             .0
            .to_string()
            .as_str(),
        );
        crate::text_diff(
            r#"
.MODEL nch_mac NMOS
+ level=2 version=4.5
.SUBCKT INV0SR_12TH40 I ZN VDD VSS
XX0 ZN I VSS VPW NHVT11LL_CKT W=0.00000031 L=0.00000004
XX3 ZN I VDD VNW PHVT11LL_CKT W=0.00000027 L=0.00000004
.ENDS INV0SR_12TH40
R1 node1 node2 var1='key1' var2='key2'
R2 node1 node2 var1='key1' var2='key2'
.mode 'nch_mac' 'nmos' level=2"#,
            local_ast(
                span(
                    r#"R1 node1 node2 var1=key1 var2=key2
        R2 node1 node2 var1=key1 
        +var2=key2
        .model nch_mac 
        + nmos (
            + level=2
        + version=4.5
    + )
        .SUBCKT INV0SR_12TH40 I ZN VDD VSS
XX0 ZN I VSS VPW NHVT11LL_CKT W=310.00n L=40.00n
XX3 ZN I VDD VNW PHVT11LL_CKT W=270.00n L=40.00n
.ENDS INV0SR_12TH40
.mode 'nch_mac' 'nmos' level=2
        "#,
                ),
                &loaded,
                &manager,
                &work_dir,
            )
            .unwrap()
            .1
             .0
            .to_string()
            .as_str(),
        );
        crate::text_diff(
            r#"
.DATA SWEEP
+ var1 var2
+ 1 2
+ 2 4
.ENDDATA"#,
            local_ast(
                span(
                    r#".data SWEEP
                    + var1 var2
                    + 1    2
                    + 2    4
                    .enddata
        "#,
                ),
                &loaded,
                &manager,
                &work_dir,
            )
            .unwrap()
            .1
             .0
            .to_string()
            .as_str(),
        );
        crate::text_diff(
            r#"
.DATA SWEEP
+ var1 var2 DATAFORM
+ 1 '2+2'
+ 2 4
.ENDDATA"#,
            local_ast(
                span(
                    r#".data SWEEP
                    + var1 var2 DATAFORM
                    + 1 2+2
                    + 2 4
                    .enddata
        "#,
                ),
                &loaded,
                &manager,
                &work_dir,
            )
            .unwrap()
            .1
             .0
            .to_string()
            .as_str(),
        );
        crate::text_diff(
            r#"
.DATA inputdata MER
+ FILE='file1' p1=1 p2=3 p3=4
+ FILE='file2' p1=1
+ FILE='file3' p1=1
.ENDDATA"#,
            local_ast(
                span(
                    r#".DATA inputdata MER
                    + FILE='file1' p1=1 p2=3 p3=4
                    + FILE='file2' p1=1
                    + FILE='file3' p1=1
                 .ENDDATA
        "#,
                ),
                &loaded,
                &manager,
                &work_dir,
            )
            .unwrap()
            .1
             .0
            .to_string()
            .as_str(),
        );
    }

    #[test]
    fn libmatch() {
        println!("{:?}", lib(span("some .EndL \nlines")));
        println!("{:?}", lib(span("some .EndL tt \nlines")));
        println!("{:?}", lib(span("some \n .EndL lines")));
        println!("{:?}", lib(span("'some' tt")));
    }
}
