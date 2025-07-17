use core::fmt;
use std::{
    borrow::Cow,
    collections::HashMap,
    hash::{Hash, Hasher},
    path::PathBuf,
};

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take, take_until},
    character::complete::char,
    combinator::{map, map_res, opt},
    multi::many1,
    sequence::preceded,
};
use tokio::fs::read_to_string;

use crate::{err::ParseError, span::LocatedSpan};

use super::utils::{name_str, space_newline};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum DataType {
    V,
    I,
    P,
}
#[derive(Debug, Clone)]
pub struct DataName {
    r#type: DataType,
    name: Cow<'static, str>,
}

impl fmt::Display for DataName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}({})",
            match self.r#type {
                DataType::V => 'V',
                DataType::I => 'I',
                DataType::P => 'P',
            },
            self.name
        )
    }
}

impl DataName {
    #[inline]
    pub fn new_volt(name: Cow<'static, str>) -> Self {
        Self {
            r#type: DataType::V,
            name,
        }
    }
    #[inline]
    pub fn new_current(name: Cow<'static, str>) -> Self {
        Self {
            r#type: DataType::I,
            name,
        }
    }
    #[inline]
    pub fn new_param(name: Cow<'static, str>) -> Self {
        Self {
            r#type: DataType::P,
            name,
        }
    }
}

impl PartialEq for DataName {
    fn eq(&self, other: &Self) -> bool {
        self.r#type.eq(&other.r#type) && self.name.to_lowercase().eq(&other.name.to_lowercase())
    }
}
impl Eq for DataName {}

impl Hash for DataName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.r#type.hash(state);
        self.name.to_lowercase().hash(state);
    }
}

#[inline]
fn data_type(i: LocatedSpan) -> IResult<LocatedSpan, DataType> {
    alt((
        map(char('v'), |_| DataType::V),
        map(char('i'), |_| DataType::I),
        map(char('p'), |_| DataType::P),
    ))
    .parse_complete(i)
}

#[inline]
fn data_name(i: LocatedSpan) -> IResult<LocatedSpan, DataName> {
    map(
        (
            data_type,
            char('('),
            (name_str, opt(preceded(char('\n'), name_str))),
        ),
        |(r#type, _, ((s1, _), n2))| DataName {
            r#type,
            name: if let Some((s2, _)) = n2 {
                s1.to_owned() + s2
            } else {
                s1.to_owned()
            }
            .into(),
        },
    )
    .parse_complete(i)
}
#[inline]
fn float(i: LocatedSpan) -> IResult<LocatedSpan, f64> {
    map_res(take(13u32), |s: LocatedSpan| {
        fast_float2::parse(s.fragment().as_bytes())
    })
    .parse_complete(i)
}

const SWEEP_FLAG: &str = "sweep";
const TERMINATION: &str = "$&%#";

#[inline]
pub async fn data_sweep(path: PathBuf) -> Result<HashMap<DataName, Vec<f64>>, ()> {
    match read_to_string(&path).await {
        Ok(s) => {
            let (_, out) = data_sweep_nom(s.as_str().into()).map_err(|e| {
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

#[inline]
fn data_sweep_nom(i: LocatedSpan) -> IResult<LocatedSpan, HashMap<DataName, Vec<f64>>> {
    map(
        (
            take_until(SWEEP_FLAG),
            take(SWEEP_FLAG.len()),
            many1(preceded(space_newline, data_name)),
            space_newline,
            tag(TERMINATION),
            space_newline,
            many1(preceded(space_newline, float)),
        ),
        |(_, _, names, _, _, _, values)| {
            let name_len = names.len() + 1;
            let size = values.len() / name_len;
            names
                .into_iter()
                .enumerate()
                .map(|(name_idx, n)| {
                    (
                        n,
                        (0..size)
                            .map(|i| values[i * name_len + name_idx + 1])
                            .collect(),
                    )
                })
                .collect()
        },
    )
    .parse_complete(i)
}

#[tokio::test]
async fn sim_sw0() {
    #[cfg(not(feature = "tracing"))]
    {
        _ = simple_logger::SimpleLogger::new().init();
    }
    #[cfg(feature = "tracing")]
    {
        let subscriber = tracing_subscriber::FmtSubscriber::builder()
            // .with_ansi(colored::control::SHOULD_COLORIZE.should_colorize())
            .with_max_level(tracing::Level::DEBUG)
            .with_target(false)
            .with_file(true)
            .with_line_number(true)
            .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
                "%FT%T".to_owned(),
            ))
            .finish();
        _ = tracing::subscriber::set_global_default(subscriber);
    }
    const DATA: &str = include_str!("../../tests/sim.sw0");
    _ = dbg!(data_sweep_nom(DATA.into()));
    _ = dbg!(data_sweep("tests/sim.sw0".into()).await);
}
