use std::collections::HashMap;

use crate::{
    file::{FileId, ParsedId, Span},
    lexer::{self, GeneralCmd, InstanceType, LocalAST, Segment},
};

#[derive(Debug)]
pub struct Parsed {
    pub top_id: ParsedId,
    pub id2idx: HashMap<FileId, ParsedId>,
    pub inner: Vec<(FileId, lexer::AST)>,
}
#[derive(Debug)]
pub struct Files {
    pub inner: Vec<String>,
}

pub trait Builder<'s> {
    type Out: 's;
    fn build(&self, file: &'s str) -> Self::Out;
    fn vec_iter(vec: &[Self], file: &'s str) -> impl Iterator<Item = Self::Out>
    where
        Self: Sized,
    {
        vec.iter().map(move |s| s.build(file))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Value<'s> {
    Num(f64),
    Expr(&'s str),
}

impl<'s> Builder<'s> for lexer::Value {
    type Out = Value<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        match self {
            lexer::Value::Num(float) => Value::Num(*float),
            lexer::Value::Expr(expr) => Value::Expr(expr.build(file)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KeyValue<'s> {
    pub k: &'s str,
    pub v: Value<'s>,
}

impl<'s> Builder<'s> for Span {
    type Out = &'s str;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        &file[self]
    }
}

impl<'s, T: Builder<'s>> Builder<'s> for Vec<T> {
    type Out = Vec<T::Out>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        T::vec_iter(self, file).collect()
    }
}

impl<'s> Builder<'s> for lexer::KeyValue {
    type Out = KeyValue<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        KeyValue {
            k: self.k.build(file),
            v: self.v.build(file),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Token<'s> {
    KV(KeyValue<'s>),
    Value(Value<'s>),
}

impl<'s> Builder<'s> for lexer::Token {
    type Out = Token<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        match self {
            lexer::Token::KV(key_value) => Token::KV(key_value.build(file)),
            lexer::Token::Value(v) => Token::Value(v.build(file)),
        }
    }
}

/// ``` spice
/// .subckt pulvt11ll_ckt d g s b w=1e-6 l=1e-6 sa='sar'
/// ...
/// .ends pulvt11ll_ckt
/// ```
/// Do NOT support `.include` / `.lib` in `.subckt`
#[derive(Debug, Clone)]
pub struct Subckt<'s> {
    pub name: &'s str,
    /// subckt/model name is the last arg
    pub ports: Vec<&'s str>,
    pub params: Vec<KeyValue<'s>>,
    pub ast: AST<'s>,
}

/// ``` spice
/// XX1 net48 D VDD VNW PHVT11LL_CKT W=0.22u L=40.00n
/// ```
#[derive(Debug, Clone)]
pub struct Instance<'s> {
    pub name: &'s str,
    pub instance_type: InstanceType,
    /// subckt/model name is the last arg
    pub ports: Vec<&'s str>,
    /// (fisrt, rest)
    pub params: Vec<KeyValue<'s>>,
}

impl<'s> Builder<'s> for lexer::Instance {
    type Out = Instance<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        Instance {
            name: self.name.build(file),
            instance_type: self.instance_type,
            ports: self.ports.build(file),
            params: self.params.build(file),
        }
    }
}

#[derive(Debug, Clone)]
pub struct General<'s> {
    pub cmd: GeneralCmd,
    pub tokens: Vec<Token<'s>>,
}

impl<'s> Builder<'s> for lexer::General {
    type Out = General<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        General {
            cmd: self.cmd,
            tokens: self.tokens.build(file),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Unknwon<'s> {
    pub cmd: &'s str,
    pub tokens: Vec<Token<'s>>,
}

impl<'s> Builder<'s> for lexer::Unknwon {
    type Out = Unknwon<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        Unknwon {
            cmd: self.cmd.build(file),
            tokens: self.tokens.build(file),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Model<'s> {
    pub name: &'s str,
    pub model_type: ModelType<'s>,
    pub params: Vec<KeyValue<'s>>,
}

impl<'s> Builder<'s> for lexer::Model {
    type Out = Model<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        Model {
            name: self.name.build(file),
            model_type: self.model_type.build(file),
            params: self.params.build(file),
        }
    }
}

#[expect(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum ModelType<'s> {
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
    Unknown(&'s str),
}

impl<'s> Builder<'s> for lexer::ModelType {
    type Out = ModelType<'s>;
    #[inline]
    fn build(&self, file: &'s str) -> Self::Out {
        match self {
            lexer::ModelType::AMP => ModelType::AMP,
            lexer::ModelType::C => ModelType::C,
            lexer::ModelType::CORE => ModelType::CORE,
            lexer::ModelType::D => ModelType::D,
            lexer::ModelType::L => ModelType::L,
            lexer::ModelType::NJF => ModelType::NJF,
            lexer::ModelType::NMOS => ModelType::NMOS,
            lexer::ModelType::NPN => ModelType::NPN,
            lexer::ModelType::OPT => ModelType::OPT,
            lexer::ModelType::PJF => ModelType::PJF,
            lexer::ModelType::PMOS => ModelType::PMOS,
            lexer::ModelType::PNP => ModelType::PNP,
            lexer::ModelType::R => ModelType::R,
            lexer::ModelType::U => ModelType::U,
            lexer::ModelType::W => ModelType::W,
            lexer::ModelType::S => ModelType::S,
            lexer::ModelType::Unknown(span) => ModelType::Unknown(span.build(file)),
        }
    }
}

#[expect(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Default)]
pub struct AST<'s> {
    pub subckt: Vec<Subckt<'s>>,
    pub instance: Vec<Instance<'s>>,
    pub model: Vec<Model<'s>>,
    pub param: Vec<KeyValue<'s>>,
    pub option: Vec<Token<'s>>,
    pub general: Vec<General<'s>>,
    pub unknwon: Vec<Unknwon<'s>>,
}

impl lexer::AST {
    #[expect(clippy::too_many_arguments)]
    fn build<'s>(
        &self,
        ast: &mut AST<'s>,
        has_err: &mut bool,
        file_id: &FileId,
        parsed_id: ParsedId,
        files: &'s Files,
        parsed: &Parsed,
    ) {
        fn build_local<'s>(
            local_ast: &LocalAST,
            ast: &mut AST<'s>,
            has_err: &mut bool,
            file: &'s str,
            file_id: &FileId,
            parsed_id: ParsedId,
            files: &'s Files,
            parsed: &Parsed,
        ) {
            fn build_subckt<'s>(
                s: &lexer::Subckt,
                has_err: &mut bool,
                file: &'s str,
                file_id: &FileId,
                parsed_id: ParsedId,
                files: &'s Files,
                parsed: &Parsed,
            ) -> Subckt<'s> {
                let mut ast = AST::default();
                s.ast
                    .build(&mut ast, has_err, file_id, parsed_id, files, parsed);
                Subckt {
                    name: s.name.build(file),
                    ports: s.ports.build(file),
                    params: s.params.build(file),
                    ast,
                }
            }
            ast.subckt.extend(
                local_ast
                    .subckt
                    .iter()
                    .map(|s| build_subckt(s, has_err, file, file_id, parsed_id, files, parsed)),
            );
            ast.instance
                .extend(lexer::Instance::vec_iter(&local_ast.instance, file));
            ast.model
                .extend(lexer::Model::vec_iter(&local_ast.model, file));
            ast.param
                .extend(lexer::KeyValue::vec_iter(&local_ast.param, file));
            ast.option
                .extend(lexer::Token::vec_iter(&local_ast.option, file));
            ast.general
                .extend(lexer::General::vec_iter(&local_ast.general, file));
            ast.unknwon
                .extend(lexer::Unknwon::vec_iter(&local_ast.unknwon, file));
            for e in &local_ast.errors {
                e.report(has_err, file_id, file);
            }
        }
        let file = &files.inner[parsed_id.0];
        for seg in &self.segments {
            match seg {
                Segment::Local(local_ast) => {
                    build_local(
                        local_ast, ast, has_err, file, file_id, parsed_id, files, parsed,
                    );
                }
                Segment::Include(ast_res) => {
                    let ast_res = ast_res.get().unwrap();
                    match ast_res {
                        Ok(parsed_id) => {
                            let (file_id, _ast) = &parsed.inner[parsed_id.0];
                            _ast.build(ast, has_err, file_id, *parsed_id, files, parsed);
                        }
                        Err(e) => {
                            e.report(has_err, file_id, file);
                        }
                    }
                }
            }
        }
    }
}

impl Files {
    #[inline]
    pub fn build(&self, parsed: Parsed) -> (AST<'_>, bool) {
        let mut ast = AST::default();
        let mut has_err = false;
        let (file_id, _ast) = &parsed.inner[parsed.top_id.0];
        _ast.build(
            &mut ast,
            &mut has_err,
            file_id,
            parsed.top_id,
            self,
            &parsed,
        );
        (ast, has_err)
    }
}
