use core::fmt;
use std::fmt::Display;

use crate::builder::*;

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num(float) => write!(f, "{float}"),
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
                WrapDispaly(values, params.len())
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

struct MultilineDispaly<'a, T: Display>(&'a [T]);
impl<T: Display> Display for MultilineDispaly<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for t in self.0 {
            write!(f, "\n{t}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Instance<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.name,
            InlineDispaly(&self.ports),
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
            write!(f, ".OPTION {}", WrapDispaly(&self.option, 4))?;
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
