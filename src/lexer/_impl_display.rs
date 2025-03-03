use core::fmt;
use std::fmt::Display;

use super::*;

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num(float) => write!(f, "{}", FloatDisplay(float)),
            Self::Expr(expr) => write!(f, "'{expr}'"),
        }
    }
}

struct FloatDisplay<'a>(&'a f64);
impl fmt::Display for FloatDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.7e}", self.0)
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

struct WrapDispaly<'a, T: Display>(&'a [T], usize);
impl<T: Display> Display for WrapDispaly<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ts in self.0.chunks(self.1) {
            write!(f, "\n+")?;
            for t in ts {
                write!(f, " {t}")?;
            }
        }
        Ok(())
    }
}
struct WrapOptionDispaly<'a, 's>(&'a [(Cow<'s, str>, Option<Value<'s>>)], usize);
impl Display for WrapOptionDispaly<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ts in self.0.chunks(self.1) {
            write!(f, "\n+")?;
            for (k, v) in ts {
                if let Some(v) = v {
                    write!(f, " {k}={v}")?;
                } else {
                    write!(f, " {k}")?;
                }
            }
        }
        Ok(())
    }
}
struct WrapFloatDispaly<'a>(&'a [f64], usize);
impl Display for WrapFloatDispaly<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ts in self.0.chunks(self.1) {
            write!(f, "\n+")?;
            for t in ts {
                write!(f, " {}", FloatDisplay(t))?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for Data<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".DATA {}", self.name)?;
        match &self.values {
            DataValues::InlineExpr { params, values } => write!(
                f,
                "\n+{} DATAFORM{}",
                InlineDispaly(params),
                WrapDispaly(values, params.len())
            )?,
            DataValues::InlineNum { params, values } => write!(
                f,
                "\n+{}{}",
                InlineDispaly(params),
                WrapFloatDispaly(values, params.len())
            )?,
            DataValues::MER() => todo!(),
            DataValues::LAM() => todo!(),
        }
        write!(f, "\n.ENDDATA")
    }
}

struct InlineDispaly<'a, T: Display>(&'a [T]);
impl<T: Display> Display for InlineDispaly<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for t in self.0 {
            write!(f, " {t}")?;
        }
        Ok(())
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
struct MultilineDispaly<'a, T: Display>(&'a [T]);
impl<T: Display> Display for MultilineDispaly<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for t in self.0 {
            write!(f, "\n{t}")?;
        }
        Ok(())
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
            } => write!(f, "{}{}", InlineDispaly(ports), InlineDispaly(params)),
        }
    }
}

impl fmt::Display for instance::Subckt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}{}",
            InlineDispaly(&self.ports),
            self.cktname,
            InlineDispaly(&self.params)
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
            instance::VoltageSource::Params(params) => write!(f, "{}", InlineDispaly(params)),
            instance::VoltageSource::Value(value) => write!(f, "{value}"),
            instance::VoltageSource::PWL(pwl) => write!(f, "{pwl}"),
        }
    }
}

impl fmt::Display for instance::CurrentSource<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            instance::CurrentSource::Params(params) => write!(f, "{}", InlineDispaly(params)),
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
        write!(f, "PWL({})", WrapDispaly(&self.points, 1),)
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
            "{} {} {} {} {}{}",
            self.nd,
            self.ng,
            self.ns,
            OptionDispaly(&self.nb),
            self.mname,
            InlineDispaly(&self.params)
        )
    }
}

impl fmt::Display for instance::BJT<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}{}",
            self.nc,
            self.nb,
            self.ne,
            OptionDispaly(&self.ns),
            self.mname,
            InlineDispaly(&self.params)
        )
    }
}

impl fmt::Display for instance::Diode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}{}",
            self.nplus,
            self.nminus,
            self.mname,
            InlineDispaly(&self.params)
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
            WrapDispaly(&self.params, 4)
        )
    }
}

impl fmt::Display for Subckt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ".SUBCKT {}{}{}",
            self.name,
            InlineDispaly(&self.ports),
            InlineDispaly(&self.params)
        )?;
        write!(f, "{}", self.ast)?;
        write!(f, "\n.ENDS {}", self.name)
    }
}

impl fmt::Display for Unknwon<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".{}{}", self.cmd, InlineDispaly(&self.tokens))
    }
}
impl fmt::Display for General<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".{}{}", self.cmd, InlineDispaly(&self.tokens))
    }
}

impl fmt::Display for AST<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.option.is_empty() {
            write!(f, ".OPTION {}", WrapOptionDispaly(&self.option, 4))?;
        }
        if !self.param.is_empty() {
            write!(f, "\n.PARAM {}", WrapDispaly(&self.param, 4))?;
        }
        write!(f, "{}", MultilineDispaly(&self.model))?;
        write!(f, "{}", MultilineDispaly(&self.subckt))?;
        write!(f, "{}", MultilineDispaly(&self.instance))?;
        write!(f, "{}", MultilineDispaly(&self.data))?;
        write!(f, "{}", MultilineDispaly(&self.general))?;
        write!(f, "{}", MultilineDispaly(&self.unknwon))?;
        Ok(())
    }
}
