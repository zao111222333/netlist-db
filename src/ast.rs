use std::sync::{Arc, OnceLock};

use crate::{
    err::ParseError,
    instance::InstanceBuilder,
    self_builder,
    span::{ParsedId, Span},
};
use netlist_macros::Builder;

#[derive(Debug, Clone, Copy, Builder)]
pub enum ValueBuilder {
    Num(f64),
    Expr(Span),
}

#[derive(Debug, Clone, Default, Copy, Builder)]
pub struct KeyValueBuilder {
    pub k: Span,
    pub v: ValueBuilder,
}

#[derive(Debug, Clone, Copy, Builder)]
pub enum TokenBuilder {
    KV(KeyValueBuilder),
    Value(ValueBuilder),
    V(Span),
    I(Span),
}

#[derive(Debug, Clone, Copy)]
pub enum GeneralCmd {
    /// `.ic` initial condition
    Ic,
    /// `.ic` initial condition
    Meas,
}
self_builder!(GeneralCmd);
#[derive(Debug, Clone, Builder)]
pub struct GeneralBuilder {
    pub cmd: GeneralCmd,
    pub tokens: Vec<TokenBuilder>,
}

#[derive(Debug, Clone, Builder)]
pub struct UnknwonBuilder {
    pub cmd: Span,
    pub tokens: Vec<TokenBuilder>,
}

#[derive(Debug, Clone, Builder)]
pub struct ModelBuilder {
    pub name: Span,
    pub model_type: ModelTypeBuilder,
    pub params: Vec<KeyValueBuilder>,
}

#[derive(Debug, Clone, Copy, Builder)]
pub enum ModelTypeBuilder {
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

#[derive(Debug, Clone, Builder)]
pub struct DataBuilder {
    pub name: Span,
    pub values: DataValuesBuilder,
}
#[derive(Debug, Clone, Builder)]
pub enum DataValuesBuilder {
    InlineExpr {
        params: Vec<Span>,
        values: Vec<ValueBuilder>,
    },
    InlineNum {
        params: Vec<Span>,
        values: Vec<f64>,
    },
    /// https://eda-cpu1.eias.junzhuo.site/~junzhuo/hspice/index.htm#page/hspice_14/data.htm
    /// Concatenated (series merging) data files to use.
    MER(DataFilesBuilder),
    /// Column-laminated (parallel merging) data files to use.
    LAM(DataFilesBuilder),
}
#[derive(Debug, Clone, Builder)]
pub struct DataFilesBuilder {
    pub files: Vec<DataFileBuilder>,
    pub out: Option<Span>,
}
#[derive(Debug, Clone, Builder)]
pub struct DataFileBuilder {
    pub file: Span,
    pub pname_col_num: Vec<PnameColNumBuilder>,
}

#[derive(Debug, Clone, Builder)]
pub struct PnameColNumBuilder {
    pub pname: Span,
    pub col_num: usize,
}

/// ``` spice
/// .subckt pulvt11ll_ckt d g s b w=1e-6 l=1e-6 sa='sar'
/// ...
/// .ends pulvt11ll_ckt
/// ```
/// Do NOT support `.include` / `.lib` in `.subckt`
#[derive(Debug)]
pub struct SubcktBuilder {
    pub name: Span,
    /// subckt/model name is the last arg
    pub ports: Vec<Span>,
    pub params: Vec<KeyValueBuilder>,
    pub ast: ASTBuilder,
}

/// The `.include` and `.lib file tt` will be directly evaluated
#[derive(Debug, Default)]
pub struct LocalAST {
    pub subckt: Vec<SubcktBuilder>,
    pub instance: Vec<InstanceBuilder>,
    pub model: Vec<ModelBuilder>,
    pub param: Vec<KeyValueBuilder>,
    pub option: Vec<(Span, Option<ValueBuilder>)>,
    pub data: Vec<DataBuilder>,
    pub general: Vec<GeneralBuilder>,
    pub init_condition: Vec<(Span, ValueBuilder, Option<Span>)>,
    pub nodeset: Vec<(Span, ValueBuilder, Option<Span>)>,
    pub unknwon: Vec<UnknwonBuilder>,
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
    Local(Box<LocalAST>),
    Include(Arc<OnceLock<Result<ParsedId, ParseError>>>),
}
#[derive(Debug, Default)]
pub struct ASTBuilder {
    pub segments: Vec<Segment>,
}

impl ASTBuilder {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }
}

impl Default for ValueBuilder {
    #[inline]
    fn default() -> Self {
        Self::Num(0.0)
    }
}

impl From<(&str, Span)> for ModelTypeBuilder {
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
