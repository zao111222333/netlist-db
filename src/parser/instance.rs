use super::utils::{
    key_value, loss_sep, many0_dummyfirst, multiline_sep, name, name_char, name_str, ports_params,
    value,
};
use crate::{
    instance::{
        BJTBuilder, CapacitorBuilder, CurrentBuilder, CurrentSourceBuilder, DiodeBuilder,
        InductorBuilder, InstanceBuilder, InstanceCtxBuilder, MOSFETBuilder, PWLBuilder,
        ResistorBuilder, SubcktBuilder, TimeValuePointBuilder, VoltageBuilder,
        VoltageSourceBuilder,
    },
    span::LocatedSpan,
};
use nom::{
    IResult, Parser,
    branch::alt,
    character::char,
    combinator::{map, map_res},
    multi::many1,
};

#[inline]
pub(super) fn instance(mut i: LocatedSpan) -> IResult<LocatedSpan, InstanceBuilder> {
    let first_char;
    let name;
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
            return map(ports_params, |(nodes, params)| InstanceBuilder {
                name,
                ctx: InstanceCtxBuilder::Unknown {
                    r#type,
                    nodes,
                    params,
                },
            })
            .parse_complete(i);
        }
    };
    map(parser, |ctx| InstanceBuilder { name, ctx }).parse_complete(i)
}

#[inline]
fn _resistor(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtxBuilder> {
    map(
        (
            multiline_sep(name),
            multiline_sep(name),
            multiline_sep(value),
        ),
        |(n1, n2, value)| InstanceCtxBuilder::Resistor(ResistorBuilder { n1, n2, value }),
    )
    .parse_complete(i)
}
#[inline]
fn _capacitor(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtxBuilder> {
    map(
        (
            multiline_sep(name),
            multiline_sep(name),
            multiline_sep(value),
        ),
        |(n1, n2, value)| InstanceCtxBuilder::Capacitor(CapacitorBuilder { n1, n2, value }),
    )
    .parse_complete(i)
}
#[inline]
fn _inductor(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtxBuilder> {
    map(
        (
            multiline_sep(name),
            multiline_sep(name),
            multiline_sep(value),
        ),
        |(n1, n2, value)| InstanceCtxBuilder::Inductor(InductorBuilder { n1, n2, value }),
    )
    .parse_complete(i)
}
#[inline]
fn _mosfet(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtxBuilder> {
    map_res(ports_params, |(ports, params)| match ports.len() {
        4 => Ok(InstanceCtxBuilder::MOSFET(MOSFETBuilder {
            nd: ports[0],
            ng: ports[1],
            ns: ports[2],
            nb: None,
            mname: ports[3],
            params,
        })),
        5 => Ok(InstanceCtxBuilder::MOSFET(MOSFETBuilder {
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
    .parse_complete(i)
}
#[inline]
fn _bjt(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtxBuilder> {
    map_res(ports_params, |(ports, params)| match ports.len() {
        4 => Ok(InstanceCtxBuilder::BJT(BJTBuilder {
            nc: ports[0],
            nb: ports[1],
            ne: ports[2],
            ns: None,
            mname: ports[3],
            params,
        })),
        5 => Ok(InstanceCtxBuilder::BJT(BJTBuilder {
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
    .parse_complete(i)
}
#[inline]
fn _diode(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtxBuilder> {
    map_res(ports_params, |(ports, params)| match ports.len() {
        3 => Ok(InstanceCtxBuilder::Diode(DiodeBuilder {
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
    .parse_complete(i)
}
#[inline]
fn _subckt(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtxBuilder> {
    map_res(ports_params, |(mut nodes, params)| {
        if let Some(cktname) = nodes.pop() {
            Ok(InstanceCtxBuilder::Subckt(SubcktBuilder {
                nodes,
                cktname,
                params,
            }))
        } else {
            Err("There is no subckt name")
        }
    })
    .parse_complete(i)
}
#[inline]
fn _pwl(i: LocatedSpan) -> IResult<LocatedSpan, PWLBuilder> {
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
                        Ok(PWLBuilder {
                            points: points
                                .chunks_exact(2)
                                .map(|values| TimeValuePointBuilder {
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
    .parse_complete(i)
}
#[inline]
fn _voltage(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtxBuilder> {
    map(
        (
            multiline_sep(name),
            multiline_sep(name),
            alt((
                map(many1(multiline_sep(key_value)), |params| {
                    VoltageSourceBuilder::Params(params)
                }),
                map(_pwl, VoltageSourceBuilder::PWL),
                map(multiline_sep(value), VoltageSourceBuilder::Value),
            )),
        ),
        |(n1, n2, source)| InstanceCtxBuilder::Voltage(VoltageBuilder { n1, n2, source }),
    )
    .parse_complete(i)
}
#[inline]
fn _current(i: LocatedSpan) -> IResult<LocatedSpan, InstanceCtxBuilder> {
    map(
        (
            multiline_sep(name),
            multiline_sep(name),
            alt((
                map(many1(multiline_sep(key_value)), |params| {
                    CurrentSourceBuilder::Params(params)
                }),
                map(_pwl, CurrentSourceBuilder::PWL),
                map(multiline_sep(value), CurrentSourceBuilder::Value),
            )),
        ),
        |(n1, n2, source)| InstanceCtxBuilder::Current(CurrentBuilder { n1, n2, source }),
    )
    .parse_complete(i)
}
