// use netlist_db::{FileId, ast::Value, parser::parse_top};
use pyo3::prelude::*;
// use pyo3_polars::PyDataFrame;

// #[pyfunction]
// fn obtain_datas(path: PathBuf) -> PyResult<HashMap<String, PyDataFrame>> {
//     _ = simple_logger::SimpleLogger::new()
//         .with_level(log::LevelFilter::Info)
//         .init();
//     let (parsed, files) = tokio::runtime::Builder::new_multi_thread()
//         .enable_all()
//         .build()
//         .unwrap()
//         .block_on(async { parse_top(FileId::Include { path }).await });
//     let (ast, _has_err) = files.build(parsed);
//     ast.data
//         .iter()
//         .map(|data| match data.values.dataframe() {
//             Ok(df) => Ok((data.name.to_string(), PyDataFrame(df))),
//             Err(e) => Err(pyo3::exceptions::PyValueError::new_err(format!(
//                 "DataFrame error: {:?}",
//                 e
//             ))),
//         })
//         .collect()
// }

// #[pyfunction]
// fn obtain_nodeset_top(path: PathBuf) -> PyResult<HashMap<String, f64>> {
//     _ = simple_logger::SimpleLogger::new()
//         .with_level(log::LevelFilter::Info)
//         .init();
//     let (parsed, files) = tokio::runtime::Builder::new_multi_thread()
//         .enable_all()
//         .build()
//         .unwrap()
//         .block_on(async { parse_top(FileId::Include { path }).await });
//     let (ast, _has_err) = files.build(parsed);
//     Ok(ast
//         .nodeset
//         .iter()
//         .filter_map(|(node, volt, subskt)| {
//             if subskt.is_none() {
//                 if let Value::Num(f) = volt {
//                     Some((node.to_string(), *f))
//                 } else {
//                     None
//                 }
//             } else {
//                 None
//             }
//         })
//         .collect())
// }

#[pymodule]
fn netlist_db(m: &Bound<PyModule>) -> PyResult<()> {
    // m.add_wrapped(wrap_pyfunction!(obtain_datas)).unwrap();
    // m.add_wrapped(wrap_pyfunction!(obtain_nodeset_top)).unwrap();
    Ok(())
}
