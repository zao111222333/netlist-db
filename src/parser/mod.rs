mod cmds;
mod data_meas;
pub use data_meas::*;
mod data_sweep;
pub use data_sweep::*;
mod instance;
mod manager;
mod measure;
mod utils;

use alloc::sync::Arc;
use indexmap::IndexMap;
use manager::ParseManager;
use nom::IResult;
use regex::Regex;
use std::{collections::HashMap, mem, path::PathBuf, sync::OnceLock};
use utils::local_ast;

use super::{
    Files, Parsed,
    err::{ParseError, ParseErrorInner},
};
use crate::{
    ast::{ASTBuilder, LocalAST, Segment},
    span::{EndReason, FileId, FileStorage, LocatedSpan, ParsedId, Pos, span},
};

#[inline]
pub fn ast(
    manager: Arc<ParseManager>,
    loaded: IndexMap<FileId, Option<Pos>>,
    work_dir: PathBuf,
    mut i: LocatedSpan,
) -> IResult<LocatedSpan, ASTBuilder> {
    let mut ast = ASTBuilder::new();
    loop {
        let _ast;
        let reason;
        (i, (_ast, reason)) = local_ast(i, &loaded, &manager, &work_dir)?;
        if !_ast.is_empty() {
            ast.segments.push(Segment::Local(Box::new(_ast)));
        }
        match reason {
            EndReason::Include { file_name, section } => {
                let res = Arc::new(OnceLock::new());
                ast.segments.push(Segment::Include(res.clone()));
                let manager_clone = manager.clone();
                let file_path = if file_name.is_absolute() {
                    file_name.to_path_buf()
                } else {
                    work_dir.join(file_name)
                };
                let include_pos = Pos::new(i);
                let mut loaded = loaded.clone();
                if let Some((_, _include_pos)) = loaded.last_mut() {
                    *_include_pos = include_pos
                }
                manager.spawn_parse(async move {
                    include(manager_clone, loaded, file_path, section, include_pos, res).await;
                });
            }
            EndReason::End => return Ok((i, ast)),
        }
    }
}

async fn _include(
    manager: Arc<ParseManager>,
    mut loaded: IndexMap<FileId, Option<Pos>>,
    file_id: FileId,
    include_pos: Option<Pos>,
) -> Result<ParsedId, (FileId, ParseError)> {
    let mut file_path = file_id.path().to_path_buf();
    if let Some(idx) = loaded.get_index_of(&file_id) {
        return Err((
            file_id,
            ParseErrorInner::CircularDefinition(loaded, idx).with(include_pos),
        ));
    } else {
        loaded.insert(file_id.clone(), include_pos);
    }
    if let Some(parsed_id) = manager.file_storage.lock().await.existed(&file_id) {
        return Ok(parsed_id);
    }
    let contents = match tokio::fs::read_to_string(&file_path).await {
        Ok(contents) => contents,
        Err(e) => return Err((file_id, ParseErrorInner::with(e.into(), include_pos))),
    };
    let (line_off_set, file_ctx) = if let FileId::Section { path: _, section } = file_id.clone() {
        if let Some((file_ctx, line_off_set)) = match_lib(&contents, &section) {
            (line_off_set, file_ctx)
        } else {
            return Err((
                file_id,
                ParseErrorInner::NoLibSection {
                    path: file_path,
                    section,
                }
                .with(include_pos),
            ));
        }
    } else {
        (0, contents)
    };
    let parsed_id = manager.file_storage.lock().await.new_file(file_id);
    let i = span(&file_ctx, line_off_set);
    // get dir
    file_path.pop();
    #[cfg(test)]
    assert!(file_path.is_dir());

    let res = match ast(manager.clone(), loaded, file_path, i) {
        Ok((_, res)) => res,
        Err(e) => error2ast(e.into()),
    };
    manager
        .file_storage
        .lock()
        .await
        .update_ctx(&parsed_id, file_ctx, res);
    Ok(parsed_id)
}
#[inline]
async fn include(
    manager: Arc<ParseManager>,
    loaded: IndexMap<FileId, Option<Pos>>,
    file_path: PathBuf,
    section: Option<String>,
    include_pos: Option<Pos>,
    res: Arc<OnceLock<Result<ParsedId, ParseError>>>,
) {
    let file_id = if let Some(section) = section {
        FileId::Section {
            path: file_path.clone(),
            section,
        }
    } else {
        FileId::Include {
            path: file_path.clone(),
        }
    };
    let _res = _include(manager, loaded, file_id, include_pos).await;
    res.set(_res.map_err(|(_, e)| e)).unwrap();
}

/// When ParseError come form the included file,
/// also return the ASTBuilder, to recover the error
fn error2ast(err: ParseError) -> ASTBuilder {
    let mut ast = ASTBuilder::new();
    let mut local = LocalAST::default();
    local.errors.push(err);
    ast.segments.push(Segment::Local(Box::new(local)));
    ast
}

fn error2parsed(file_id: FileId, err: ParseError) -> (Parsed, Files) {
    let ast = error2ast(err);
    let parsed_id = ParsedId(0);
    let mut id2idx = HashMap::new();
    id2idx.insert(file_id.clone(), parsed_id);
    (
        Parsed {
            top_ids: vec![parsed_id],
            id2idx,
            inner: vec![(file_id, ast)],
        },
        Files {
            inner: vec![String::new()],
        },
    )
}

pub async fn parse_top_multi<I: Iterator<Item = FileId>>(file_ids: I) -> (Parsed, Files) {
    let (manager, done_rx) = ParseManager::new();
    let handles: Vec<_> = file_ids
        .into_iter()
        .map(|file_id| {
            crate::spawn({
                let manager = manager.clone();
                async move { _include(manager, IndexMap::with_capacity(1), file_id, None).await }
            })
        })
        .collect();
    let mut top_ids = Vec::with_capacity(handles.len());
    for handle in handles {
        match handle.await.unwrap() {
            Ok(parsed_id) => top_ids.push(parsed_id),
            Err((file_id, e)) => return error2parsed(file_id, e),
        }
    }
    manager.wait(done_rx).await;
    let mut guard = manager.file_storage.lock().await;
    let file_storage: FileStorage<ASTBuilder> = mem::take(&mut *guard);
    (
        Parsed {
            top_ids,
            id2idx: file_storage.id2idx,
            inner: file_storage.parsed,
        },
        Files {
            inner: file_storage.file,
        },
    )
}

fn match_lib(text: &str, section: &str) -> Option<(String, u32)> {
    let section_escaped = regex::escape(section);
    let re = Regex::new(&format!(
        r"(?ims)^\s*\.lib\s+{section_escaped}\b(.*?)^\s*\.endl(?:\s+{section_escaped}\b)?"
    ))
    .unwrap();
    re.captures_iter(text).last().map(|caps| {
        let start_offset = caps.get(0).unwrap().start();
        let line_num_offset = (text[..start_offset].matches('\n').count()) as u32;
        (caps[1].to_owned(), line_num_offset)
    })
}
#[tokio::test]
async fn test_top() {
    #[cfg(not(feature = "tracing"))]
    {
        _ = simple_logger::SimpleLogger::new().init();
    }
    #[cfg(feature = "tracing")]
    {
        let subscriber = tracing_subscriber::FmtSubscriber::builder()
            // .with_ansi(colored::control::SHOULD_COLORIZE.should_colorize())
            .with_max_level(tracing::Level::DEBUG)
            .with_target(false)
            .with_file(true)
            .with_line_number(true)
            .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
                "%FT%T".to_owned(),
            ))
            .finish();
        _ = tracing::subscriber::set_global_default(subscriber);
    }
    let (parsed, files) = parse_top_multi(
        [FileId::Include {
            path: PathBuf::from("tests/top.sp"),
        }]
        .into_iter(),
    )
    .await;
    println!("{parsed:?}");
    println!("{files:?}");
}
#[test]
fn test_match_lib() {
    let text = r#"
        .LIB TT
        * Some lines here
        M1 d g s b NMOS
        .MEAS ...
        .ENDL TT

        .lib tt
        R1 in out 10k
        .endl
        .lib ttg
        it is wrong
        .endl"#;
    assert_eq!(
        Some(("\n        R1 in out 10k\n".to_owned(), 6)),
        match_lib(text, "tt")
    );
}
