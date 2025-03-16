use ::netlist_db::{Value, parser::top};
use pyo3::prelude::*;
use pyo3_polars::PyDataFrame;
use std::{collections::HashMap, path::PathBuf};

#[pyfunction]
fn obtain_datas(file: &str) -> PyResult<HashMap<String, PyDataFrame>> {
    _ = simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init();
    let (parsed, files) = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { top(PathBuf::from(file)).await });
    let (ast, _has_err) = files.build(parsed);
    ast.data
        .iter()
        .map(|data| match data.values.dataframe() {
            Ok(df) => Ok((data.name.to_string(), PyDataFrame(df))),
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(format!(
                "DataFrame error: {:?}",
                e
            ))),
        })
        .collect()
}

#[pyfunction]
fn obtain_nodeset_top(file: &str) -> PyResult<HashMap<String, f64>> {
    _ = simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init();
    let (parsed, files) = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { top(PathBuf::from(file)).await });
    let (ast, _has_err) = files.build(parsed);
    Ok(ast
        .nodeset
        .iter()
        .filter_map(|(node, volt, subskt)| {
            if subskt.is_none() {
                if let Value::Num(f) = volt {
                    Some((node.to_string(), *f))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect())
}

#[pymodule]
fn netlist_db(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(obtain_datas)).unwrap();
    m.add_wrapped(wrap_pyfunction!(obtain_nodeset_top)).unwrap();
    Ok(())
}
