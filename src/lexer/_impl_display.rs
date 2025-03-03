use core::fmt;
use std::fmt::Display;

use super::*;

struct WrapDispaly<'a, T, F: Fn(&T, &mut fmt::Formatter<'_>) -> fmt::Result, SEP: Display>(
    &'a [T],
    F,
    /// line sep
    SEP,
    /// item sep
    char,
    usize,
);
impl<T, F: Fn(&T, &mut fmt::Formatter<'_>) -> fmt::Result, SEP: Display> Display
    for WrapDispaly<'_, T, F, SEP>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ts in self.0.chunks(self.4) {
            write!(f, "\n{}", self.2)?;
            let mut iter = ts.iter();
            if let Some(first) = iter.next() {
                self.1(first, f)?;
                for t in iter {
                    write!(f, "{}", self.3)?;
                    self.1(t, f)?;
                }
            }
        }
        Ok(())
    }
}
struct InlineDispaly<'a, T, F: Fn(&T, &mut fmt::Formatter<'_>) -> fmt::Result>(&'a [T], F, char);
impl<T, F: Fn(&T, &mut fmt::Formatter<'_>) -> fmt::Result> Display for InlineDispaly<'_, T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.0.iter();
        if let Some(first) = iter.next() {
            self.1(first, f)?;
            for t in iter {
                write!(f, "{}", self.2)?;
                self.1(t, f)?;
            }
        }
        Ok(())
    }
}
struct FloatDisplay<'a>(&'a f64);
impl fmt::Display for FloatDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.7e}", self.0)
    }
}

struct OptionDispaly<'a, T: Display>(&'a Option<T>);
impl<T: Display> Display for OptionDispaly<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(t) = self.0 {
            write!(f, " {t}")
        } else {
            Ok(())
        }
    }
}
struct MultilineDispaly<'a, T, F: Fn(&T, &mut fmt::Formatter<'_>) -> fmt::Result>(&'a [T], F);
impl<T, F: Fn(&T, &mut fmt::Formatter<'_>) -> fmt::Result> Display for MultilineDispaly<'_, T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for t in self.0 {
            write!(f, "\n")?;
            self.1(t, f)?;
        }
        Ok(())
    }
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num(float) => write!(f, "{}", FloatDisplay(float)),
            Self::Expr(expr) => write!(f, "'{expr}'"),
        }
    }
}
impl fmt::Display for KeyValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.k, self.v)
    }
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KV(key_value) => write!(f, "{key_value}"),
            Self::Value(v) => write!(f, "{v}"),
            Self::V(name) => write!(f, "V({name})"),
            Self::I(name) => write!(f, "I({name})"),
        }
    }
}

impl fmt::Display for ModelType<'_> {
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

impl fmt::Display for Data<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".DATA {}", self.name)?;
        match &self.values {
            DataValues::InlineExpr { params, values } => write!(
                f,
                "\n+ {} DATAFORM{}",
                InlineDispaly(params, Display::fmt, ' '),
                WrapDispaly(values, Display::fmt, "+ ", ' ', params.len())
            )?,
            DataValues::InlineNum { params, values } => write!(
                f,
                "\n+ {}{}",
                InlineDispaly(params, Display::fmt, ' '),
                WrapDispaly(
                    values,
                    |float: &f64, f: &mut fmt::Formatter<'_>| write!(f, "{}", FloatDisplay(float)),
                    "+ ",
                    ' ',
                    params.len()
                )
            )?,
            DataValues::MER() => todo!(),
            DataValues::LAM() => todo!(),
        }
        write!(f, "\n.ENDDATA")
    }
}
impl fmt::Display for DataValuesCsv<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            DataValues::InlineExpr { params, values } => write!(
                f,
                "{}{}",
                InlineDispaly(params, Display::fmt, ','),
                WrapDispaly(values, Display::fmt, "", ',', params.len())
            ),
            DataValues::InlineNum { params, values } => write!(
                f,
                "{}{}",
                InlineDispaly(params, Display::fmt, ','),
                WrapDispaly(
                    values,
                    |float: &f64, f: &mut fmt::Formatter<'_>| write!(f, "{}", FloatDisplay(float)),
                    "",
                    ',',
                    params.len()
                )
            ),
            DataValues::MER() => todo!(),
            DataValues::LAM() => todo!(),
        }
    }
}

impl fmt::Display for instance::Instance<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.name, self.ctx,)
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
            } => write!(
                f,
                " {} {}",
                InlineDispaly(ports, Display::fmt, ' '),
                InlineDispaly(params, Display::fmt, ' ')
            ),
        }
    }
}

impl fmt::Display for instance::Subckt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            " {} {} {}",
            InlineDispaly(&self.ports, Display::fmt, ' '),
            self.cktname,
            InlineDispaly(&self.params, Display::fmt, ' ')
        )
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
                write!(f, " {}", InlineDispaly(params, Display::fmt, ' '))
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
                write!(f, " {}", InlineDispaly(params, Display::fmt, ' '))
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
        write!(
            f,
            "PWL({})",
            WrapDispaly(&self.points, Display::fmt, "+ ", ' ', 1),
        )
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
            "{} {} {} {} {} {}",
            self.nd,
            self.ng,
            self.ns,
            OptionDispaly(&self.nb),
            self.mname,
            InlineDispaly(&self.params, Display::fmt, ' ')
        )
    }
}

impl fmt::Display for instance::BJT<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {}",
            self.nc,
            self.nb,
            self.ne,
            OptionDispaly(&self.ns),
            self.mname,
            InlineDispaly(&self.params, Display::fmt, ' ')
        )
    }
}

impl fmt::Display for instance::Diode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.nplus,
            self.nminus,
            self.mname,
            InlineDispaly(&self.params, Display::fmt, ' ')
        )
    }
}

impl fmt::Display for Model<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ".MODEL {} {}{}",
            self.name,
            self.model_type,
            WrapDispaly(&self.params, Display::fmt, "+ ", ' ', 4)
        )
    }
}

impl fmt::Display for Subckt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ".SUBCKT {} {} {}",
            self.name,
            InlineDispaly(&self.ports, Display::fmt, ' '),
            InlineDispaly(&self.params, Display::fmt, ' ')
        )?;
        write!(f, "{}", self.ast)?;
        write!(f, "\n.ENDS {}", self.name)
    }
}

impl fmt::Display for Unknwon<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ".{} {}",
            self.cmd,
            InlineDispaly(&self.tokens, Display::fmt, ' ')
        )
    }
}
impl fmt::Display for General<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ".{} {}",
            self.cmd,
            InlineDispaly(&self.tokens, Display::fmt, ' ')
        )
    }
}

impl fmt::Display for AST<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.option.is_empty() {
            write!(
                f,
                ".OPTION {}",
                WrapDispaly(
                    &self.option,
                    |option: &(Cow<'_, str>, Option<Value<'_>>), f: &mut fmt::Formatter<'_>| {
                        if let Some(v) = &option.1 {
                            write!(f, "{}={v}", option.0)
                        } else {
                            write!(f, "{}", option.0)
                        }
                    },
                    "+ ",
                    ' ',
                    4
                )
            )?;
        }
        if !self.param.is_empty() {
            write!(
                f,
                "\n.PARAM {}",
                WrapDispaly(&self.param, Display::fmt, "+ ", ' ', 4)
            )?;
        }
        write!(f, "{}", MultilineDispaly(&self.model, Display::fmt))?;
        write!(f, "{}", MultilineDispaly(&self.subckt, Display::fmt))?;
        write!(f, "{}", MultilineDispaly(&self.instance, Display::fmt))?;
        write!(
            f,
            "{}",
            MultilineDispaly(
                &self.init_condition,
                |ic: &(Cow<'_, str>, Value<'_>, Option<Cow<'_, str>>),
                 f: &mut fmt::Formatter<'_>| {
                    write!(f, ".IC V({})={}", ic.0, ic.1)?;
                    if let Some(subckt) = &ic.2 {
                        write!(f, " suckt={subckt}")
                    } else {
                        Ok(())
                    }
                },
            )
        )?;
        write!(f, "{}", MultilineDispaly(&self.data, Display::fmt))?;
        write!(f, "{}", MultilineDispaly(&self.general, Display::fmt))?;
        write!(f, "{}", MultilineDispaly(&self.unknwon, Display::fmt))?;
        Ok(())
    }
}
