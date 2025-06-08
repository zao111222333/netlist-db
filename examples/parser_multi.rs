// cargo run --example parser_multi -- tests/data.sp tests/lib.sp
// Avoid musl's default allocator due to lackluster performance
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use netlist_db::{FileId, parser::parse_top_multi};
use std::{env, path::PathBuf, process::exit, time::Instant};
#[tokio::main]
async fn main() {
    _ = simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        // .without_timestamps()
        .init();
    let now = Instant::now();
    let (parsed, files) = parse_top_multi(env::args().skip(1).map(|path| FileId::Include {
        path: PathBuf::from(path),
    }))
    .await;
    let elapsed_parse = now.elapsed();
    let now = Instant::now();
    let (ast, has_err) = files.build(parsed);
    let elapsed_build = now.elapsed();
    let now = Instant::now();
    println!("======= AST ===========");
    println!("{ast}");
    let elapsed_print = now.elapsed();
    println!("======= stats =========");
    println!("parse: {elapsed_parse:?}");
    println!("build: {elapsed_build:?}");
    println!("print: {elapsed_print:?}");
    println!("=======================");
    if has_err {
        exit(1)
    };
}
