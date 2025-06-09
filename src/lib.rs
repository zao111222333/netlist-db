#[cfg(feature = "py")]
mod py;
#[expect(unused_imports)]
#[cfg(not(feature = "tracing"))]
use log::{debug, error, info, trace, warn};
#[expect(unused_imports)]
#[cfg(feature = "tracing")]
use tracing::{debug, error, info, trace, warn};

extern crate alloc;
pub mod ast;
pub mod instance;
pub mod parser;

pub mod _impl_display;
mod _impl_hash;
mod builder;
mod err;
mod span;

use alloc::borrow::Cow;
use ast::ASTBuilder;
use indexmap::IndexSet;
pub use span::{FileId, ParsedId};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Parsed {
    pub top_ids: Vec<span::ParsedId>,
    pub id2idx: HashMap<span::FileId, span::ParsedId>,
    pub inner: Vec<(span::FileId, ast::ASTBuilder)>,
}
#[derive(Debug)]
pub struct Files {
    pub inner: Vec<String>,
}

/// ``` spice
/// .subckt pulvt11ll_ckt d g s b w=1e-6 l=1e-6 sa='sar'
/// ...
/// .ends pulvt11ll_ckt
/// ```
/// Do NOT support `.include` / `.lib` in `.subckt`
#[derive(Debug, Clone)]
pub struct Subckt<'s> {
    pub name: Cow<'s, str>,
    /// subckt/model name is the last arg
    pub ports: Vec<Cow<'s, str>>,
    pub params: Vec<ast::KeyValue<'s>>,
    pub ast: AST<'s>,
}

#[derive(Debug, Clone, Default)]
pub struct AST<'s> {
    pub subckt: IndexSet<Subckt<'s>>,
    pub instance: Vec<instance::Instance<'s>>,
    pub model: Vec<ast::Model<'s>>,
    pub param: Vec<ast::KeyValue<'s>>,
    pub option: Vec<(Cow<'s, str>, Option<ast::Value<'s>>)>,
    /// transient initial conditions
    /// https://eda-cpu1.eias.junzhuo.site/~junzhuo/hspice/index.htm#page/hspice_14/ic.htm
    ///
    /// `node, val, [subckt]`
    pub init_condition: Vec<(Cow<'s, str>, ast::Value<'s>, Option<Cow<'s, str>>)>,
    /// Initializes specified nodal voltages for DC operating point analysis and corrects convergence problems in DC analysis.
    /// https://eda-cpu1.eias.junzhuo.site/~junzhuo/hspice/index.htm#page/hspice_14/nodeset.htm
    ///
    /// `node, val, [subckt]`
    pub nodeset: Vec<(Cow<'s, str>, ast::Value<'s>, Option<Cow<'s, str>>)>,
    pub general: Vec<ast::General<'s>>,
    pub data: Vec<ast::Data<'s>>,
    pub unknwon: Vec<ast::Unknwon<'s>>,
}

// #[cfg(feature = "py")]
// use polars::{error::PolarsError, frame::DataFrame, prelude::Column};
// pub struct DataValuesCsv<'s, 'a>(pub(crate) &'a ast::DataValues<'s>);
// impl<'s> ast::DataValues<'s> {
//     pub fn csv(&self) -> DataValuesCsv<'s, '_> {
//         DataValuesCsv(self)
//     }
//     #[cfg(feature = "py")]
//     pub fn dataframe(&self) -> Result<DataFrame, PolarsError> {
//         if let Self::InlineNum { params, values } = self {
//             let ncols = params.len();
//             if ncols == 0 {
//                 return Err(PolarsError::ComputeError("Header is empty".into()));
//             }
//             if values.len() % ncols != 0 {
//                 return Err(PolarsError::ComputeError(
//                     "Data length is not a multiple of the number of columns".into(),
//                 ));
//             }
//             let nrows = values.len() / ncols;
//             let columns = params
//                 .into_iter()
//                 .enumerate()
//                 .map(|(col_idx, name)| {
//                     Column::new(
//                         name.as_ref().into(),
//                         (0..nrows)
//                             .into_iter()
//                             .map(|row| values[row * ncols + col_idx])
//                             .collect::<Vec<f64>>(),
//                     )
//                 })
//                 .collect();
//             DataFrame::new(columns)
//         } else {
//             Err(PolarsError::ComputeError("Is not inline data".into()))
//         }
//     }
// }

impl ast::ASTBuilder {
    #[expect(clippy::too_many_arguments)]
    fn build<'s>(
        &self,
        ast: &mut AST<'s>,
        has_err: &mut bool,
        file_id: &span::FileId,
        parsed_id: span::ParsedId,
        files: &'s Files,
        parsed_id2idx: &HashMap<FileId, ParsedId>,
        parsed_inner: &Vec<(FileId, ASTBuilder)>,
    ) {
        use builder::Builder as _;
        fn build_local<'s>(
            local_ast: &ast::LocalAST,
            ast: &mut AST<'s>,
            has_err: &mut bool,
            file: &'s str,
            file_id: &span::FileId,
            parsed_id: span::ParsedId,
            files: &'s Files,
            parsed_id2idx: &HashMap<FileId, ParsedId>,
            parsed_inner: &Vec<(FileId, ASTBuilder)>,
        ) {
            fn build_subckt<'s>(
                s: &ast::SubcktBuilder,
                has_err: &mut bool,
                file: &'s str,
                file_id: &span::FileId,
                parsed_id: span::ParsedId,
                files: &'s Files,
                parsed_id2idx: &HashMap<FileId, ParsedId>,
                parsed_inner: &Vec<(FileId, ASTBuilder)>,
            ) -> Subckt<'s> {
                let mut ast = AST::default();
                s.ast.build(
                    &mut ast,
                    has_err,
                    file_id,
                    parsed_id,
                    files,
                    parsed_id2idx,
                    parsed_inner,
                );
                Subckt {
                    name: s.name.build(file),
                    ports: s.ports.build(file),
                    params: s.params.build(file),
                    ast,
                }
            }
            ast.subckt.extend(local_ast.subckt.iter().map(|s| {
                build_subckt(
                    s,
                    has_err,
                    file,
                    file_id,
                    parsed_id,
                    files,
                    parsed_id2idx,
                    parsed_inner,
                )
            }));
            ast.instance
                .extend(local_ast.instance.iter().map(|b| b.build(file)));
            ast.model
                .extend(local_ast.model.iter().map(|b| b.build(file)));
            ast.param
                .extend(local_ast.param.iter().map(|b| b.build(file)));
            ast.option
                .extend(local_ast.option.iter().map(|b| b.build(file)));
            ast.general
                .extend(local_ast.general.iter().map(|b| b.build(file)));
            ast.data
                .extend(local_ast.data.iter().map(|b| b.build(file)));
            ast.init_condition
                .extend(local_ast.init_condition.iter().map(|b| b.build(file)));
            ast.nodeset
                .extend(local_ast.nodeset.iter().map(|b| b.build(file)));
            ast.unknwon
                .extend(local_ast.unknwon.iter().map(|b| b.build(file)));
            for e in &local_ast.errors {
                e.report(has_err, file_id, file);
            }
        }
        let file = &files.inner[parsed_id.0];
        for seg in &self.segments {
            match seg {
                ast::Segment::Local(local_ast) => {
                    build_local(
                        local_ast,
                        ast,
                        has_err,
                        file,
                        file_id,
                        parsed_id,
                        files,
                        parsed_id2idx,
                        parsed_inner,
                    );
                }
                ast::Segment::Include(ast_res) => {
                    let ast_res = ast_res.get().unwrap();
                    match ast_res {
                        Ok(parsed_id) => {
                            let (file_id, _ast) = &parsed_inner[parsed_id.0];
                            _ast.build(
                                ast,
                                has_err,
                                file_id,
                                *parsed_id,
                                files,
                                parsed_id2idx,
                                parsed_inner,
                            );
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
        for top_id in parsed.top_ids {
            let (file_id, _ast) = &parsed.inner[top_id.0];
            _ast.build(
                &mut ast,
                &mut has_err,
                file_id,
                top_id,
                self,
                &parsed.id2idx,
                &parsed.inner,
            );
        }
        (ast, has_err)
    }
}
