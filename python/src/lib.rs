use pyo3::prelude::*;

mod escaping;
mod exceptions;
mod pymoentry;
mod pymofile;
mod pypoentry;
mod pypofile;

use crate::escaping::{py_escape, py_unescape};
use crate::pymoentry::PyMOEntry;
use crate::pymofile::{py_mofile, PyMOFile};
use crate::pypoentry::PyPOEntry;
use crate::pypofile::{py_pofile, PyPOFile};

#[pymodule]
#[pyo3(name = "rspolib")]
fn py_rspolib(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Classes
    m.add_class::<PyMOEntry>()?;
    m.add_class::<PyPOEntry>()?;
    m.add_class::<PyMOFile>()?;
    m.add_class::<PyPOFile>()?;

    // Functions
    m.add_function(wrap_pyfunction!(py_pofile, m)?)?;
    m.add_function(wrap_pyfunction!(py_mofile, m)?)?;
    m.add_function(wrap_pyfunction!(py_escape, m)?)?;
    m.add_function(wrap_pyfunction!(py_unescape, m)?)?;

    // Exceptions
    m.add("IOError", py.get_type::<exceptions::IOError>())?;
    m.add("SyntaxError", py.get_type::<exceptions::SyntaxError>())?;
    m.add(
        "EscapingError",
        py.get_type::<exceptions::EscapingError>(),
    )?;
    Ok(())
}
