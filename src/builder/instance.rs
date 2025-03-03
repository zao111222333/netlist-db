use super::{KeyValue, Span, Value};

/// ``` spice
/// XX1 net48 D VDD VNW PHVT11LL_CKT W=0.22u L=40.00n
/// ```
#[derive(Debug, Clone)]
pub struct Instance {
    pub name: Span,
    /// subckt/model name is the last arg
    /// (fisrt, rest)
    pub ctx: InstanceCtx,
}

#[derive(Debug, Clone)]
pub enum InstanceCtx {
    Resistor(Resistor),
    Capacitor(Capacitor),
    Inductor(Inductor),
    Voltage(Voltage),
    Current(Current),
    MOSFET(MOSFET),
    BJT(BJT),
    Diode(Diode),
    Subckt(Subckt),
    Unknown {
        r#type: u8,
        ports: Vec<Span>,
        params: Vec<KeyValue>,
    },
}
#[derive(Debug, Clone)]
pub struct Resistor {
    pub n1: Span,
    pub n2: Span,
    pub value: Value,
}
#[derive(Debug, Clone)]
pub struct Capacitor {
    pub n1: Span,
    pub n2: Span,
    pub value: Value,
}
#[derive(Debug, Clone)]
pub struct Inductor {
    pub n1: Span,
    pub n2: Span,
    pub value: Value,
}
#[derive(Debug, Clone)]
pub struct Subckt {
    pub ports: Vec<Span>,
    pub cktname: Span,
    pub params: Vec<KeyValue>,
}
/// https://eda-cpu1.eias.junzhuo.site/~junzhuo/hspice/index.htm#page/hspice_11/bipolar_junction_transistor_bjt_element.htm
#[derive(Debug, Clone)]
pub struct BJT {
    pub nc: Span,
    pub nb: Span,
    pub ne: Span,
    pub ns: Option<Span>,
    pub mname: Span,
    pub params: Vec<KeyValue>,
}
#[derive(Debug, Clone)]
pub struct MOSFET {
    pub nd: Span,
    pub ng: Span,
    pub ns: Span,
    pub nb: Option<Span>,
    pub mname: Span,
    pub params: Vec<KeyValue>,
}
/// https://eda-cpu1.eias.junzhuo.site/~junzhuo/hspice/index.htm#page/hspice_11/diode_element.htm
#[derive(Debug, Clone)]
pub struct Diode {
    pub nplus: Span,
    pub nminus: Span,
    pub mname: Span,
    pub params: Vec<KeyValue>,
}
#[derive(Debug, Clone)]
pub struct Voltage {
    pub n1: Span,
    pub n2: Span,
    pub source: VoltageSource,
}
#[derive(Debug, Clone)]
pub enum VoltageSource {
    Params(Vec<KeyValue>),
    Value(Value),
    PWL(PWL),
}
#[derive(Debug, Clone)]
pub struct Current {
    pub n1: Span,
    pub n2: Span,
    pub source: CurrentSource,
}
#[derive(Debug, Clone)]
pub enum CurrentSource {
    Params(Vec<KeyValue>),
    Value(Value),
    PWL(PWL),
}

#[derive(Debug, Clone)]
pub struct TimeValuePoint {
    pub time: Value,
    pub value: Value,
}

/// https://eda-cpu1.eias.junzhuo.site/~junzhuo/hspice/index.htm#page/hspice_11/pwl_source.htm
#[derive(Debug, Clone, Default)]
pub struct PWL {
    pub points: Vec<TimeValuePoint>,
    /// Keyword and time value to specify a repeating function.
    /// With no argument, the source repeats from the beginning of the function.
    /// repeat is the time, in units of seconds, which specifies
    /// the startpoint of the waveform to repeat. This time needs
    /// to be less than the greatest time point, tn.
    pub repeat: Option<Value>,
    /// Specifies the stop time for the repeat.
    pub rstop: Option<Value>,
    /// Specifies the value of the current/voltage source at the time of rstop.
    /// stopvalue can be either a real number or Z for high impedance state.
    pub stopvalue: Option<Value>,
    /// stopeslope is the switching time from the last PWL value to the stopvalue.
    /// Default value is 30ps, if unspecified.
    pub stopslope: Option<Value>,
    /// `TD=delay`
    ///
    /// Time, in units of seconds, which specifies the length of time to delay (propagation delay) the piecewise linear function.
    pub delay: Option<Value>,
    pub edgetype: super::super::instance::EdgeType,
}
