use core::fmt;
use std::fmt::Display;

use super::*;

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ctx)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Num(n) => write!(f, "{n}"),
            Value::Expr(expr) => write!(f, "'{expr}'"),
        }
    }
}

impl fmt::Display for KeyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.k, self.v)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KV(key_value) => write!(f, "{key_value}"),
            Self::Value(v) => write!(f, "{v}"),
        }
    }
}

impl fmt::Display for ModelType {
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

struct InlineDispaly<'a, T: Display>(&'a [T]);
impl<'a, T: Display> Display for InlineDispaly<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for t in self.0 {
            write!(f, " {t}")?;
        }
        Ok(())
    }
}

struct MultilineDispaly<'a, T: Display>(&'a [T]);
impl<'a, T: Display> Display for MultilineDispaly<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for t in self.0 {
            write!(f, "\n{t}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Instance {
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

impl fmt::Display for Model {
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

impl fmt::Display for Subckt {
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
impl fmt::Display for PnameColNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.pname, self.col_num)
    }
}
impl fmt::Display for DataFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "+ FILE='{}'{}",
            self.file,
            InlineDispaly(&self.pname_col_num)
        )
    }
}
impl fmt::Display for DataFiles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", MultilineDispaly(&self.files))?;
        if let Some(out) = &self.out {
            write!(f, "\n+ OUT={out}")
        } else {
            Ok(())
        }
    }
}
impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".DATA {}", self.name)?;
        match &self.values {
            DataValues::InlineExpr { params, values } => write!(
                f,
                "\n+{} DATAFORM{}",
                InlineDispaly(&params),
                WrapDispaly(&values, params.len())
            )?,
            DataValues::InlineNum { params, values } => write!(
                f,
                "\n+{}{}",
                InlineDispaly(&params),
                WrapDispaly(&values, params.len())
            )?,
            DataValues::MER(data_files) => write!(f, " MER{data_files}")?,
            DataValues::LAM(data_files) => write!(f, " LAM{data_files}")?,
        }
        write!(f, "\n.ENDDATA")
    }
}

impl fmt::Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for seg in &self.segments {
            match seg {
                Segment::Local(local_ast) => write!(f, "{local_ast}")?,
                Segment::Include(once_lock) => match once_lock.get().unwrap().as_ref() {
                    Ok(ast) => write!(f, "* {ast:?}")?,
                    Err(e) => write!(f, "* {e:?}")?,
                },
            }
        }
        Ok(())
    }
}

impl fmt::Display for Unknwon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".{}{}", self.cmd, InlineDispaly(&self.tokens))
    }
}
impl fmt::Display for General {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".{}{}", self.cmd, InlineDispaly(&self.tokens))
    }
}

impl fmt::Display for LocalAST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.option.is_empty() {
            write!(f, "\n.OPTION {}", WrapDispaly(&self.option, 4))?;
        }
        if !self.param.is_empty() {
            write!(f, "\n.PARAM {}", WrapDispaly(&self.param, 4))?;
        }
        write!(f, "{}", MultilineDispaly(&self.model))?;
        write!(f, "{}", MultilineDispaly(&self.subckt))?;
        write!(f, "{}", MultilineDispaly(&self.instance))?;
        write!(f, "{}", MultilineDispaly(&self.general))?;
        write!(f, "{}", MultilineDispaly(&self.data))?;
        write!(f, "{}", MultilineDispaly(&self.unknwon))?;
        Ok(())
    }
}
