use crate::exceptions;
use pyo3::prelude::*;
use rspolib::escaping::{escape, unescape};

#[pyfunction]
#[pyo3(name = "escape")]
pub fn py_escape(text: &str) -> PyResult<String> {
    Ok(escape(text).to_string())
}

#[pyfunction]
#[pyo3(name = "unescape")]
pub fn py_unescape(text: &str) -> PyResult<String> {
    let ret = unescape(text);
    match ret {
        Ok(ret) => Ok(ret.to_string()),
        Err(e) => Err(PyErr::new::<exceptions::EscapingError, _>(
            e.to_string(),
        )),
    }
}
