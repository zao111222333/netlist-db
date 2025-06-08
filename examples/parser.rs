// cargo run --example parser -- tests/top.sp
// Avoid musl's default allocator due to lackluster performance
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use netlist_db::{FileId, parser::parse_top};
use std::{env, path::PathBuf, process::exit, time::Instant};
#[tokio::main]
async fn main() {
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
    let now = Instant::now();
    let (parsed, files) = parse_top(FileId::Include {
        path: PathBuf::from(env::args().nth(1).unwrap()),
    })
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
