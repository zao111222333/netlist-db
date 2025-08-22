use std::path::PathBuf;

use super::{
    BEGIN_TITLE,
    utils::{name_str, space, space_newline},
};
use crate::{err::ParseError, span::LocatedSpan};
use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_until},
    character::complete::char,
    combinator::{map, opt},
    multi::separated_list1,
};
use tokio::fs::read_to_string;

#[inline]
fn data_mc0_variation<'a>(i: LocatedSpan<'a>) -> IResult<LocatedSpan<'a>, Vec<String>> {
    map(
        (
            take_until(BEGIN_TITLE),
            take_until("\n"),
            space_newline,
            opt((tag("index"), space, char(','), space)),
            separated_list1(
                (space, char(','), space),
                map(name_str, |(s, _)| String::from(s)),
            ),
        ),
        |(_, _, _, _, variation)| variation,
    )
    .parse_complete(i)
}

#[inline]
pub async fn data_mc0_variation_csv(path: PathBuf) -> Result<Vec<String>, ()> {
    match read_to_string(&path).await {
        Ok(s) => {
            let (_, out) = data_mc0_variation(s.as_str().into()).map_err(|e| {
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
async fn data_mc0_variation_csv() {
    crate::utlis::test::init_logger();
    _ = dbg!(
        data_csv_mc0_variation_file("tests/sim.mc0.csv".into())
            .await
            .unwrap()
    );
}
