use super::Span;
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
impl<'s, T1: Builder<'s>, T2: Builder<'s>, T3: Builder<'s>> Builder<'s> for (T1, T2, T3) {
    type Out = (T1::Out, T2::Out, T3::Out);
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        (self.0.build(file), self.1.build(file), self.2.build(file))
    }
}

impl<'s> Builder<'s> for super::Value {
    type Out = crate::Value<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        match self {
            super::Value::Num(float) => crate::Value::Num(float.build(file)),
            super::Value::Expr(expr) => crate::Value::Expr(expr.build(file)),
        }
    }
}

impl<'s> Builder<'s> for super::KeyValue {
    type Out = crate::KeyValue<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        crate::KeyValue {
            k: self.k.build(file),
            v: self.v.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::Token {
    type Out = crate::Token<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        match self {
            super::Token::KV(key_value) => crate::Token::KV(key_value.build(file)),
            super::Token::Value(v) => crate::Token::Value(v.build(file)),
            super::Token::V(v) => crate::Token::V(v.build(file)),
            super::Token::I(v) => crate::Token::I(v.build(file)),
        }
    }
}

impl<'s> Builder<'s> for super::instance::Instance {
    type Out = crate::instance::Instance<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        crate::instance::Instance {
            name: self.name.build(file),
            ctx: self.ctx.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::InstanceCtx {
    type Out = crate::instance::InstanceCtx<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        match self {
            super::instance::InstanceCtx::Resistor(resistor) => {
                crate::instance::InstanceCtx::Resistor(resistor.build(file))
            }
            super::instance::InstanceCtx::Capacitor(capacitor) => {
                crate::instance::InstanceCtx::Capacitor(capacitor.build(file))
            }
            super::instance::InstanceCtx::Inductor(inductor) => {
                crate::instance::InstanceCtx::Inductor(inductor.build(file))
            }
            super::instance::InstanceCtx::Voltage(voltage) => {
                crate::instance::InstanceCtx::Voltage(voltage.build(file))
            }
            super::instance::InstanceCtx::Current(current) => {
                crate::instance::InstanceCtx::Current(current.build(file))
            }
            super::instance::InstanceCtx::MOSFET(mosfet) => {
                crate::instance::InstanceCtx::MOSFET(mosfet.build(file))
            }
            super::instance::InstanceCtx::BJT(bjt) => {
                crate::instance::InstanceCtx::BJT(bjt.build(file))
            }
            super::instance::InstanceCtx::Diode(diode) => {
                crate::instance::InstanceCtx::Diode(diode.build(file))
            }
            super::instance::InstanceCtx::Subckt(subckt) => {
                crate::instance::InstanceCtx::Subckt(subckt.build(file))
            }
            super::instance::InstanceCtx::Unknown {
                r#type,
                ports,
                params,
            } => crate::instance::InstanceCtx::Unknown {
                r#type: *r#type,
                ports: ports.build(file),
                params: params.build(file),
            },
        }
    }
}

impl<'s> Builder<'s> for super::instance::Resistor {
    type Out = crate::instance::Resistor<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        crate::instance::Resistor {
            n1: self.n1.build(file),
            n2: self.n2.build(file),
            value: self.value.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::Capacitor {
    type Out = crate::instance::Capacitor<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        crate::instance::Capacitor {
            n1: self.n1.build(file),
            n2: self.n2.build(file),
            value: self.value.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::Voltage {
    type Out = crate::instance::Voltage<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        crate::instance::Voltage {
            n1: self.n1.build(file),
            n2: self.n2.build(file),
            source: self.source.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::Current {
    type Out = crate::instance::Current<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        crate::instance::Current {
            n1: self.n1.build(file),
            n2: self.n2.build(file),
            source: self.source.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::VoltageSource {
    type Out = crate::instance::VoltageSource<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        match self {
            super::instance::VoltageSource::Params(params) => {
                crate::instance::VoltageSource::Params(params.build(file))
            }
            super::instance::VoltageSource::Value(value) => {
                crate::instance::VoltageSource::Value(value.build(file))
            }
            super::instance::VoltageSource::PWL(pwl) => {
                crate::instance::VoltageSource::PWL(pwl.build(file))
            }
        }
    }
}

impl<'s> Builder<'s> for super::instance::CurrentSource {
    type Out = crate::instance::CurrentSource<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        match self {
            super::instance::CurrentSource::Params(params) => {
                crate::instance::CurrentSource::Params(params.build(file))
            }
            super::instance::CurrentSource::Value(value) => {
                crate::instance::CurrentSource::Value(value.build(file))
            }
            super::instance::CurrentSource::PWL(pwl) => {
                crate::instance::CurrentSource::PWL(pwl.build(file))
            }
        }
    }
}

impl<'s> Builder<'s> for super::instance::TimeValuePoint {
    type Out = crate::instance::TimeValuePoint<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        crate::instance::TimeValuePoint {
            time: self.time.build(file),
            value: self.value.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::PWL {
    type Out = crate::instance::PWL<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        crate::instance::PWL {
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
    type Out = crate::instance::Inductor<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        crate::instance::Inductor {
            n1: self.n1.build(file),
            n2: self.n2.build(file),
            value: self.value.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::MOSFET {
    type Out = crate::instance::MOSFET<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        crate::instance::MOSFET {
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
    type Out = crate::instance::BJT<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        crate::instance::BJT {
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
    type Out = crate::instance::Diode<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        crate::instance::Diode {
            nplus: self.nplus.build(file),
            nminus: self.nminus.build(file),
            mname: self.mname.build(file),
            params: self.params.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::instance::Subckt {
    type Out = crate::instance::Subckt<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        crate::instance::Subckt {
            ports: self.ports.build(file),
            cktname: self.cktname.build(file),
            params: self.params.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::General {
    type Out = crate::General<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        crate::General {
            cmd: self.cmd,
            tokens: self.tokens.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::Unknwon {
    type Out = crate::Unknwon<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        crate::Unknwon {
            cmd: self.cmd.build(file),
            tokens: self.tokens.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::Model {
    type Out = crate::Model<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        crate::Model {
            name: self.name.build(file),
            model_type: self.model_type.build(file),
            params: self.params.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::Data {
    type Out = crate::Data<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        crate::Data {
            name: self.name.build(file),
            values: self.values.build(file),
        }
    }
}

impl<'s> Builder<'s> for super::DataValues {
    type Out = crate::DataValues<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        match self {
            super::DataValues::InlineExpr { params, values } => crate::DataValues::InlineExpr {
                params: params.build(file),
                values: values.build(file),
            },
            super::DataValues::InlineNum { params, values } => crate::DataValues::InlineNum {
                params: params.build(file),
                values: values.build(file),
            },
            super::DataValues::MER(data_files) => todo!(),
            super::DataValues::LAM(data_files) => todo!(),
        }
    }
}

impl<'s> Builder<'s> for super::ModelType {
    type Out = crate::ModelType<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        match self {
            super::ModelType::AMP => crate::ModelType::AMP,
            super::ModelType::C => crate::ModelType::C,
            super::ModelType::CORE => crate::ModelType::CORE,
            super::ModelType::D => crate::ModelType::D,
            super::ModelType::L => crate::ModelType::L,
            super::ModelType::NJF => crate::ModelType::NJF,
            super::ModelType::NMOS => crate::ModelType::NMOS,
            super::ModelType::NPN => crate::ModelType::NPN,
            super::ModelType::OPT => crate::ModelType::OPT,
            super::ModelType::PJF => crate::ModelType::PJF,
            super::ModelType::PMOS => crate::ModelType::PMOS,
            super::ModelType::PNP => crate::ModelType::PNP,
            super::ModelType::R => crate::ModelType::R,
            super::ModelType::U => crate::ModelType::U,
            super::ModelType::W => crate::ModelType::W,
            super::ModelType::S => crate::ModelType::S,
            super::ModelType::Unknown(span) => crate::ModelType::Unknown(span.build(file)),
        }
    }
}
