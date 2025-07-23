use std::{collections::HashMap, path::PathBuf};

use super::utils::{name_str, space, space_newline};
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
use tokio::fs::read_to_string;

const BEGIN_TITLE: &str = ".TITLE";
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
fn data_meas_csv_nom<'a>(
    i: LocatedSpan<'a>,
    data_prefix: Option<&str>,
) -> IResult<LocatedSpan<'a>, HashMap<String, Vec<Option<f64>>>> {
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
            names
                .into_iter()
                .enumerate()
                .filter_map(|(name_idx, name)| {
                    if let Some(prefix) = data_prefix {
                        if !name.starts_with(prefix) {
                            return None;
                        }
                    }
                    Some((
                        name.to_owned(),
                        value_table.iter().map(|values| values[name_idx]).collect(),
                    ))
                })
                .collect()
        },
    )
    .parse_complete(i)
}

#[inline]
pub async fn data_meas_csv(
    path: PathBuf,
    data_prefix: Option<&str>,
) -> Result<HashMap<String, Vec<Option<f64>>>, ()> {
    match read_to_string(&path).await {
        Ok(s) => {
            let (_, out) = data_meas_csv_nom(s.as_str().into(), data_prefix).map_err(|e| {
                let err: ParseError = e.into();
                err.report(&mut true, &crate::FileId::Include { path }, &s);
            })?;
            Ok(out)
        }
        Err(e) => {
            let err: ParseError = e.into();
            err.report(&mut true, &crate::FileId::Include { path }, "");
            Err(())
        }
    }
}

#[tokio::test]
async fn sim_mt0_csv() {
    crate::utlis::test::init_logger();
    const DATA: &str = include_str!("../../tests/sim.mt0.csv");
    _ = dbg!(data_meas_csv_nom(DATA.into(), None));
    _ = dbg!(
        data_meas_csv("tests/sim.mt0.csv".into(), Some("kcell"))
            .await
            .unwrap()
    );
}
