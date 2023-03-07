use pyo3::exceptions::PyException;

pyo3::create_exception!(rspolib, IOError, PyException);
pyo3::create_exception!(rspolib, SyntaxError, PyException);
pyo3::create_exception!(rspolib, EscapingError, PyException);
