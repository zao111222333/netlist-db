#![expect(clippy::type_complexity)]
use std::path::PathBuf;

use super::{
    BEGIN_TITLE,
    utils::{name_str, space, space_newline},
};
use crate::{err::ParseError, span::LocatedSpan};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::char,
    combinator::{map, opt},
    multi::{many1, separated_list1},
    sequence::delimited,
};

const FAILED_MEAS: &str = "failed";

#[inline]
fn float(i: LocatedSpan) -> IResult<LocatedSpan, Option<f64>> {
    alt((
        map(super::utils::float, Some),
        map(tag(FAILED_MEAS), |_| None),
    ))
    .parse_complete(i)
}

#[inline]
fn data_meas_csv_nom(
    i: LocatedSpan<'_>,
) -> IResult<LocatedSpan<'_>, (Vec<&str>, Vec<Vec<Option<f64>>>)> {
    map(
        (
            take_until(BEGIN_TITLE),
            take_until("\n"),
            space_newline,
            separated_list1((space, char(','), space), map(name_str, |(s, _)| s)),
            opt(char('#')),
            opt((space, char(','))),
            many1(delimited(
                space_newline,
                separated_list1((space, char(','), space), float),
                opt((space, char(','))),
            )),
        ),
        |(_, _, _, names, _, _, value_table): (_, _, _, Vec<&str>, _, _, Vec<Vec<Option<f64>>>)| {
            (names, value_table)
        },
    )
    .parse_complete(i)
}

pub struct MeasCols<'a> {
    names: Vec<&'a str>,
    idx: usize,
    vt: &'a [Vec<Option<f64>>],
    prefix: Option<&'a str>,
}

impl<'a> MeasCols<'a> {
    pub fn new(
        names: Vec<&'a str>,
        value_table: &'a Vec<Vec<Option<f64>>>,
        data_prefix: Option<&'a str>,
    ) -> Self {
        Self {
            names,
            idx: 0,
            vt: value_table,
            prefix: data_prefix,
        }
    }
}

pub struct Col<'a> {
    vt: &'a [Vec<Option<f64>>],
    col: usize,
    row: usize,
}

impl<'a> Iterator for MeasCols<'a> {
    type Item = (&'a str, Col<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.names.len() {
            let i = self.idx;
            let name = self.names[i];
            self.idx += 1;

            if let Some(p) = self.prefix {
                if !name.starts_with(p) {
                    continue;
                }
            }

            return Some((
                name,
                Col {
                    vt: self.vt,
                    col: i,
                    row: 0,
                },
            ));
        }
        None
    }
}

impl<'a> Iterator for Col<'a> {
    type Item = Option<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row >= self.vt.len() {
            return None;
        }
        let v = self.vt[self.row][self.col]; // Option<f64>: Copy
        self.row += 1;
        Some(v)
    }
}

#[inline]
pub fn data_meas_csv(path: PathBuf, s: &str) -> Option<(Vec<&str>, Vec<Vec<Option<f64>>>)> {
    match data_meas_csv_nom(s.into()) {
        Ok((_, out)) => Some(out),
        Err(e) => {
            let err: ParseError = e.into();
            err.report(&mut true, &crate::FileId::Include { path }, s);
            None
        }
    }
}

#[test]
fn sim_mt0_csv() {
    crate::utlis::test::init_logger();
    const DATA: &str = include_str!("../../tests/sim.mt0.csv");
    let (names, value_table) =
        data_meas_csv(PathBuf::from("tests/sim.mt0.csv"), DATA.into()).unwrap();
    for (k, v) in MeasCols::new(names, &value_table, Some("kcell")) {
        dbg!(k, v.collect::<Vec<_>>());
    }
}
