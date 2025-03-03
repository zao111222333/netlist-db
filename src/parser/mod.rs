use std::{
    collections::HashMap,
    mem,
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use indexmap::IndexMap;
use manager::ParseManager;
use nom::IResult;
use regex::Regex;
use utils::local_ast;

use super::{
    Files, ParseError, ParseErrorInner, Parsed,
    builder::{
        AST, LocalAST, Segment,
        span::{EndReason, FileId, FileStorage, LocatedSpan, ParsedId, Pos, span},
    },
};

mod instance;
mod manager;
mod utils;

#[inline]
pub fn ast(
    manager: Arc<ParseManager>,
    loaded: IndexMap<FileId, Option<Pos>>,
    work_dir: PathBuf,
    mut i: LocatedSpan,
) -> IResult<LocatedSpan, AST> {
    let mut ast = AST::new();
    loop {
        log::trace!("\n{:?}", i.fragment());
        let _ast;
        let reason;
        (i, (_ast, reason)) = local_ast(i, &loaded, &manager, &work_dir)?;
        if !_ast.is_empty() {
            ast.segments.push(Segment::Local(_ast));
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
                let pos = Pos::new(i);
                let mut loaded = loaded.clone();
                if let Some((_, _pos)) = loaded.last_mut() {
                    *_pos = pos
                }
                manager.spawn_parse(async move {
                    include(manager_clone, loaded, file_path, pos, section, res).await;
                });
            }
            EndReason::End => return Ok((i, ast)),
        }
    }
}

async fn _include(
    manager: Arc<ParseManager>,
    mut loaded: IndexMap<FileId, Option<Pos>>,
    mut file_path: PathBuf,
    pos: Option<Pos>,
    section: Option<String>,
) -> Result<ParsedId, ParseError> {
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
    if let Some(idx) = loaded.get_index_of(&file_id) {
        return Err(ParseErrorInner::CircularDefinition(loaded, idx).with(pos));
    } else {
        loaded.insert(file_id.clone(), pos);
    }
    if let Some(parsed_id) = manager.file_storage.lock().await.existed(&file_id) {
        return Ok(parsed_id);
    }
    let contents = tokio::fs::read_to_string(&file_path)
        .await
        .map_err(|e| ParseErrorInner::with(e.into(), pos))?;
    let (line_off_set, file_ctx) = if let FileId::Section { path: _, section } = &file_id {
        if let Some((file_ctx, line_off_set)) = match_lib(&contents, section) {
            (line_off_set, file_ctx)
        } else {
            return Err(ParseErrorInner::NoLibSection {
                path: file_path,
                section: section.clone(),
            }
            .with(pos));
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
    pos: Option<Pos>,
    section: Option<String>,
    res: Arc<OnceLock<Result<ParsedId, ParseError>>>,
) {
    let _res = _include(manager, loaded, file_path, pos, section).await;
    res.set(_res).unwrap();
}

/// When ParseError come form the included file,
/// also return the AST, to recover the error
fn error2ast(err: ParseError) -> AST {
    let mut ast = AST::new();
    let mut local = LocalAST::default();
    local.errors.push(err);
    ast.segments.push(Segment::Local(local));
    ast
}

fn error2parsed(file_id: FileId, err: ParseError) -> (Parsed, Files) {
    let ast = error2ast(err);
    let parsed_id = ParsedId(0);
    let mut id2idx = HashMap::new();
    id2idx.insert(file_id.clone(), parsed_id);
    (
        Parsed {
            top_id: parsed_id,
            id2idx,
            inner: vec![(file_id, ast)],
        },
        Files {
            inner: vec![String::new()],
        },
    )
}

pub async fn top(mut path: PathBuf) -> (Parsed, Files) {
    let (manager, done_rx) = ParseManager::new();
    let file_id = FileId::Include { path: path.clone() };
    let file_ctx = match tokio::fs::read_to_string(&path).await {
        Ok(s) => s,
        Err(e) => return error2parsed(file_id, ParseErrorInner::with(e.into(), None)),
    };
    let mut loaded = IndexMap::with_capacity(1);
    loaded.insert(file_id.clone(), None);
    let parsed_id = manager.file_storage.lock().await.new_file(file_id);
    path.pop();
    let input = span(&file_ctx, 0);
    let res = match ast(manager.clone(), loaded, path, input) {
        Ok((_, res)) => res,
        Err(e) => error2ast(e.into()),
    };
    manager.wait(done_rx).await;
    let mut guard = manager.file_storage.lock().await;
    let mut file_storage: FileStorage<AST> = mem::take(&mut *guard);
    file_storage.update_ctx(&parsed_id, file_ctx, res);
    (
        Parsed {
            top_id: parsed_id,
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
    _ = simple_logger::SimpleLogger::new().init();
    let (parsed, files) = top(PathBuf::from("tests/top.sp")).await;
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
