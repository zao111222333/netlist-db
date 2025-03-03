use alloc::borrow::Cow;

use super::{KeyValue, Value};

/// ``` spice
/// XX1 net48 D VDD VNW PHVT11LL_CKT W=0.22u L=40.00n
/// ```
#[derive(Debug, Clone)]
pub struct Instance<'s> {
    pub name: Cow<'s, str>,
    /// subckt/model name is the last arg
    /// (fisrt, rest)
    pub ctx: InstanceCtx<'s>,
}

#[derive(Debug, Clone)]
pub enum InstanceCtx<'s> {
    Resistor(Resistor<'s>),
    Capacitor(Capacitor<'s>),
    Inductor(Inductor<'s>),
    Voltage(Voltage<'s>),
    Current(Current<'s>),
    MOSFET(MOSFET<'s>),
    BJT(BJT<'s>),
    Diode(Diode<'s>),
    Subckt(Subckt<'s>),
    Unknown {
        r#type: u8,
        ports: Vec<Cow<'s, str>>,
        params: Vec<KeyValue<'s>>,
    },
}
#[derive(Debug, Clone)]
pub struct Resistor<'s> {
    pub n1: Cow<'s, str>,
    pub n2: Cow<'s, str>,
    pub value: Value<'s>,
}
#[derive(Debug, Clone)]
pub struct Capacitor<'s> {
    pub n1: Cow<'s, str>,
    pub n2: Cow<'s, str>,
    pub value: Value<'s>,
}
#[derive(Debug, Clone)]
pub struct Inductor<'s> {
    pub n1: Cow<'s, str>,
    pub n2: Cow<'s, str>,
    pub value: Value<'s>,
}
#[derive(Debug, Clone)]
pub struct Subckt<'s> {
    pub ports: Vec<Cow<'s, str>>,
    pub cktname: Cow<'s, str>,
    pub params: Vec<KeyValue<'s>>,
}
/// https://eda-cpu1.eias.junzhuo.site/~junzhuo/hspice/index.htm#page/hspice_11/bipolar_junction_transistor_bjt_element.htm
#[derive(Debug, Clone)]
pub struct BJT<'s> {
    pub nc: Cow<'s, str>,
    pub nb: Cow<'s, str>,
    pub ne: Cow<'s, str>,
    pub ns: Option<Cow<'s, str>>,
    pub mname: Cow<'s, str>,
    pub params: Vec<KeyValue<'s>>,
}
#[derive(Debug, Clone)]
pub struct MOSFET<'s> {
    pub nd: Cow<'s, str>,
    pub ng: Cow<'s, str>,
    pub ns: Cow<'s, str>,
    pub nb: Option<Cow<'s, str>>,
    pub mname: Cow<'s, str>,
    pub params: Vec<KeyValue<'s>>,
}
/// https://eda-cpu1.eias.junzhuo.site/~junzhuo/hspice/index.htm#page/hspice_11/diode_element.htm
#[derive(Debug, Clone)]
pub struct Diode<'s> {
    pub nplus: Cow<'s, str>,
    pub nminus: Cow<'s, str>,
    pub mname: Cow<'s, str>,
    pub params: Vec<KeyValue<'s>>,
}
#[derive(Debug, Clone)]
pub struct Voltage<'s> {
    pub n1: Cow<'s, str>,
    pub n2: Cow<'s, str>,
    pub source: VoltageSource<'s>,
}
#[derive(Debug, Clone)]
pub enum VoltageSource<'s> {
    Params(Vec<KeyValue<'s>>),
    Value(Value<'s>),
    PWL(PWL<'s>),
}
#[derive(Debug, Clone)]
pub struct Current<'s> {
    pub n1: Cow<'s, str>,
    pub n2: Cow<'s, str>,
    pub source: CurrentSource<'s>,
}
#[derive(Debug, Clone)]
pub enum CurrentSource<'s> {
    Params(Vec<KeyValue<'s>>),
    Value(Value<'s>),
    PWL(PWL<'s>),
}
#[derive(Debug, Clone)]
pub struct TimeValuePoint<'s> {
    pub time: Value<'s>,
    pub value: Value<'s>,
}
/// https://eda-cpu1.eias.junzhuo.site/~junzhuo/hspice/index.htm#page/hspice_11/pwl_source.htm
#[derive(Debug, Clone, Default)]
pub struct PWL<'s> {
    pub points: Vec<TimeValuePoint<'s>>,
    /// Keyword and time value to specify a repeating function.
    /// With no argument, the source repeats from the beginning of the function.
    /// repeat is the time, in units of seconds, which specifies
    /// the startpoint of the waveform to repeat. This time needs
    /// to be less than the greatest time point, tn.
    pub repeat: Option<Value<'s>>,
    /// Specifies the stop time for the repeat.
    pub rstop: Option<Value<'s>>,
    /// Specifies the value of the current/voltage source at the time of rstop.
    /// stopvalue can be either a real number or Z for high impedance state.
    pub stopvalue: Option<Value<'s>>,
    /// stopeslope is the switching time from the last PWL value to the stopvalue.
    /// Default value is 30ps, if unspecified.
    pub stopslope: Option<Value<'s>>,
    /// `TD=delay`
    ///
    /// Time, in units of seconds, which specifies the length of time to delay (propagation delay) the piecewise linear function.
    pub delay: Option<Value<'s>>,
    pub edgetype: EdgeType,
}
#[derive(Debug, Clone, Copy, Default)]
pub enum EdgeType {
    #[default]
    Linear,
    HalfSine,
}
