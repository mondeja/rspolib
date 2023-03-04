use pyo3::prelude::*;

mod pymoentry;
mod pymofile;
mod pypoentry;
mod pypofile;

use crate::pymoentry::PyMOEntry;
use crate::pymofile::{py_mofile, PyMOFile};
use crate::pypoentry::PyPOEntry;
use crate::pypofile::{py_pofile, PyPOFile};

#[pymodule]
#[pyo3(name = "rspolib")]
fn py_rspolib(_py: Python, m: &PyModule) -> PyResult<()> {
    //m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<PyMOEntry>()?;
    m.add_class::<PyPOEntry>()?;
    m.add_class::<PyMOFile>()?;
    m.add_class::<PyPOFile>()?;
    m.add_function(wrap_pyfunction!(py_pofile, m)?)?;
    m.add_function(wrap_pyfunction!(py_mofile, m)?)?;
    Ok(())
}
