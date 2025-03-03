// cargo run --example parser -- tests/demo.sp
// Avoid musl's default allocator due to lackluster performance
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use netlist_db::parser::top;
use std::{env, path::PathBuf, time::Instant};
#[tokio::main]
async fn main() {
    _ = simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        // .without_timestamps()
        .init();
    let args: Vec<String> = env::args().collect();
    let now = Instant::now();
    let (parsed, files) = top(PathBuf::from(args[1].clone())).await;
    let elapsed_parse = now.elapsed();
    let now = Instant::now();
    let (ast, has_err) = files.build(parsed);
    let elapsed_build = now.elapsed();
    let now = Instant::now();
    println!("======= AST ===========");
    println!("{ast}");
    println!("======= ERR ===========");
    println!("{has_err:?}");
    let elapsed_print = now.elapsed();
    println!("======= stats =========");
    println!("parse: {elapsed_parse:?}");
    println!("build: {elapsed_build:?}");
    println!("print: {elapsed_print:?}");
    println!("=======================");
}
