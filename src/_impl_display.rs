use core::fmt;
use std::fmt::Display;

use super::*;

pub struct FloatDisplay(pub f64);
impl fmt::Display for FloatDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.7e}", self.0)
    }
}

pub struct OptionDispaly<'a, T, F: Fn(&T, &mut fmt::Formatter<'_>) -> fmt::Result>(
    pub &'a Option<T>,
    pub F,
);
impl<T, F: Fn(&T, &mut fmt::Formatter<'_>) -> fmt::Result> Display for OptionDispaly<'_, T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(t) = self.0 {
            self.1(t, f)
        } else {
            Ok(())
        }
    }
}

pub fn display_wrap<
    W: fmt::Write,
    T,
    I: Iterator<Item = T>,
    SEP: fmt::Display,
    F: Fn(T, &mut W) -> fmt::Result,
>(
    f: &mut W,
    iter: I,
    fmt_one: F,
    line_sep: SEP,
    item_sep: char,
    wrap_size: usize,
) -> fmt::Result {
    use itertools::Itertools as _;
    for mut ts in iter.into_iter().chunks(wrap_size).into_iter() {
        write!(f, "\n{line_sep}")?;
        if let Some(first) = ts.next() {
            fmt_one(first, f)?;
            for t in ts {
                write!(f, "{}", item_sep)?;
                fmt_one(t, f)?;
            }
        }
    }
    Ok(())
}

pub fn display_inline<W: fmt::Write, T, I: Iterator<Item = T>, F: Fn(T, &mut W) -> fmt::Result>(
    f: &mut W,
    iter: I,
    fmt_one: F,
    sep: char,
) -> fmt::Result {
    let mut iter = iter.into_iter();
    if let Some(first) = iter.next() {
        fmt_one(first, f)?;
        for t in iter {
            write!(f, "{sep}")?;
            fmt_one(t, f)?;
        }
    }
    Ok(())
}

pub fn display_multiline<
    W: fmt::Write,
    T,
    I: Iterator<Item = T>,
    F: Fn(T, &mut W) -> fmt::Result,
>(
    f: &mut W,
    iter: I,
    fmt_one: F,
) -> fmt::Result {
    for t in iter.into_iter() {
        writeln!(f)?;
        fmt_one(t, f)?;
    }
    Ok(())
}

impl fmt::Display for ast::Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num(float) => write!(f, "{}", FloatDisplay(*float)),
            Self::Expr(expr) => write!(f, "'{expr}'"),
        }
    }
}
impl fmt::Display for ast::KeyValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.k, self.v)
    }
}

impl fmt::Display for ast::Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KV(key_value) => write!(f, "{key_value}"),
            Self::Value(v) => write!(f, "{v}"),
            Self::V(name) => write!(f, "V({name})"),
            Self::I(name) => write!(f, "I({name})"),
        }
    }
}

impl fmt::Display for ast::ModelType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AMP => write!(f, "AMP"),
            Self::C => write!(f, "C"),
            Self::CORE => write!(f, "CORE"),
            Self::D => write!(f, "D"),
            Self::L => write!(f, "L"),
            Self::NJF => write!(f, "NJF"),
            Self::NMOS => write!(f, "NMOS"),
            Self::NPN => write!(f, "NPN"),
            Self::OPT => write!(f, "OPT"),
            Self::PJF => write!(f, "PJF"),
            Self::PMOS => write!(f, "PMOS"),
            Self::PNP => write!(f, "PNP"),
            Self::R => write!(f, "R"),
            Self::U => write!(f, "U"),
            Self::W => write!(f, "W"),
            Self::S => write!(f, "S"),
            Self::Unknown(span) => write!(f, "{span}"),
        }
    }
}
impl fmt::Display for ast::DataFiles<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_multiline(f, self.files.iter(), |file, f| {
            write!(f, "+ FILE='{}' ", file.file)?;
            display_inline(
                f,
                file.pname_col_num.iter(),
                |pname_col_num, f| write!(f, "{}={}", pname_col_num.pname, pname_col_num.col_num),
                ' ',
            )
        })?;
        write!(
            f,
            "{}",
            OptionDispaly(&self.out, |out, f| write!(f, "\n+ OUT='{out}'")),
        )
    }
}

impl fmt::Display for ast::Data<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".DATA {}", self.name)?;
        match &self.values {
            ast::DataValues::InlineExpr { params, values } => {
                write!(f, "\n+ ",)?;
                display_inline(f, params.iter(), Display::fmt, ' ')?;
                write!(f, " DATAFORM")?;
                display_wrap(f, values.iter(), Display::fmt, "+ ", ' ', params.len())?;
            }
            ast::DataValues::InlineNum { params, values } => {
                write!(f, "\n+ ",)?;
                display_inline(f, params.iter(), Display::fmt, ' ')?;
                display_wrap(
                    f,
                    values.iter(),
                    |float: &f64, f: &mut fmt::Formatter<'_>| write!(f, "{}", FloatDisplay(*float)),
                    "+ ",
                    ' ',
                    params.len(),
                )?;
            }
            ast::DataValues::MER(data_files) => write!(f, " MER{data_files}")?,
            ast::DataValues::LAM(data_files) => write!(f, " LAM{data_files}")?,
        }
        write!(f, "\n.ENDDATA")
    }
}

impl fmt::Display for instance::Instance<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.ctx)
    }
}

impl fmt::Display for instance::InstanceCtx<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            instance::InstanceCtx::Resistor(resistor) => write!(f, "{resistor}"),
            instance::InstanceCtx::Capacitor(capacitor) => write!(f, "{capacitor}"),
            instance::InstanceCtx::Inductor(inductor) => write!(f, "{inductor}"),
            instance::InstanceCtx::Voltage(voltage) => write!(f, "{voltage}"),
            instance::InstanceCtx::Current(current) => write!(f, "{current}"),
            instance::InstanceCtx::MOSFET(mosfet) => write!(f, "{mosfet}"),
            instance::InstanceCtx::BJT(bjt) => write!(f, "{bjt}"),
            instance::InstanceCtx::Diode(diode) => write!(f, "{diode}"),
            instance::InstanceCtx::Subckt(subckt) => write!(f, "{subckt}"),
            instance::InstanceCtx::Unknown {
                r#type: _,
                ports,
                params,
            } => {
                display_inline(f, ports.iter(), Display::fmt, ' ')?;
                write!(f, " ")?;
                display_inline(f, params.iter(), Display::fmt, ' ')
            }
        }
    }
}

impl fmt::Display for instance::Subckt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_inline(f, self.ports.iter(), Display::fmt, ' ')?;
        write!(f, " {} ", self.cktname,)?;
        display_inline(f, self.params.iter(), Display::fmt, ' ')
    }
}

impl fmt::Display for instance::Voltage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.n1, self.n2, self.source,)
    }
}

impl fmt::Display for instance::Current<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.n1, self.n2, self.source,)
    }
}

impl fmt::Display for instance::VoltageSource<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            instance::VoltageSource::Params(params) => {
                write!(f, " ")?;
                display_inline(f, params.iter(), Display::fmt, ' ')
            }
            instance::VoltageSource::Value(value) => write!(f, "{value}"),
            instance::VoltageSource::PWL(pwl) => write!(f, "{pwl}"),
        }
    }
}

impl fmt::Display for instance::CurrentSource<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            instance::CurrentSource::Params(params) => {
                write!(f, " ")?;
                display_inline(f, params.iter(), Display::fmt, ' ')
            }
            instance::CurrentSource::Value(value) => write!(f, "{value}"),
            instance::CurrentSource::PWL(pwl) => write!(f, "{pwl}"),
        }
    }
}

impl fmt::Display for instance::TimeValuePoint<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.time, self.value,)
    }
}

impl fmt::Display for instance::PWL<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PWL(",)?;
        display_wrap(f, self.points.iter(), Display::fmt, "+ ", ' ', 1)?;
        write!(f, ")",)
    }
}

impl fmt::Display for instance::Resistor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.n1, self.n2, self.value,)
    }
}
impl fmt::Display for instance::Capacitor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.n1, self.n2, self.value,)
    }
}
impl fmt::Display for instance::Inductor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.n1, self.n2, self.value,)
    }
}

impl fmt::Display for instance::MOSFET<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}{} {} ",
            self.nd,
            self.ng,
            self.ns,
            OptionDispaly(&self.nb, |nb, f| write!(f, " {nb}")),
            self.mname,
        )?;
        display_inline(f, self.params.iter(), Display::fmt, ' ')
    }
}

impl fmt::Display for instance::BJT<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}{} {} ",
            self.nc,
            self.nb,
            self.ne,
            OptionDispaly(&self.ns, |ns, f| write!(f, " {ns}")),
            self.mname,
        )?;
        display_inline(f, self.params.iter(), Display::fmt, ' ')
    }
}

impl fmt::Display for instance::Diode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {} ", self.nplus, self.nminus, self.mname,)?;
        display_inline(f, self.params.iter(), Display::fmt, ' ')
    }
}

impl fmt::Display for ast::Model<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".MODEL {} {}", self.name, self.model_type,)?;
        display_wrap(f, self.params.iter(), Display::fmt, "+ ", ' ', 4)
    }
}

impl fmt::Display for Subckt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".SUBCKT {} ", self.name,)?;
        display_inline(f, self.ports.iter(), Display::fmt, ' ')?;
        write!(f, " ")?;
        display_inline(f, self.params.iter(), Display::fmt, ' ')?;
        write!(f, "{}\n.ENDS {}", self.ast, self.name)
    }
}

impl fmt::Display for ast::Unknwon<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".{} ", self.cmd)?;
        display_inline(f, self.tokens.iter(), Display::fmt, ' ')
    }
}
impl fmt::Display for ast::General<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".{} ", self.cmd)?;
        display_inline(f, self.tokens.iter(), Display::fmt, ' ')
    }
}

impl fmt::Display for AST<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.option.is_empty() {
            write!(f, ".OPTION ",)?;
            display_wrap(
                f,
                self.option.iter(),
                |option: &(Cow<'_, str>, Option<ast::Value<'_>>), f: &mut fmt::Formatter<'_>| {
                    if let Some(v) = &option.1 {
                        write!(f, "{}={v}", option.0)
                    } else {
                        write!(f, "{}", option.0)
                    }
                },
                "+ ",
                ' ',
                4,
            )?;
        }
        if !self.param.is_empty() {
            write!(f, "\n.PARAM ",)?;
            display_wrap(f, self.param.iter(), Display::fmt, "+ ", ' ', 4)?;
        }
        display_multiline(f, self.model.iter(), Display::fmt)?;
        display_multiline(f, self.subckt.iter(), Display::fmt)?;
        display_multiline(f, self.instance.iter(), Display::fmt)?;
        display_multiline(
            f,
            self.init_condition.iter(),
            |ic: &(Cow<'_, str>, ast::Value<'_>, Option<Cow<'_, str>>),
             f: &mut fmt::Formatter<'_>| {
                write!(f, ".IC V({})={}", ic.0, ic.1)?;
                if let Some(subckt) = &ic.2 {
                    write!(f, " suckt={subckt}")
                } else {
                    Ok(())
                }
            },
        )?;
        display_multiline(
            f,
            self.nodeset.iter(),
            |ic: &(Cow<'_, str>, ast::Value<'_>, Option<Cow<'_, str>>),
             f: &mut fmt::Formatter<'_>| {
                write!(f, ".NODESET {}={}", ic.0, ic.1)?;
                if let Some(subckt) = &ic.2 {
                    write!(f, " suckt={subckt}")
                } else {
                    Ok(())
                }
            },
        )?;
        display_multiline(f, self.data.iter(), Display::fmt)?;
        display_multiline(f, self.general.iter(), Display::fmt)?;
        display_multiline(f, self.unknwon.iter(), Display::fmt)?;
        Ok(())
    }
}

impl fmt::Display for ast::GeneralCmd {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
