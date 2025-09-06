use core::fmt;
use std::{borrow::Borrow as _, fmt::Display};

use super::*;

#[derive(Debug, Clone, Copy)]
pub struct FloatDisplay(pub f64);
impl fmt::Display for FloatDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.7e}", self.0)
    }
}

pub struct TableFloatDisplay(pub f64);
impl fmt::Display for TableFloatDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_sign_positive() {
            write!(f, " {: <13.7e}", self.0)
        } else {
            write!(f, "{: <14.7e}", self.0)
        }
    }
}

pub struct OptionDispaly<'a, T, F: Fn(&T, &mut fmt::Formatter<'_>) -> fmt::Result>(
    pub &'a Option<T>,
    pub F,
);
impl<T, F: Fn(&T, &mut fmt::Formatter<'_>) -> fmt::Result> Display for OptionDispaly<'_, T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
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
    F: FnMut(T, &mut W) -> fmt::Result,
>(
    f: &mut W,
    iter: I,
    mut fmt_one: F,
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
                write!(f, "{item_sep}")?;
                fmt_one(t, f)?;
            }
        }
    }
    Ok(())
}

pub fn display_inline<
    W: fmt::Write,
    T,
    I: Iterator<Item = T>,
    F: FnMut(T, &mut W) -> fmt::Result,
>(
    f: &mut W,
    iter: I,
    mut fmt_one: F,
    sep: char,
    skip_first_sep: bool,
) -> fmt::Result {
    let mut iter = iter.into_iter();
    if skip_first_sep {
        if let Some(first) = iter.next() {
            fmt_one(first, f)?;
        }
    }
    for t in iter {
        write!(f, "{sep}")?;
        fmt_one(t, f)?;
    }
    Ok(())
}

pub fn display_multiline<
    W: fmt::Write,
    T,
    I: Iterator<Item = T>,
    F: FnMut(T, &mut W) -> fmt::Result,
>(
    f: &mut W,
    iter: I,
    mut fmt_one: F,
) -> fmt::Result {
    for t in iter.into_iter() {
        writeln!(f)?;
        fmt_one(t, f)?;
    }
    Ok(())
}

impl fmt::Display for AST<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        DefaultFmt.ast(self, f)
    }
}
impl fmt::Display for Subckt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        DefaultFmt.subckt(self, f)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DefaultFmt;
impl SpiceFmt for DefaultFmt {}

pub trait SpiceFmt: Sized {
    fn value<W: fmt::Write>(&self, t: &ast::Value<'_>, f: &mut W) -> fmt::Result {
        match t {
            ast::Value::Num(float) => write!(f, "{}", FloatDisplay(*float)),
            ast::Value::Expr(expr) => write!(f, "'{expr}'"),
        }
    }
    fn key_value<W: fmt::Write>(&self, t: &ast::KeyValue<'_>, f: &mut W) -> fmt::Result {
        write!(f, "{}=", t.k)?;
        self.value(&t.v, f)
    }
    fn token<W: fmt::Write>(&self, t: &ast::Token<'_>, f: &mut W) -> fmt::Result {
        match t {
            ast::Token::KV(t) => self.key_value(t, f),
            ast::Token::Value(t) => self.value(t, f),
            ast::Token::V(name) => write!(f, "V({name})"),
            ast::Token::I(name) => write!(f, "I({name})"),
        }
    }
    fn data_files<W: fmt::Write>(&self, t: &ast::DataFiles<'_>, f: &mut W) -> fmt::Result {
        display_multiline(f, t.files.iter(), |file, f| {
            write!(f, "+ FILE='{}'", file.file)?;
            display_inline(
                f,
                file.pname_col_num.iter(),
                |pname_col_num, f| write!(f, "{}={}", pname_col_num.pname, pname_col_num.col_num),
                ' ',
                false,
            )
        })?;
        write!(
            f,
            "{}",
            OptionDispaly(&t.out, |out, f| write!(f, "\n+ OUT='{out}'")),
        )
    }
    fn data<W: fmt::Write>(&self, t: &ast::Data<'_>, f: &mut W) -> fmt::Result {
        write!(f, ".DATA {}", t.name)?;
        match &t.values {
            ast::DataValues::InlineExpr { params, values } => {
                write!(f, "\n+",)?;
                display_inline(f, params.iter(), |t, f| write!(f, "{t}"), ' ', false)?;
                write!(f, " DATAFORM")?;
                display_wrap(
                    f,
                    values.iter(),
                    |t, f| self.value(t, f),
                    "+ ",
                    ' ',
                    params.len(),
                )?;
            }
            ast::DataValues::InlineNum { params, values } => {
                write!(f, "\n+",)?;
                display_inline(f, params.iter(), |t, f| write!(f, "{t}"), ' ', false)?;
                display_wrap(
                    f,
                    values.iter(),
                    |float, f| write!(f, "{}", FloatDisplay(*float)),
                    "+ ",
                    ' ',
                    params.len(),
                )?;
            }
            ast::DataValues::MER(t) => {
                write!(f, " MER")?;
                self.data_files(t, f)?;
            }
            ast::DataValues::LAM(t) => {
                write!(f, " LAM")?;
                self.data_files(t, f)?;
            }
        }
        write!(f, "\n.ENDDATA")
    }
    fn instance<W: fmt::Write>(&self, t: &instance::Instance<'_>, f: &mut W) -> fmt::Result {
        write!(f, "{} ", t.name)?;
        self.instance_ctx(&t.ctx, f)
    }
    fn instance_ctx<W: fmt::Write>(&self, t: &instance::InstanceCtx<'_>, f: &mut W) -> fmt::Result {
        match t {
            instance::InstanceCtx::Resistor(t) => self.resistor(t, f),
            instance::InstanceCtx::Capacitor(t) => self.capacitor(t, f),
            instance::InstanceCtx::Inductor(t) => self.inductor(t, f),
            instance::InstanceCtx::Voltage(t) => self.voltage(t, f),
            instance::InstanceCtx::Current(t) => self.current(t, f),
            instance::InstanceCtx::MOSFET(t) => self.mosfet(t, f),
            instance::InstanceCtx::BJT(t) => self.bjt(t, f),
            instance::InstanceCtx::Diode(t) => self.diode(t, f),
            instance::InstanceCtx::Subckt(t) => self.inst_subckt(t, f),
            instance::InstanceCtx::Unknown {
                r#type: _,
                nodes,
                params,
            } => {
                display_inline(f, nodes.iter(), |t, f| write!(f, "{t}"), ' ', true)?;
                display_inline(f, params.iter(), |t, f| self.key_value(t, f), ' ', false)
            }
        }
    }
    fn inst_subckt<W: fmt::Write>(&self, t: &instance::Subckt<'_>, f: &mut W) -> fmt::Result {
        self.ports(t.nodes.iter().map(|s| s.borrow()), f)?;
        write!(f, " {}", t.cktname)?;
        display_inline(f, t.params.iter(), |t, f| self.key_value(t, f), ' ', false)
    }
    fn voltage<W: fmt::Write>(&self, t: &instance::Voltage<'_>, f: &mut W) -> fmt::Result {
        write!(f, "{} {} ", t.n1, t.n2)?;
        self.voltage_source(&t.source, f)
    }
    fn current<W: fmt::Write>(&self, t: &instance::Current<'_>, f: &mut W) -> fmt::Result {
        write!(f, "{} {} ", t.n1, t.n2)?;
        self.current_source(&t.source, f)
    }
    fn voltage_source<W: fmt::Write>(
        &self,
        t: &instance::VoltageSource<'_>,
        f: &mut W,
    ) -> fmt::Result {
        match t {
            instance::VoltageSource::Params(params) => {
                display_inline(f, params.iter(), |t, f| self.key_value(t, f), ' ', false)
            }
            instance::VoltageSource::Value(t) => self.value(t, f),
            instance::VoltageSource::PWL(t) => self.pwl(t, f),
        }
    }
    fn current_source<W: fmt::Write>(
        &self,
        t: &instance::CurrentSource<'_>,
        f: &mut W,
    ) -> fmt::Result {
        match t {
            instance::CurrentSource::Params(params) => {
                display_inline(f, params.iter(), |t, f| self.key_value(t, f), ' ', false)
            }
            instance::CurrentSource::Value(t) => self.value(t, f),
            instance::CurrentSource::PWL(t) => self.pwl(t, f),
        }
    }
    fn time_value_point<W: fmt::Write>(
        &self,
        t: &instance::TimeValuePoint<'_>,
        f: &mut W,
    ) -> fmt::Result {
        self.value(&t.time, f)?;
        write!(f, " ")?;
        self.value(&t.value, f)
    }
    fn pwl<W: fmt::Write>(&self, t: &instance::PWL<'_>, f: &mut W) -> fmt::Result {
        write!(f, "PWL(",)?;
        display_wrap(
            f,
            t.points.iter(),
            |t, f| self.time_value_point(t, f),
            "+ ",
            ' ',
            1,
        )?;
        write!(f, ")",)
    }
    fn resistor<W: fmt::Write>(&self, t: &instance::Resistor<'_>, f: &mut W) -> fmt::Result {
        write!(f, "{} {} ", t.n1, t.n2)?;
        self.value(&t.value, f)
    }
    fn capacitor<W: fmt::Write>(&self, t: &instance::Capacitor<'_>, f: &mut W) -> fmt::Result {
        write!(f, "{} {} ", t.n1, t.n2)?;
        self.value(&t.value, f)
    }
    fn inductor<W: fmt::Write>(&self, t: &instance::Inductor<'_>, f: &mut W) -> fmt::Result {
        write!(f, "{} {} ", t.n1, t.n2)?;
        self.value(&t.value, f)
    }
    fn mosfet<W: fmt::Write>(&self, t: &instance::MOSFET<'_>, f: &mut W) -> fmt::Result {
        write!(
            f,
            "{} {} {}{} {}",
            t.nd,
            t.ng,
            t.ns,
            OptionDispaly(&t.nb, |nb, f| write!(f, " {nb}")),
            t.mname,
        )?;
        display_inline(f, t.params.iter(), |t, f| self.key_value(t, f), ' ', false)
    }
    fn bjt<W: fmt::Write>(&self, t: &instance::BJT<'_>, f: &mut W) -> fmt::Result {
        write!(
            f,
            "{} {} {}{} {}",
            t.nc,
            t.nb,
            t.ne,
            OptionDispaly(&t.ns, |ns, f| write!(f, " {ns}")),
            t.mname,
        )?;
        display_inline(f, t.params.iter(), |t, f| self.key_value(t, f), ' ', false)
    }
    fn diode<W: fmt::Write>(&self, t: &instance::Diode<'_>, f: &mut W) -> fmt::Result {
        write!(f, "{} {} {}", t.nplus, t.nminus, t.mname,)?;
        display_inline(f, t.params.iter(), |t, f| self.key_value(t, f), ' ', false)
    }
    fn model_type<W: fmt::Write>(&self, t: &ast::ModelType<'_>, f: &mut W) -> fmt::Result {
        match t {
            ast::ModelType::AMP => write!(f, "AMP"),
            ast::ModelType::C => write!(f, "C"),
            ast::ModelType::CORE => write!(f, "CORE"),
            ast::ModelType::D => write!(f, "D"),
            ast::ModelType::L => write!(f, "L"),
            ast::ModelType::NJF => write!(f, "NJF"),
            ast::ModelType::NMOS => write!(f, "NMOS"),
            ast::ModelType::NPN => write!(f, "NPN"),
            ast::ModelType::OPT => write!(f, "OPT"),
            ast::ModelType::PJF => write!(f, "PJF"),
            ast::ModelType::PMOS => write!(f, "PMOS"),
            ast::ModelType::PNP => write!(f, "PNP"),
            ast::ModelType::R => write!(f, "R"),
            ast::ModelType::U => write!(f, "U"),
            ast::ModelType::W => write!(f, "W"),
            ast::ModelType::S => write!(f, "S"),
            ast::ModelType::Unknown(span) => write!(f, "{span}"),
        }
    }
    fn model<W: fmt::Write>(&self, t: &ast::Model<'_>, f: &mut W) -> fmt::Result {
        write!(f, ".MODEL {} ", t.name)?;
        self.model_type(&t.model_type, f)?;
        display_wrap(
            f,
            t.params.iter(),
            |t, f| self.key_value(t, f),
            "+ ",
            ' ',
            4,
        )
    }
    fn ports<'a, W: fmt::Write, I: Iterator<Item = &'a str>>(
        &self,
        t: I,
        f: &mut W,
    ) -> fmt::Result {
        display_inline(f, t, |t, f| write!(f, "{t}"), ' ', true)
    }
    fn subckt<W: fmt::Write>(&self, t: &Subckt<'_>, f: &mut W) -> fmt::Result {
        write!(f, ".SUBCKT {} ", t.name)?;
        self.ports(t.ports.iter().map(|s| s.borrow()), f)?;
        display_inline(f, t.params.iter(), |t, f| self.key_value(t, f), ' ', false)?;
        self.ast(&t.ast, f)?;
        write!(f, "\n.ENDS {}", t.name)
    }
    fn unknwon<W: fmt::Write>(&self, t: &ast::Unknwon<'_>, f: &mut W) -> fmt::Result {
        write!(f, ".{}", t.cmd)?;
        display_inline(f, t.tokens.iter(), |t, f| self.token(t, f), ' ', false)
    }
    fn general_cmd<W: fmt::Write>(&self, t: &ast::GeneralCmd, f: &mut W) -> fmt::Result {
        _ = (t, f);
        todo!()
    }
    fn general<W: fmt::Write>(&self, t: &ast::General<'_>, f: &mut W) -> fmt::Result {
        write!(f, ".")?;
        self.general_cmd(&t.cmd, f)?;
        display_inline(f, t.tokens.iter(), |t, f| self.token(t, f), ' ', false)
    }
    fn ast<W: fmt::Write>(&self, t: &AST<'_>, f: &mut W) -> fmt::Result {
        if !t.option.is_empty() {
            write!(f, ".OPTION ",)?;
            display_wrap(
                f,
                t.option.iter(),
                |option, f| {
                    if let Some(v) = &option.1 {
                        write!(f, "{}=", option.0)?;
                        self.value(v, f)
                    } else {
                        write!(f, "{}", option.0)
                    }
                },
                "+ ",
                ' ',
                4,
            )?;
        }
        if !t.param.is_empty() {
            write!(f, "\n.PARAM ",)?;
            display_wrap(f, t.param.iter(), |t, f| self.key_value(t, f), "+ ", ' ', 4)?;
        }
        display_multiline(f, t.model.iter(), |t, f| self.model(t, f))?;
        display_multiline(f, t.subckt.values(), |t, f| self.subckt(t, f))?;
        display_multiline(f, t.instance.iter(), |t, f| self.instance(t, f))?;
        display_multiline(f, t.init_condition.iter(), |ic, f| {
            write!(f, ".IC V({})=", ic.0)?;
            self.value(&ic.1, f)?;
            if let Some(subckt) = &ic.2 {
                write!(f, " suckt={subckt}")
            } else {
                Ok(())
            }
        })?;
        display_multiline(f, t.nodeset.iter(), |ic, f| {
            write!(f, ".NODESET {}=", ic.0)?;
            self.value(&ic.1, f)?;
            if let Some(subckt) = &ic.2 {
                write!(f, " suckt={subckt}")
            } else {
                Ok(())
            }
        })?;
        display_multiline(f, t.data.iter(), |t, f| self.data(t, f))?;
        display_multiline(f, t.general.iter(), |t, f| self.general(t, f))?;
        display_multiline(f, t.unknwon.iter(), |t, f| self.unknwon(t, f))?;
        Ok(())
    }
}
