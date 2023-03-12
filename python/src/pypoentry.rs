use std::cmp::Ordering;

use pyo3::prelude::*;

use rspolib::prelude::*;
use rspolib::{EntryCmpByOptions, POEntry};

#[derive(Clone)]
#[pyclass]
#[pyo3(name = "POEntry")]
pub struct PyPOEntry(POEntry);

impl PyPOEntry {
    pub fn _inner(&self) -> POEntry {
        self.0.clone()
    }
}

#[pymethods]
impl PyPOEntry {
    #[allow(clippy::too_many_arguments)]
    #[new]
    #[pyo3(
        signature = (
            msgid="".to_string(),
            msgstr=None,
            msgid_plural=None,
            msgstr_plural=vec![] as Vec<String>,
            msgctxt=None,
            tcomment=None,
            comment=None,
            flags=vec![] as Vec<String>,
        )
    )]
    fn new(
        msgid: String,
        msgstr: Option<String>,
        msgid_plural: Option<String>,
        msgstr_plural: Vec<String>,
        msgctxt: Option<String>,
        tcomment: Option<String>,
        comment: Option<String>,
        flags: Vec<String>,
    ) -> Self {
        let mut poentry = POEntry::new(0);
        poentry.msgid = msgid;
        poentry.msgstr = msgstr;
        poentry.msgid_plural = msgid_plural;
        poentry.msgstr_plural = msgstr_plural;
        poentry.msgctxt = msgctxt;
        poentry.tcomment = tcomment;
        poentry.comment = comment;
        poentry.flags = flags;
        PyPOEntry(poentry)
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

    fn get_msgstr_plural(&self) -> PyResult<Vec<String>> {
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

    #[getter]
    fn obsolete(&self) -> PyResult<bool> {
        Ok(self.0.obsolete)
    }

    #[setter]
    fn set_obsolete(&mut self, obsolete: bool) -> PyResult<()> {
        self.0.obsolete = obsolete;
        Ok(())
    }

    #[getter]
    fn comment(&self) -> PyResult<Option<String>> {
        Ok(self.0.comment.clone())
    }

    #[setter]
    fn set_comment(
        &mut self,
        comment: Option<String>,
    ) -> PyResult<()> {
        self.0.comment = comment;
        Ok(())
    }

    #[getter]
    fn tcomment(&self) -> PyResult<Option<String>> {
        Ok(self.0.tcomment.clone())
    }

    #[setter]
    fn set_tcomment(
        &mut self,
        tcomment: Option<String>,
    ) -> PyResult<()> {
        self.0.tcomment = tcomment;
        Ok(())
    }

    fn get_occurrences(&self) -> PyResult<Vec<(String, String)>> {
        Ok(self.0.occurrences.clone())
    }

    #[setter]
    fn set_occurrences(
        &mut self,
        occurrences: Vec<(String, String)>,
    ) -> PyResult<()> {
        self.0.occurrences = occurrences;
        Ok(())
    }

    fn get_flags(&self) -> PyResult<Vec<String>> {
        Ok(self.0.flags.clone())
    }

    #[setter]
    fn set_flags(&mut self, flags: Vec<String>) -> PyResult<()> {
        self.0.flags = flags;
        Ok(())
    }

    #[getter]
    fn previous_msgid(&self) -> PyResult<Option<String>> {
        Ok(self.0.previous_msgid.clone())
    }

    #[setter]
    fn set_previous_msgid(
        &mut self,
        previous_msgid: Option<String>,
    ) -> PyResult<()> {
        self.0.previous_msgid = previous_msgid;
        Ok(())
    }

    #[getter]
    fn previous_msgid_plural(&self) -> PyResult<Option<String>> {
        Ok(self.0.previous_msgid_plural.clone())
    }

    #[setter]
    fn set_previous_msgid_plural(
        &mut self,
        previous_msgid_plural: Option<String>,
    ) -> PyResult<()> {
        self.0.previous_msgid_plural = previous_msgid_plural;
        Ok(())
    }

    #[getter]
    fn previous_msgctxt(&self) -> PyResult<Option<String>> {
        Ok(self.0.previous_msgctxt.clone())
    }

    #[setter]
    fn set_previous_msgctxt(
        &mut self,
        previous_msgctxt: Option<String>,
    ) -> PyResult<()> {
        self.0.previous_msgctxt = previous_msgctxt;
        Ok(())
    }

    #[getter]
    fn linenum(&self) -> PyResult<usize> {
        Ok(self.0.linenum)
    }

    #[setter]
    fn set_linenum(&mut self, linenum: usize) -> PyResult<()> {
        self.0.linenum = linenum;
        Ok(())
    }

    #[getter]
    fn fuzzy(&self) -> PyResult<bool> {
        Ok(self.0.fuzzy())
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

    fn merge(&mut self, other: &PyPOEntry) -> PyResult<()> {
        self.0.merge(other.0.clone());
        Ok(())
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.0.to_string())
    }

    fn __eq__(&self, other: &PyPOEntry) -> PyResult<bool> {
        Ok(self
            .0
            .cmp_by(&other._inner(), &EntryCmpByOptions::default())
            == Ordering::Equal)
    }

    fn __ne__(&self, other: &PyPOEntry) -> PyResult<bool> {
        Ok(self
            .0
            .cmp_by(&other._inner(), &EntryCmpByOptions::default())
            != Ordering::Equal)
    }

    fn __gt__(&self, other: &PyPOEntry) -> PyResult<bool> {
        Ok(self
            .0
            .cmp_by(&other._inner(), &EntryCmpByOptions::default())
            == Ordering::Greater)
    }

    fn __ge__(&self, other: &PyPOEntry) -> PyResult<bool> {
        Ok(self
            .0
            .cmp_by(&other._inner(), &EntryCmpByOptions::default())
            != Ordering::Less)
    }

    fn __lt__(&self, other: &PyPOEntry) -> PyResult<bool> {
        Ok(self
            .0
            .cmp_by(&other._inner(), &EntryCmpByOptions::default())
            == Ordering::Less)
    }

    fn __le__(&self, other: &PyPOEntry) -> PyResult<bool> {
        Ok(self
            .0
            .cmp_by(&other._inner(), &EntryCmpByOptions::default())
            != Ordering::Greater)
    }

    fn __cmp__(&self, other: &PyPOEntry) -> PyResult<i8> {
        Ok(
            match self.0.cmp_by(
                &other._inner(),
                &EntryCmpByOptions::default(),
            ) {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature=(other, msgid=true, msgstr=true, msgctxt=true, obsolete=true, msgid_plural=true, msgstr_plural=true, occurrences=true, flags=true))]
    fn cmp_by(
        &self,
        other: &PyPOEntry,
        msgid: bool,
        msgstr: bool,
        msgctxt: bool,
        obsolete: bool,
        msgid_plural: bool,
        msgstr_plural: bool,
        occurrences: bool,
        flags: bool,
    ) -> PyResult<i8> {
        Ok(
            match self.0.cmp_by(
                &other._inner(),
                &EntryCmpByOptions::from(&vec![
                    ("msgid".to_string(), msgid),
                    ("msgstr".to_string(), msgstr),
                    ("msgctxt".to_string(), msgctxt),
                    ("obsolete".to_string(), obsolete),
                    ("msgid_plural".to_string(), msgid_plural),
                    ("msgstr_plural".to_string(), msgstr_plural),
                    ("occurrences".to_string(), occurrences),
                    ("flags".to_string(), flags),
                ]),
            ) {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            },
        )
    }
}

impl From<&POEntry> for PyPOEntry {
    fn from(entry: &POEntry) -> Self {
        PyPOEntry(entry.clone())
    }
}

impl From<PyObject> for PyPOEntry {
    fn from(obj: PyObject) -> Self {
        Python::with_gil(move |py| {
            obj.extract::<PyPOEntry>(py).unwrap()
        })
    }
}
