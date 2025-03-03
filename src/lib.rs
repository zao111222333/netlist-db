extern crate alloc;
#[cfg(test)]
mod _test_utils;

mod _impl_display;
mod builder;
mod err;
pub mod instance;
pub mod parser;

use alloc::borrow::Cow;
use builder::{
    Builder as _,
    span::{FileId, ParsedId},
};
use err::{ParseError, ParseErrorInner};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Parsed {
    pub top_id: ParsedId,
    pub id2idx: HashMap<FileId, ParsedId>,
    pub inner: Vec<(FileId, builder::AST)>,
}
#[derive(Debug)]
pub struct Files {
    pub inner: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Value<'s> {
    Num(f64),
    Expr(Cow<'s, str>),
}

#[derive(Debug, Clone)]
pub struct KeyValue<'s> {
    pub k: Cow<'s, str>,
    pub v: Value<'s>,
}

#[derive(Debug, Clone)]
pub enum Token<'s> {
    KV(KeyValue<'s>),
    Value(Value<'s>),
    V(Cow<'s, str>),
    I(Cow<'s, str>),
}

/// ``` spice
/// .subckt pulvt11ll_ckt d g s b w=1e-6 l=1e-6 sa='sar'
/// ...
/// .ends pulvt11ll_ckt
/// ```
/// Do NOT support `.include` / `.lib` in `.subckt`
#[derive(Debug, Clone)]
pub struct Subckt<'s> {
    pub name: Cow<'s, str>,
    /// subckt/model name is the last arg
    pub ports: Vec<Cow<'s, str>>,
    pub params: Vec<KeyValue<'s>>,
    pub ast: AST<'s>,
}

#[derive(Debug, Clone)]
pub struct General<'s> {
    pub cmd: builder::GeneralCmd,
    pub tokens: Vec<Token<'s>>,
}

#[derive(Debug, Clone)]
pub struct Unknwon<'s> {
    pub cmd: Cow<'s, str>,
    pub tokens: Vec<Token<'s>>,
}

#[derive(Debug, Clone)]
pub struct Model<'s> {
    pub name: Cow<'s, str>,
    pub model_type: ModelType<'s>,
    pub params: Vec<KeyValue<'s>>,
}

#[derive(Debug, Clone)]
pub struct Data<'s> {
    pub name: Cow<'s, str>,
    pub values: DataValues<'s>,
}

#[derive(Debug, Clone)]
pub enum DataValues<'s> {
    InlineExpr {
        params: Vec<Cow<'s, str>>,
        values: Vec<Value<'s>>,
    },
    InlineNum {
        params: Vec<Cow<'s, str>>,
        values: Vec<f64>,
    },
    /// https://eda-cpu1.eias.junzhuo.site/~junzhuo/hspice/index.htm#page/hspice_14/data.htm
    /// Concatenated (series merging) data files to use.
    MER(),
    /// Column-laminated (parallel merging) data files to use.
    LAM(),
}
#[cfg(feature = "py")]
use polars::{error::PolarsError, frame::DataFrame, prelude::Column};
pub struct DataValuesCsv<'s, 'a>(pub(crate) &'a DataValues<'s>);
impl<'s> DataValues<'s> {
    pub fn csv(&self) -> DataValuesCsv<'s, '_> {
        DataValuesCsv(self)
    }
    #[cfg(feature = "py")]
    pub fn dataframe(&self) -> Result<DataFrame, PolarsError> {
        if let Self::InlineNum { params, values } = self {
            let ncols = params.len();
            if ncols == 0 {
                return Err(PolarsError::ComputeError("Header is empty".into()));
            }
            if values.len() % ncols != 0 {
                return Err(PolarsError::ComputeError(
                    "Data length is not a multiple of the number of columns".into(),
                ));
            }
            let nrows = values.len() / ncols;
            let columns = params
                .into_iter()
                .enumerate()
                .map(|(col_idx, name)| {
                    Column::new(
                        name.as_ref().into(),
                        (0..nrows)
                            .into_iter()
                            .map(|row| values[row * ncols + col_idx])
                            .collect::<Vec<f64>>(),
                    )
                })
                .collect();
            DataFrame::new(columns)
        } else {
            Err(PolarsError::ComputeError("Is not inline data".into()))
        }
    }
}
#[expect(clippy::upper_case_acronyms)]
#[derive(Debug, Clone)]
pub enum ModelType<'s> {
    /// operational amplifier model
    AMP,
    /// capacitor model
    C,
    /// magnetic core model
    CORE,
    /// diode model
    D,
    /// inductor model or magnetic core mutual inductor model
    L,
    /// n-channel JFET model
    NJF,
    /// n-channel MOSFET model
    NMOS,
    /// npn BJT model
    NPN,
    /// optimization model
    OPT,
    /// p-channel JFET model
    PJF,
    /// p-channel MOSFET model
    PMOS,
    /// pnp BJT model
    PNP,
    /// resistor model
    R,
    /// lossy transmission line model (lumped)
    U,
    /// lossy transmission line model
    W,
    /// S-parameter
    S,
    Unknown(Cow<'s, str>),
}
#[expect(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Default)]
pub struct AST<'s> {
    pub subckt: Vec<Subckt<'s>>,
    pub instance: Vec<instance::Instance<'s>>,
    pub model: Vec<Model<'s>>,
    pub param: Vec<KeyValue<'s>>,
    pub option: Vec<(Cow<'s, str>, Option<Value<'s>>)>,
    /// transient initial conditions
    /// https://eda-cpu1.eias.junzhuo.site/~junzhuo/hspice/index.htm#page/hspice_14/ic.htm
    ///
    /// `node, val, [subckt]`
    pub init_condition: Vec<(Cow<'s, str>, Value<'s>, Option<Cow<'s, str>>)>,
    pub general: Vec<General<'s>>,
    pub data: Vec<Data<'s>>,
    pub unknwon: Vec<Unknwon<'s>>,
}

impl builder::AST {
    #[expect(clippy::too_many_arguments)]
    fn build<'s>(
        &self,
        ast: &mut AST<'s>,
        has_err: &mut bool,
        file_id: &FileId,
        parsed_id: ParsedId,
        files: &'s Files,
        parsed: &Parsed,
    ) {
        fn build_local<'s>(
            local_ast: &builder::LocalAST,
            ast: &mut AST<'s>,
            has_err: &mut bool,
            file: &'s str,
            file_id: &FileId,
            parsed_id: ParsedId,
            files: &'s Files,
            parsed: &Parsed,
        ) {
            fn build_subckt<'s>(
                s: &builder::Subckt,
                has_err: &mut bool,
                file: &'s str,
                file_id: &FileId,
                parsed_id: ParsedId,
                files: &'s Files,
                parsed: &Parsed,
            ) -> Subckt<'s> {
                let mut ast = AST::default();
                s.ast
                    .build(&mut ast, has_err, file_id, parsed_id, files, parsed);
                Subckt {
                    name: s.name.build(file),
                    ports: s.ports.build(file),
                    params: s.params.build(file),
                    ast,
                }
            }
            ast.subckt.extend(
                local_ast
                    .subckt
                    .iter()
                    .map(|s| build_subckt(s, has_err, file, file_id, parsed_id, files, parsed)),
            );
            ast.instance
                .extend(local_ast.instance.iter().map(|b| b.build(file)));
            ast.model
                .extend(local_ast.model.iter().map(|b| b.build(file)));
            ast.param
                .extend(local_ast.param.iter().map(|b| b.build(file)));
            ast.option
                .extend(local_ast.option.iter().map(|b| b.build(file)));
            ast.general
                .extend(local_ast.general.iter().map(|b| b.build(file)));
            ast.data
                .extend(local_ast.data.iter().map(|b| b.build(file)));
            ast.init_condition
                .extend(local_ast.init_condition.iter().map(|b| b.build(file)));
            ast.unknwon
                .extend(local_ast.unknwon.iter().map(|b| b.build(file)));
            for e in &local_ast.errors {
                e.report(has_err, file_id, file);
            }
        }
        let file = &files.inner[parsed_id.0];
        for seg in &self.segments {
            match seg {
                builder::Segment::Local(local_ast) => {
                    build_local(
                        local_ast, ast, has_err, file, file_id, parsed_id, files, parsed,
                    );
                }
                builder::Segment::Include(ast_res) => {
                    let ast_res = ast_res.get().unwrap();
                    match ast_res {
                        Ok(parsed_id) => {
                            let (file_id, _ast) = &parsed.inner[parsed_id.0];
                            _ast.build(ast, has_err, file_id, *parsed_id, files, parsed);
                        }
                        Err(e) => {
                            e.report(has_err, file_id, file);
                        }
                    }
                }
            }
        }
    }
}

impl Files {
    #[inline]
    pub fn build(&self, parsed: Parsed) -> (AST<'_>, bool) {
        let mut ast = AST::default();
        let mut has_err = false;
        let (file_id, _ast) = &parsed.inner[parsed.top_id.0];
        _ast.build(
            &mut ast,
            &mut has_err,
            file_id,
            parsed.top_id,
            self,
            &parsed,
        );
        (ast, has_err)
    }
}
