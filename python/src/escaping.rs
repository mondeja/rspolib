use pyo3::prelude::*;
use rspolib::escaping::{escape, unescape};

#[pyfunction]
#[pyo3(name = "escape")]
pub fn py_escape(text: &str) -> PyResult<String> {
    Ok(escape(text))
}

#[pyfunction]
#[pyo3(name = "unescape")]
pub fn py_unescape(text: &str) -> PyResult<String> {
    Ok(unescape(text))
}
