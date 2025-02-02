#[cfg(test)]
mod _test_impl;
pub mod parser;
use anstyle::Style;
use indexmap::IndexMap;
use nom::error::ErrorKind;

use crate::file::{FileId, LocatedSpan, ParsedId, Pos, Span};
use core::fmt;
use std::{
    cell::LazyCell,
    path::PathBuf,
    sync::{Arc, OnceLock},
};

#[derive(Debug, Clone, Default)]
#[cfg_attr(not(test), derive(Copy))]
pub struct KeyValue {
    pub k: Span,
    pub v: Value,
}
#[derive(Debug, Clone)]
#[cfg_attr(not(test), derive(Copy))]
pub enum Token {
    KV(KeyValue),
    Value(Value),
}

#[derive(Debug, Clone)]
#[cfg_attr(not(test), derive(Copy))]
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

/// ``` spice
/// XX1 net48 D VDD VNW PHVT11LL_CKT W=0.22u L=40.00n
/// ```
#[derive(Debug, Clone)]
pub struct Instance {
    pub name: Span,
    pub instance_type: InstanceType,
    /// subckt/model name is the last arg
    pub ports: Vec<Span>,
    /// (fisrt, rest)
    pub params: Vec<KeyValue>,
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
    pub instance: Vec<Instance>,
    pub model: Vec<Model>,
    pub param: Vec<KeyValue>,
    pub option: Vec<Token>,
    pub data: Vec<Data>,
    pub general: Vec<General>,
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

impl From<nom::Err<nom::error::Error<LocatedSpan<'_>>>> for ParseError {
    #[inline]
    fn from(e: nom::Err<nom::error::Error<LocatedSpan<'_>>>) -> Self {
        match e {
            nom::Err::Incomplete(_) => ParseErrorInner::Nom(None).with(None),
            nom::Err::Failure(e) | nom::Err::Error(e) => {
                ParseErrorInner::Nom(Some(e.code)).record(e.input)
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseErrorInner {
    #[error("Incomplete")]
    IO(#[from] std::io::Error),
    #[error("Can NOT find section [{section}] in file {path}")]
    NoLibSection { path: PathBuf, section: String },
    /// Nom Error
    #[error("Syntax error")]
    Nom(Option<ErrorKind>),
    /// something else
    #[error("{0:?}")]
    Unknown(Span),
    #[error("Circular definition")]
    CircularDefinition(IndexMap<FileId, Option<Pos>>, usize),
}

impl ParseErrorInner {
    pub fn record(self, i: LocatedSpan) -> ParseError {
        ParseError {
            pos: Pos::new(i),
            err: self,
        }
    }
    pub fn with(self, pos: Option<Pos>) -> ParseError {
        ParseError { pos, err: self }
    }
}
#[derive(Debug, Clone, Copy)]
struct Styles {
    msg: Style,
    typ: Style,
    err: Style,
}
const STYLES: LazyCell<Styles> = LazyCell::new(|| {
    use anstyle::{AnsiColor, Color};
    use std::io::IsTerminal;
    if std::io::stdout().is_terminal() {
        Styles {
            msg: Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightMagenta))),
            typ: Style::new()
                .fg_color(Some(Color::Ansi(AnsiColor::BrightMagenta)))
                .bold(),
            err: Style::new()
                .fg_color(Some(AnsiColor::BrightRed.into()))
                .bold(),
        }
    } else {
        Styles {
            msg: Style::new(),
            typ: Style::new(),
            err: Style::new(),
        }
    }
});

#[derive(Debug)]
pub struct ParseError {
    pub pos: Option<Pos>,
    pub err: ParseErrorInner,
}

impl ParseError {
    pub fn report(&self, has_err: &mut bool, file_id: &FileId, file: &str) {
        *has_err = true;
        struct ReportDisplay<'a> {
            err: &'a ParseError,
            file_id: &'a FileId,
            file: &'a str,
        }
        impl fmt::Display for ReportDisplay<'_> {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                use crate::builder::Builder;
                let styles: Styles = *STYLES;
                write!(
                    f,
                    "\nFile {}\"{}\"{}",
                    styles.msg.render(),
                    self.file_id.path().display(),
                    styles.msg.render_reset()
                )?;
                if let Some(pos) = self.err.pos {
                    write!(
                        f,
                        ", line {}{}{}",
                        styles.msg.render(),
                        pos.line_num,
                        styles.msg.render_reset()
                    )?;
                    let span = unsafe {
                        LocatedSpan::new_from_raw_offset(
                            pos.start,
                            pos.line_num,
                            &self.file[pos.start..],
                            (),
                        )
                    };
                    if let Ok(s) = core::str::from_utf8(span.get_line_beginning()) {
                        write!(f, "\n{s}\n")?;
                        for _ in 0..span.get_column() - 1 {
                            write!(f, " ")?;
                        }
                        write!(f, "{}<-{}", styles.err.render(), styles.err.render_reset())?;
                    }
                }
                writeln!(f)?;
                match &self.err.err {
                    ParseErrorInner::IO(error) => {
                        writeln!(
                            f,
                            "{}Error{}: {}{error}{}",
                            styles.typ.render(),
                            styles.typ.render_reset(),
                            styles.msg.render(),
                            styles.msg.render_reset()
                        )
                    }
                    ParseErrorInner::NoLibSection { path, section } => {
                        writeln!(
                            f,
                            "{}Error{}: {}Can NOT find section `{section}` in file \"{}\"{}",
                            styles.typ.render(),
                            styles.typ.render_reset(),
                            styles.msg.render(),
                            path.display(),
                            styles.msg.render_reset()
                        )
                    }
                    ParseErrorInner::Nom(e) => {
                        write!(
                            f,
                            "{}ParserError{}",
                            styles.typ.render(),
                            styles.typ.render_reset(),
                        )?;
                        if let Some(e) = e {
                            writeln!(
                                f,
                                ": {}{e:?}{}",
                                styles.msg.render(),
                                styles.msg.render_reset()
                            )
                        } else {
                            writeln!(f)
                        }
                    }
                    ParseErrorInner::Unknown(span) => {
                        writeln!(
                            f,
                            "{}SyntaxError{}: {}Unknwon command `{}`{}",
                            styles.typ.render(),
                            styles.typ.render_reset(),
                            styles.msg.render(),
                            span.build(self.file),
                            styles.msg.render_reset()
                        )
                    }
                    ParseErrorInner::CircularDefinition(index_set, idx) => {
                        struct FileDisplay<'a>(&'a FileId, &'a Option<Pos>);
                        impl fmt::Display for FileDisplay<'_> {
                            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                                match self.0 {
                                    FileId::Include { path } => {
                                        write!(f, "File \"{}\"", path.display())?;
                                        if let Some(pos) = self.1 {
                                            write!(f, ", line {}", pos.line_num)?;
                                        }
                                        Ok(())
                                    }
                                    FileId::Section { path, section } => {
                                        write!(f, "File \"{}\"", path.display())?;
                                        if let Some(pos) = self.1 {
                                            write!(f, ", line {}", pos.line_num)?;
                                        }
                                        write!(f, ", section {section}")
                                    }
                                }
                            }
                        }
                        impl<'s> FileDisplay<'s> {
                            fn new(f: (&'s FileId, &'s Option<Pos>)) -> Self {
                                Self(f.0, f.1)
                            }
                        }
                        let circular_file = index_set.get_index(*idx).unwrap();
                        writeln!(
                            f,
                            "{}CircularDefinition{}: {}Detect circular definition in {}{}",
                            styles.typ.render(),
                            styles.typ.render_reset(),
                            styles.msg.render(),
                            FileDisplay::new(circular_file),
                            styles.msg.render_reset()
                        )?;
                        for (i, file) in index_set.iter().enumerate() {
                            if *idx == i {
                                writeln!(
                                    f,
                                    "{} * {}{}\n     ↓",
                                    styles.err.render(),
                                    FileDisplay::new(file),
                                    styles.err.render_reset()
                                )?;
                            } else {
                                writeln!(f, "   {}\n     ↓", FileDisplay::new(file))?;
                            }
                        }
                        writeln!(
                            f,
                            "{} * {}{}",
                            styles.err.render(),
                            FileDisplay::new(circular_file),
                            styles.err.render_reset()
                        )
                    }
                }
            }
        }
        log::error!(
            "{}",
            ReportDisplay {
                err: self,
                file_id,
                file
            }
        )
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
    fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }
}

impl From<u8> for InstanceType {
    fn from(value: u8) -> Self {
        match value.to_ascii_lowercase() {
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
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InstanceType {
    /// `R`
    Resistor,
    /// `C`
    Capacitor,
    /// `V`
    VoltageSource,
    /// `I`
    CurrentSource,
    /// `M`
    MOSFET,
    /// `Q`
    BJT,
    /// `D`
    Diode,
    /// `X`
    Subckt,
    /// char
    Unknown(u8),
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
