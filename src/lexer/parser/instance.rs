use super::super::{
    builder::{
        instance::{
            BJT, Capacitor, Current, CurrentSource, Diode, Inductor, Instance, InstanceCtx, MOSFET,
            PWL, Resistor, Subckt, TimeValuePoint, Voltage, VoltageSource,
        },
        span::LocatedSpan,
    },
    parser::utils::{
        key_value, loss_sep, many0_dummyfirst, multiline_sep, name, name_char, name_str,
        ports_params, value,
    },
};
use nom::{
    IResult, Parser,
    branch::alt,
    character::char,
    combinator::{map, map_res},
    multi::many1,
};

#[inline]
pub(super) fn instance(mut i: LocatedSpan) -> IResult<LocatedSpan, Instance> {
    let first_char: _;
    let name: _;
    (i, (first_char, name)) = name_char(i)?;
    let parser = match first_char.to_ascii_lowercase() {
        b'r' => _resistor,
        b'c' => _capacitor,
        b'l' => _inductor,
        b'v' => _voltage,
        b'i' => _current,
        b'm' => _mosfet,
        b'q' => _bjt,
        b'd' => _diode,
        b'x' => _subckt,
        r#type => {
            return map(ports_params, |(ports, params)| Instance {
                name,
                ctx: InstanceCtx::Unknown {
                    r#type,
                    ports,
                    params,
                },
            })
            .parse(i);
        }
    };
    map(parser, |ctx| Instance { name, ctx }).parse(i)
}

#[inline]
fn _resistor(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtx> {
    map(
        (
            multiline_sep(name),
            multiline_sep(name),
            multiline_sep(value),
        ),
        |(n1, n2, value)| InstanceCtx::Resistor(Resistor { n1, n2, value }),
    )
    .parse(i)
}
#[inline]
fn _capacitor(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtx> {
    map(
        (
            multiline_sep(name),
            multiline_sep(name),
            multiline_sep(value),
        ),
        |(n1, n2, value)| InstanceCtx::Capacitor(Capacitor { n1, n2, value }),
    )
    .parse(i)
}
#[inline]
fn _inductor(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtx> {
    map(
        (
            multiline_sep(name),
            multiline_sep(name),
            multiline_sep(value),
        ),
        |(n1, n2, value)| InstanceCtx::Inductor(Inductor { n1, n2, value }),
    )
    .parse(i)
}
#[inline]
fn _mosfet(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtx> {
    map_res(ports_params, |(ports, params)| match ports.len() {
        4 => Ok(InstanceCtx::MOSFET(MOSFET {
            nd: ports[0],
            ng: ports[1],
            ns: ports[2],
            nb: None,
            mname: ports[3],
            params,
        })),
        5 => Ok(InstanceCtx::MOSFET(MOSFET {
            nd: ports[0],
            ng: ports[1],
            ns: ports[2],
            nb: Some(ports[3]),
            mname: ports[4],
            params,
        })),
        0 => Err("There is no model name".to_string()),
        n => Err(format!(
            "MOSFET is 3/4 ports device, but found {} port(s)",
            n - 1
        )),
    })
    .parse(i)
}
#[inline]
fn _bjt(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtx> {
    map_res(ports_params, |(ports, params)| match ports.len() {
        4 => Ok(InstanceCtx::BJT(BJT {
            nc: ports[0],
            nb: ports[1],
            ne: ports[2],
            ns: None,
            mname: ports[3],
            params,
        })),
        5 => Ok(InstanceCtx::BJT(BJT {
            nc: ports[0],
            nb: ports[1],
            ne: ports[2],
            ns: Some(ports[3]),
            mname: ports[4],
            params,
        })),
        0 => Err("There is no model name".to_string()),
        n => Err(format!(
            "BJT is 3/4 ports device, but found {} port(s)",
            n - 1
        )),
    })
    .parse(i)
}
#[inline]
fn _diode(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtx> {
    map_res(ports_params, |(ports, params)| match ports.len() {
        3 => Ok(InstanceCtx::Diode(Diode {
            nplus: ports[0],
            nminus: ports[1],
            mname: ports[2],
            params,
        })),
        0 => Err("There is no model name".to_string()),
        n => Err(format!(
            "Diode is 2 ports device, but found {} port(s)",
            n - 1
        )),
    })
    .parse(i)
}
#[inline]
fn _subckt(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtx> {
    map_res(ports_params, |(mut ports, params)| {
        if let Some(cktname) = ports.pop() {
            Ok(InstanceCtx::Subckt(Subckt {
                ports,
                cktname,
                params,
            }))
        } else {
            Err("There is no subckt name")
        }
    })
    .parse(i)
}
#[inline]
fn _pwl(i: LocatedSpan) -> IResult<LocatedSpan, PWL> {
    map_res(
        (
            multiline_sep(name_str),
            loss_sep,
            char('('),
            loss_sep,
            value,
            many0_dummyfirst(multiline_sep(value)),
            loss_sep,
            char(')'),
        ),
        |((tag, _), _, _, _, first, mut points, _, _)| {
            if tag.to_ascii_uppercase().eq("PWL") {
                if let Some(dummy_first) = points.first_mut() {
                    *dummy_first = first;
                    if points.len() % 2 != 0 {
                        Err("points should be 2N numbers")
                    } else {
                        Ok(PWL {
                            points: points
                                .chunks_exact(2)
                                .map(|values| TimeValuePoint {
                                    time: values[0],
                                    value: values[1],
                                })
                                .collect(),
                            ..Default::default()
                        })
                    }
                } else {
                    Err("points should at least 2")
                }
            } else {
                Err("Want PWL keyword")
            }
        },
    )
    .parse(i)
}
#[inline]
fn _voltage(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtx> {
    map(
        (
            multiline_sep(name),
            multiline_sep(name),
            alt((
                map(many1(multiline_sep(key_value)), |params| {
                    VoltageSource::Params(params)
                }),
                map(_pwl, |pwl| VoltageSource::PWL(pwl)),
                map(multiline_sep(value), |value| VoltageSource::Value(value)),
            )),
        ),
        |(n1, n2, source)| InstanceCtx::Voltage(Voltage { n1, n2, source }),
    )
    .parse(i)
}
#[inline]
fn _current(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtx> {
    map(
        (
            multiline_sep(name),
            multiline_sep(name),
            alt((
                map(many1(multiline_sep(key_value)), |params| {
                    CurrentSource::Params(params)
                }),
                map(_pwl, |pwl| CurrentSource::PWL(pwl)),
                map(multiline_sep(value), |value| CurrentSource::Value(value)),
            )),
        ),
        |(n1, n2, source)| InstanceCtx::Current(Current { n1, n2, source }),
    )
    .parse(i)
}
