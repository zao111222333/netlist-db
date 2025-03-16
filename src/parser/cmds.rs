use super::super::{
    builder::{
        Value,
        span::{LocatedSpan, Span},
    },
    parser::utils::{loss_sep, multiline_sep, name, name_str, v, value},
};
use nom::{
    IResult, Parser,
    character::char,
    combinator::{map, map_res, opt},
    multi::many1,
};

#[inline]
fn subckt(i: LocatedSpan) -> IResult<LocatedSpan, Span> {
    map_res(
        (name_str, loss_sep, char('='), loss_sep, name),
        |((keyword, _), _, _, _, name)| {
            if keyword.to_lowercase().eq("subckt") {
                Ok(name)
            } else {
                Err("want subckt")
            }
        },
    )
    .parse_complete(i)
}

#[inline]
pub(super) fn init_condition(
    i: LocatedSpan,
) -> IResult<LocatedSpan, impl Iterator<Item = (Span, Value, Option<Span>)>> {
    #[inline]
    fn node_volt(i: LocatedSpan) -> IResult<LocatedSpan, (Span, Value)> {
        map(
            (v, loss_sep, char('='), loss_sep, value),
            |(node, _, _, _, val)| (node, val),
        )
        .parse_complete(i)
    }
    map(
        (many1(multiline_sep(node_volt)), opt(multiline_sep(subckt))),
        |(iter, opt_subckt)| {
            iter.into_iter()
                .map(move |(node, val)| (node, val, opt_subckt))
        },
    )
    .parse_complete(i)
}

#[inline]
pub(super) fn nodeset(
    i: LocatedSpan,
) -> IResult<LocatedSpan, impl Iterator<Item = (Span, Value, Option<Span>)>> {
    #[inline]
    fn node_volt(i: LocatedSpan) -> IResult<LocatedSpan, (Span, Value)> {
        map(
            (name, loss_sep, opt(char('=')), loss_sep, value),
            |(node, _, _, _, val)| (node, val),
        )
        .parse_complete(i)
    }
    map(
        (many1(multiline_sep(node_volt)), opt(multiline_sep(subckt))),
        |(iter, opt_subckt)| {
            iter.into_iter()
                .map(move |(node, val)| (node, val, opt_subckt))
        },
    )
    .parse_complete(i)
}
