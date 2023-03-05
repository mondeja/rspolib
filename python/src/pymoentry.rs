use pyo3::prelude::*;

use rspolib::prelude::*;
use rspolib::MOEntry;

#[pyclass]
#[pyo3(name = "MOEntry")]
pub struct PyMOEntry(MOEntry);

impl PyMOEntry {
    pub fn _inner(&self) -> MOEntry {
        self.0.clone()
    }
}

#[pymethods]
impl PyMOEntry {
    #[new]
    #[pyo3(
        signature = (
            msgid="".to_string(),
            msgstr=None,
            msgid_plural=None,
            msgstr_plural=vec![] as Vec<String>,
            msgctxt=None,
        )
    )]
    fn new(
        msgid: String,
        msgstr: Option<String>,
        msgid_plural: Option<String>,
        msgstr_plural: Vec<String>,
        msgctxt: Option<String>,
    ) -> Self {
        PyMOEntry(MOEntry::new(
            msgid,
            msgstr,
            msgid_plural,
            msgstr_plural,
            msgctxt,
        ))
    }

    #[getter]
    fn msgid(&self) -> PyResult<String> {
        Ok(self.0.msgid.clone())
    }

    #[setter]
    fn set_msgid(&mut self, msgid: String) -> PyResult<()> {
        self.0.msgid = msgid;
        Ok(())
    }

    #[getter]
    fn msgstr(&self) -> PyResult<Option<String>> {
        Ok(self.0.msgstr.clone())
    }

    #[setter]
    fn set_msgstr(&mut self, msgstr: Option<String>) -> PyResult<()> {
        self.0.msgstr = msgstr;
        Ok(())
    }

    #[getter]
    fn msgid_plural(&self) -> PyResult<Option<String>> {
        Ok(self.0.msgid_plural.clone())
    }

    #[setter]
    fn set_msgid_plural(
        &mut self,
        msgid_plural: Option<String>,
    ) -> PyResult<()> {
        self.0.msgid_plural = msgid_plural;
        Ok(())
    }

    #[getter]
    fn msgstr_plural(&self) -> PyResult<Vec<String>> {
        Ok(self.0.msgstr_plural.clone())
    }

    #[setter]
    fn set_msgstr_plural(
        &mut self,
        msgstr_plural: Vec<String>,
    ) -> PyResult<()> {
        self.0.msgstr_plural = msgstr_plural;
        Ok(())
    }

    #[getter]
    fn msgctxt(&self) -> PyResult<Option<String>> {
        Ok(self.0.msgctxt.clone())
    }

    #[setter]
    fn set_msgctxt(
        &mut self,
        msgctxt: Option<String>,
    ) -> PyResult<()> {
        self.0.msgctxt = msgctxt;
        Ok(())
    }

    #[pyo3(text_signature = "($self, wrapwidth=78)")]
    fn to_string_with_wrapwidth(
        &self,
        wrapwidth: usize,
    ) -> PyResult<String> {
        Ok(self.0.to_string_with_wrapwidth(wrapwidth))
    }

    fn msgid_eot_msgctxt(&self) -> PyResult<String> {
        Ok(self.0.msgid_eot_msgctxt())
    }

    fn translated(&self) -> PyResult<bool> {
        Ok(self.0.translated())
    }

    fn merge(&mut self, other: &PyMOEntry) -> PyResult<()> {
        self.0.merge(other.0.clone());
        Ok(())
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.0.to_string())
    }

    fn __eq__(&self, other: &PyMOEntry) -> PyResult<bool> {
        Ok(self.0 == other.0)
    }

    fn __ne__(&self, other: &PyMOEntry) -> PyResult<bool> {
        Ok(self.0 != other.0)
    }
}

impl From<&MOEntry> for PyMOEntry {
    fn from(entry: &MOEntry) -> Self {
        PyMOEntry(entry.clone())
    }
}
