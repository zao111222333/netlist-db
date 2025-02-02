use nom::{combinator::map, sequence::tuple, IResult};

use crate::{file::{LocatedSpan, Span}, lexer::{Instance, KeyValue}};

use super::utils::{hierarchical_node_char, ports_params};
// https://eda-cpu1.eias.junzhuo.site/~junzhuo/hspice/index.htm#page/hspice_12/resistor_device_model_equations.htm
#[inline]
fn instance(mut i: LocatedSpan) -> IResult<LocatedSpan, Instance> {
    let first_char;
    let name;
    (i, (first_char, name)) = hierarchical_node_char(i)?;
    match first_char.to_ascii_lowercase() {
        b'r' => Self::Resistor,
        b'c' => Self::Capacitor,
        b'v' => Self::VoltageSource,
        b'i' => Self::CurrentSource,
        b'm' => Self::MOSFET,
        b'q' => Self::BJT,
        b'd' => Self::Diode,
        b'x' => Self::Subckt,
        _ => Self::Unknown(value),
    }
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

#[derive(Debug, Clone)]
pub struct Resistor {
    pub n1: Span,
    pub n2: Span,
    /// subckt/model name is the last arg
    pub ports: Vec<Span>,
    /// (fisrt, rest)
    pub params: Vec<KeyValue>,
}

pub enum ResistorProperty {
    Val(f64),
    Val(f64),
}