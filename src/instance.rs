use crate::{
    ast::{KeyValueBuilder, ValueBuilder},
    self_builder,
    span::Span,
};
use netlist_macros::Builder;

/// ``` spice
/// XX1 net48 D VDD VNW PHVT11LL_CKT W=0.22u L=40.00n
/// ```
#[derive(Debug, Clone, Builder)]
pub struct InstanceBuilder {
    pub name: Span,
    /// subckt/model name is the last arg
    /// (fisrt, rest)
    pub ctx: InstanceCtxBuilder,
}

#[derive(Debug, Clone, Builder)]
pub enum InstanceCtxBuilder {
    Resistor(ResistorBuilder),
    Capacitor(CapacitorBuilder),
    Inductor(InductorBuilder),
    Voltage(VoltageBuilder),
    Current(CurrentBuilder),
    MOSFET(MOSFETBuilder),
    BJT(BJTBuilder),
    Diode(DiodeBuilder),
    Subckt(SubcktBuilder),
    Unknown {
        r#type: u8,
        ports: Vec<Span>,
        params: Vec<KeyValueBuilder>,
    },
}
#[derive(Debug, Clone, Builder)]
pub struct ResistorBuilder {
    pub n1: Span,
    pub n2: Span,
    pub value: ValueBuilder,
}
#[derive(Debug, Clone, Builder)]
pub struct CapacitorBuilder {
    pub n1: Span,
    pub n2: Span,
    pub value: ValueBuilder,
}
#[derive(Debug, Clone, Builder)]
pub struct InductorBuilder {
    pub n1: Span,
    pub n2: Span,
    pub value: ValueBuilder,
}
#[derive(Debug, Clone, Builder)]
pub struct SubcktBuilder {
    pub ports: Vec<Span>,
    pub cktname: Span,
    pub params: Vec<KeyValueBuilder>,
}
/// https://eda-cpu1.eias.junzhuo.site/~junzhuo/hspice/index.htm#page/hspice_11/bipolar_junction_transistor_bjt_element.htm
#[derive(Debug, Clone, Builder)]
pub struct BJTBuilder {
    pub nc: Span,
    pub nb: Span,
    pub ne: Span,
    pub ns: Option<Span>,
    pub mname: Span,
    pub params: Vec<KeyValueBuilder>,
}
#[derive(Debug, Clone, Builder)]
pub struct MOSFETBuilder {
    pub nd: Span,
    pub ng: Span,
    pub ns: Span,
    pub nb: Option<Span>,
    pub mname: Span,
    pub params: Vec<KeyValueBuilder>,
}
/// https://eda-cpu1.eias.junzhuo.site/~junzhuo/hspice/index.htm#page/hspice_11/diode_element.htm
#[derive(Debug, Clone, Builder)]
pub struct DiodeBuilder {
    pub nplus: Span,
    pub nminus: Span,
    pub mname: Span,
    pub params: Vec<KeyValueBuilder>,
}
#[derive(Debug, Clone, Builder)]
pub struct VoltageBuilder {
    pub n1: Span,
    pub n2: Span,
    pub source: VoltageSourceBuilder,
}
#[derive(Debug, Clone, Builder)]
pub enum VoltageSourceBuilder {
    Params(Vec<KeyValueBuilder>),
    Value(ValueBuilder),
    PWL(PWLBuilder),
}
#[derive(Debug, Clone, Builder)]
pub struct CurrentBuilder {
    pub n1: Span,
    pub n2: Span,
    pub source: CurrentSourceBuilder,
}
#[derive(Debug, Clone, Builder)]
pub enum CurrentSourceBuilder {
    Params(Vec<KeyValueBuilder>),
    Value(ValueBuilder),
    PWL(PWLBuilder),
}
#[derive(Debug, Clone, Builder)]
pub struct TimeValuePointBuilder {
    pub time: ValueBuilder,
    pub value: ValueBuilder,
}

/// https://eda-cpu1.eias.junzhuo.site/~junzhuo/hspice/index.htm#page/hspice_11/pwl_source.htm
#[derive(Debug, Clone, Default, Builder)]
pub struct PWLBuilder {
    pub points: Vec<TimeValuePointBuilder>,
    /// Keyword and time value to specify a repeating function.
    /// With no argument, the source repeats from the beginning of the function.
    /// repeat is the time, in units of seconds, which specifies
    /// the startpoint of the waveform to repeat. This time needs
    /// to be less than the greatest time point, tn.
    pub repeat: Option<ValueBuilder>,
    /// Specifies the stop time for the repeat.
    pub rstop: Option<ValueBuilder>,
    /// Specifies the value of the current/voltage source at the time of rstop.
    /// stopvalue can be either a real number or Z for high impedance state.
    pub stopvalue: Option<ValueBuilder>,
    /// stopeslope is the switching time from the last PWL value to the stopvalue.
    /// Default value is 30ps, if unspecified.
    pub stopslope: Option<ValueBuilder>,
    /// `TD=delay`
    ///
    /// Time, in units of seconds, which specifies the length of time to delay (propagation delay) the piecewise linear function.
    pub delay: Option<ValueBuilder>,
    pub edgetype: EdgeType,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum EdgeType {
    #[default]
    Linear,
    HalfSine,
}

self_builder!(EdgeType);
