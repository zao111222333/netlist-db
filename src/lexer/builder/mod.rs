mod _impl;
pub mod instance;
pub mod span;
use super::ParseError;
pub use _impl::Builder;
use core::fmt;
use span::{ParsedId, Span};
use std::sync::{Arc, OnceLock};

#[derive(Debug, Clone, Default, Copy)]
pub struct KeyValue {
    pub k: Span,
    pub v: Value,
}
#[derive(Debug, Clone, Copy)]
pub enum Token {
    KV(KeyValue),
    Value(Value),
    V(Span),
    I(Span),
}

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Num(f64),
    Expr(Span),
}

impl Default for Value {
    #[inline]
    fn default() -> Self {
        Self::Num(0.0)
    }
}

/// ``` spice
/// .subckt pulvt11ll_ckt d g s b w=1e-6 l=1e-6 sa='sar'
/// ...
/// .ends pulvt11ll_ckt
/// ```
/// Do NOT support `.include` / `.lib` in `.subckt`
#[derive(Debug)]
pub struct Subckt {
    pub name: Span,
    /// subckt/model name is the last arg
    pub ports: Vec<Span>,
    pub params: Vec<KeyValue>,
    pub ast: AST,
}

#[derive(Debug, Clone)]
pub struct Data {
    pub name: Span,
    pub values: DataValues,
}
#[derive(Debug, Clone)]
pub enum DataValues {
    InlineExpr {
        params: Vec<Span>,
        values: Vec<Value>,
    },
    InlineNum {
        params: Vec<Span>,
        values: Vec<f64>,
    },
    /// https://eda-cpu1.eias.junzhuo.site/~junzhuo/hspice/index.htm#page/hspice_14/data.htm
    /// Concatenated (series merging) data files to use.
    MER(DataFiles),
    /// Column-laminated (parallel merging) data files to use.
    LAM(DataFiles),
}

#[derive(Debug, Clone)]
pub struct DataFile {
    pub file: Span,
    pub pname_col_num: Vec<PnameColNum>,
}

#[derive(Debug, Clone)]
pub struct PnameColNum {
    pub pname: Span,
    pub col_num: usize,
}
#[derive(Debug, Clone)]
pub struct DataFiles {
    pub files: Vec<DataFile>,
    pub out: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct General {
    pub cmd: GeneralCmd,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct Unknwon {
    pub cmd: Span,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub name: Span,
    pub model_type: ModelType,
    pub params: Vec<KeyValue>,
}

/// The `.include` and `.lib file tt` will be directly evaluated
#[derive(Debug, Default)]
pub struct LocalAST {
    pub subckt: Vec<Subckt>,
    pub instance: Vec<instance::Instance>,
    pub model: Vec<Model>,
    pub param: Vec<KeyValue>,
    pub option: Vec<(Span, Option<Value>)>,
    pub data: Vec<Data>,
    pub general: Vec<General>,
    pub init_condition: Vec<(Span, Value, Option<Span>)>,
    pub unknwon: Vec<Unknwon>,
    pub errors: Vec<ParseError>,
}

impl LocalAST {
    pub fn is_empty(&self) -> bool {
        self.subckt.is_empty()
            && self.instance.is_empty()
            && self.model.is_empty()
            && self.param.is_empty()
            && self.option.is_empty()
            && self.data.is_empty()
            && self.general.is_empty()
            && self.unknwon.is_empty()
            && self.errors.is_empty()
    }
}

#[derive(Debug)]
pub enum Segment {
    Local(LocalAST),
    Include(Arc<OnceLock<Result<ParsedId, ParseError>>>),
}
#[derive(Debug, Default)]
pub struct AST {
    pub segments: Vec<Segment>,
}

impl AST {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(not(test), derive(Copy))]
pub enum ModelType {
    /// operational amplifier model
    AMP,
    /// capacitor model
    C,
    /// magnetic core model
    CORE,
    /// diode model
    D,
    /// inductor model or magnetic core mutual inductor model
    L,
    /// n-channel JFET model
    NJF,
    /// n-channel MOSFET model
    NMOS,
    /// npn BJT model
    NPN,
    /// optimization model
    OPT,
    /// p-channel JFET model
    PJF,
    /// p-channel MOSFET model
    PMOS,
    /// pnp BJT model
    PNP,
    /// resistor model
    R,
    /// lossy transmission line model (lumped)
    U,
    /// lossy transmission line model
    W,
    /// S-parameter
    S,
    Unknown(Span),
}
impl From<(&str, Span)> for ModelType {
    #[inline]
    fn from(value: (&str, Span)) -> Self {
        let (_str, _type) = value;
        match _str.to_uppercase().as_str() {
            "AMP" => Self::AMP,
            "C" => Self::C,
            "CORE" => Self::CORE,
            "D" => Self::D,
            "L" => Self::L,
            "NJF" => Self::NJF,
            "NMOS" => Self::NMOS,
            "NPN" => Self::NPN,
            "OPT" => Self::OPT,
            "PJF" => Self::PJF,
            "PMOS" => Self::PMOS,
            "PNP" => Self::PNP,
            "R" => Self::R,
            "U" => Self::U,
            "W" => Self::W,
            "S" => Self::S,
            _ => Self::Unknown(_type),
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum GeneralCmd {
    /// `.ic` initial condition
    Ic,
    /// `.ic` initial condition
    Meas,
}
impl fmt::Display for GeneralCmd {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
