use super::Span;
use crate::lexer;
use alloc::borrow::Cow;
pub trait Builder<'s> {
    type Out: 's;
    fn build(&self, file: &'s str) -> Self::Out;
}

impl<'s> Builder<'s> for f64 {
    type Out = f64;
    #[inline]
    fn build(&self, _file: &'s str) -> Self::Out {
        *self
    }
}

impl<'s> Builder<'s> for Span {
    type Out = Cow<'s, str>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        Cow::Borrowed(&file[self])
    }
}

impl<'s, T: Builder<'s>> Builder<'s> for Vec<T> {
    type Out = Vec<T::Out>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        self.iter().map(|s| s.build(file)).collect()
    }
}

impl<'s, T: Builder<'s>> Builder<'s> for Option<T> {
    type Out = Option<T::Out>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        self.as_ref().map(|s| s.build(file))
    }
}
impl<'s, T1: Builder<'s>, T2: Builder<'s>> Builder<'s> for (T1, T2) {
    type Out = (T1::Out, T2::Out);
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        (self.0.build(file), self.1.build(file))
    }
}

impl<'s> Builder<'s> for super::Value {
    type Out = lexer::Value<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        match self {
            super::Value::Num(float) => lexer::Value::Num(float.build(file)),
            super::Value::Expr(expr) => lexer::Value::Expr(expr.build(file)),
        }
    }
}

impl<'s> Builder<'s> for super::KeyValue {
    type Out = lexer::KeyValue<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        lexer::KeyValue {
            k: self.k.build(file),
            v: self.v.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::Token {
    type Out = lexer::Token<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        match self {
            super::Token::KV(key_value) => lexer::Token::KV(key_value.build(file)),
            super::Token::Value(v) => lexer::Token::Value(v.build(file)),
        }
    }
}

impl<'s> Builder<'s> for super::instance::Instance {
    type Out = lexer::instance::Instance<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        lexer::instance::Instance {
            name: self.name.build(file),
            ctx: self.ctx.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::InstanceCtx {
    type Out = lexer::instance::InstanceCtx<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        match self {
            super::instance::InstanceCtx::Resistor(resistor) => {
                lexer::instance::InstanceCtx::Resistor(resistor.build(file))
            }
            super::instance::InstanceCtx::Capacitor(capacitor) => {
                lexer::instance::InstanceCtx::Capacitor(capacitor.build(file))
            }
            super::instance::InstanceCtx::Inductor(inductor) => {
                lexer::instance::InstanceCtx::Inductor(inductor.build(file))
            }
            super::instance::InstanceCtx::Voltage(voltage) => {
                lexer::instance::InstanceCtx::Voltage(voltage.build(file))
            }
            super::instance::InstanceCtx::Current(current) => {
                lexer::instance::InstanceCtx::Current(current.build(file))
            }
            super::instance::InstanceCtx::MOSFET(mosfet) => {
                lexer::instance::InstanceCtx::MOSFET(mosfet.build(file))
            }
            super::instance::InstanceCtx::BJT(bjt) => {
                lexer::instance::InstanceCtx::BJT(bjt.build(file))
            }
            super::instance::InstanceCtx::Diode(diode) => {
                lexer::instance::InstanceCtx::Diode(diode.build(file))
            }
            super::instance::InstanceCtx::Subckt(subckt) => {
                lexer::instance::InstanceCtx::Subckt(subckt.build(file))
            }
            super::instance::InstanceCtx::Unknown {
                r#type,
                ports,
                params,
            } => lexer::instance::InstanceCtx::Unknown {
                r#type: *r#type,
                ports: ports.build(file),
                params: params.build(file),
            },
        }
    }
}

impl<'s> Builder<'s> for super::instance::Resistor {
    type Out = lexer::instance::Resistor<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        lexer::instance::Resistor {
            n1: self.n1.build(file),
            n2: self.n2.build(file),
            value: self.value.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::Capacitor {
    type Out = lexer::instance::Capacitor<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        lexer::instance::Capacitor {
            n1: self.n1.build(file),
            n2: self.n2.build(file),
            value: self.value.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::Voltage {
    type Out = lexer::instance::Voltage<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        lexer::instance::Voltage {
            n1: self.n1.build(file),
            n2: self.n2.build(file),
            source: self.source.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::Current {
    type Out = lexer::instance::Current<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        lexer::instance::Current {
            n1: self.n1.build(file),
            n2: self.n2.build(file),
            source: self.source.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::VoltageSource {
    type Out = lexer::instance::VoltageSource<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        match self {
            super::instance::VoltageSource::Params(params) => {
                lexer::instance::VoltageSource::Params(params.build(file))
            }
            super::instance::VoltageSource::Value(value) => {
                lexer::instance::VoltageSource::Value(value.build(file))
            }
            super::instance::VoltageSource::PWL(pwl) => {
                lexer::instance::VoltageSource::PWL(pwl.build(file))
            }
        }
    }
}

impl<'s> Builder<'s> for super::instance::CurrentSource {
    type Out = lexer::instance::CurrentSource<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        match self {
            super::instance::CurrentSource::Params(params) => {
                lexer::instance::CurrentSource::Params(params.build(file))
            }
            super::instance::CurrentSource::Value(value) => {
                lexer::instance::CurrentSource::Value(value.build(file))
            }
            super::instance::CurrentSource::PWL(pwl) => {
                lexer::instance::CurrentSource::PWL(pwl.build(file))
            }
        }
    }
}

impl<'s> Builder<'s> for super::instance::TimeValuePoint {
    type Out = lexer::instance::TimeValuePoint<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        lexer::instance::TimeValuePoint {
            time: self.time.build(file),
            value: self.value.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::PWL {
    type Out = lexer::instance::PWL<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        lexer::instance::PWL {
            points: self.points.build(file),
            repeat: self.repeat.build(file),
            rstop: self.rstop.build(file),
            stopvalue: self.stopvalue.build(file),
            stopslope: self.stopslope.build(file),
            delay: self.delay.build(file),
            edgetype: self.edgetype,
        }
    }
}

impl<'s> Builder<'s> for super::instance::Inductor {
    type Out = lexer::instance::Inductor<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        lexer::instance::Inductor {
            n1: self.n1.build(file),
            n2: self.n2.build(file),
            value: self.value.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::MOSFET {
    type Out = lexer::instance::MOSFET<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        lexer::instance::MOSFET {
            nd: self.nd.build(file),
            ng: self.ng.build(file),
            ns: self.ns.build(file),
            nb: self.nb.build(file),
            mname: self.mname.build(file),
            params: self.params.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::BJT {
    type Out = lexer::instance::BJT<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        lexer::instance::BJT {
            nc: self.nc.build(file),
            ne: self.ne.build(file),
            ns: self.ns.build(file),
            nb: self.nb.build(file),
            mname: self.mname.build(file),
            params: self.params.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::Diode {
    type Out = lexer::instance::Diode<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        lexer::instance::Diode {
            nplus: self.nplus.build(file),
            nminus: self.nminus.build(file),
            mname: self.mname.build(file),
            params: self.params.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::Subckt {
    type Out = lexer::instance::Subckt<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        lexer::instance::Subckt {
            ports: self.ports.build(file),
            cktname: self.cktname.build(file),
            params: self.params.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::General {
    type Out = lexer::General<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        lexer::General {
            cmd: self.cmd,
            tokens: self.tokens.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::Unknwon {
    type Out = lexer::Unknwon<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        lexer::Unknwon {
            cmd: self.cmd.build(file),
            tokens: self.tokens.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::Model {
    type Out = lexer::Model<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        lexer::Model {
            name: self.name.build(file),
            model_type: self.model_type.build(file),
            params: self.params.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::Data {
    type Out = lexer::Data<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        lexer::Data {
            name: self.name.build(file),
            values: self.values.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::DataValues {
    type Out = lexer::DataValues<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        match self {
            super::DataValues::InlineExpr { params, values } => lexer::DataValues::InlineExpr {
                params: params.build(file),
                values: values.build(file),
            },
            super::DataValues::InlineNum { params, values } => lexer::DataValues::InlineNum {
                params: params.build(file),
                values: values.build(file),
            },
            super::DataValues::MER(data_files) => todo!(),
            super::DataValues::LAM(data_files) => todo!(),
        }
    }
}

impl<'s> Builder<'s> for super::ModelType {
    type Out = lexer::ModelType<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        match self {
            super::ModelType::AMP => lexer::ModelType::AMP,
            super::ModelType::C => lexer::ModelType::C,
            super::ModelType::CORE => lexer::ModelType::CORE,
            super::ModelType::D => lexer::ModelType::D,
            super::ModelType::L => lexer::ModelType::L,
            super::ModelType::NJF => lexer::ModelType::NJF,
            super::ModelType::NMOS => lexer::ModelType::NMOS,
            super::ModelType::NPN => lexer::ModelType::NPN,
            super::ModelType::OPT => lexer::ModelType::OPT,
            super::ModelType::PJF => lexer::ModelType::PJF,
            super::ModelType::PMOS => lexer::ModelType::PMOS,
            super::ModelType::PNP => lexer::ModelType::PNP,
            super::ModelType::R => lexer::ModelType::R,
            super::ModelType::U => lexer::ModelType::U,
            super::ModelType::W => lexer::ModelType::W,
            super::ModelType::S => lexer::ModelType::S,
            super::ModelType::Unknown(span) => lexer::ModelType::Unknown(span.build(file)),
        }
    }
}
