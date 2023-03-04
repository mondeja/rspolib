use std::collections::HashMap;

use pyo3::prelude::*;

use crate::pypoentry::PyPOEntry;
use rspolib::{
    pofile, AsBytes, FileOptions, MOFile, POFile, Save, SaveAsMOFile,
    SaveAsPOFile,
};

#[pyfunction]
#[pyo3(name = "pofile")]
#[pyo3(signature = (path_or_content, wrapwidth=78))]
pub fn py_pofile(
    path_or_content: &str,
    wrapwidth: usize,
) -> PyResult<PyPOFile> {
    let result =
        pofile(FileOptions::from((path_or_content, wrapwidth)));
    match result {
        Ok(pofile) => Ok(PyPOFile(pofile)),
        Err(e) => {
            Err(PyErr::new::<pyo3::exceptions::PyException, _>(
                e.to_string(),
            ))
        }
    }
}

#[pyclass]
#[pyo3(name = "POFile")]
pub struct PyPOFile(POFile);

#[pymethods]
impl PyPOFile {
    #[getter]
    fn entries(&self) -> PyResult<Vec<PyPOEntry>> {
        let mut entries = Vec::new();
        for entry in &self.0.entries {
            entries.push(PyPOEntry::from(entry));
        }
        Ok(entries)
    }

    #[getter]
    fn header(&self) -> PyResult<String> {
        Ok(self.0.header.clone().unwrap_or("".to_string()))
    }

    #[getter]
    fn metadata(&self) -> PyResult<HashMap<String, String>> {
        Ok(self.0.metadata.clone())
    }

    #[getter]
    fn metadata_is_fuzzy(&self) -> PyResult<bool> {
        Ok(self.0.metadata_is_fuzzy)
    }

    #[getter]
    fn path_or_content(&self) -> PyResult<String> {
        Ok(self.0.options.path_or_content.clone())
    }

    #[getter]
    fn wrapwidth(&self) -> PyResult<usize> {
        Ok(self.0.options.wrapwidth)
    }

    #[getter]
    fn bytes_content(&self) -> PyResult<Option<Vec<u8>>> {
        Ok(self.0.options.byte_content.clone())
    }

    fn save(&self, path: &str) -> PyResult<()> {
        self.0.save(path);
        Ok(())
    }

    fn save_as_pofile(&self, path: &str) -> PyResult<()> {
        self.0.save_as_pofile(path);
        Ok(())
    }

    fn save_as_mofile(&self, path: &str) -> PyResult<()> {
        self.0.save_as_mofile(path);
        Ok(())
    }

    fn remove_by_msgid(&mut self, msgid: &str) -> PyResult<()> {
        self.0.remove_by_msgid(msgid);
        Ok(())
    }

    fn remove_by_msgid_msgctxt(
        &mut self,
        msgid: &str,
        msgctxt: &str,
    ) -> PyResult<()> {
        self.0.remove_by_msgid_msgctxt(msgid, msgctxt);
        Ok(())
    }

    fn find_by_msgid(
        &self,
        msgid: &str,
    ) -> PyResult<Option<PyPOEntry>> {
        match self.0.find_by_msgid(msgid) {
            Some(entry) => Ok(Some(PyPOEntry::from(&entry))),
            None => Ok(None),
        }
    }

    fn find_by_msgid_msgctxt(
        &self,
        msgid: &str,
        msgctxt: &str,
    ) -> PyResult<Option<PyPOEntry>> {
        match self.0.find_by_msgid_msgctxt(msgid, msgctxt) {
            Some(entry) => Ok(Some(PyPOEntry::from(&entry))),
            None => Ok(None),
        }
    }

    fn percent_translated(&self) -> PyResult<f32> {
        Ok(self.0.percent_translated())
    }

    fn translated_entries(&self) -> PyResult<Vec<PyPOEntry>> {
        let mut entries = Vec::new();
        for entry in self.0.translated_entries() {
            entries.push(PyPOEntry::from(entry));
        }
        Ok(entries)
    }

    fn untranslated_entries(&self) -> PyResult<Vec<PyPOEntry>> {
        let mut entries = Vec::new();
        for entry in self.0.untranslated_entries() {
            entries.push(PyPOEntry::from(entry));
        }
        Ok(entries)
    }

    fn obsolete_entries(&self) -> PyResult<Vec<PyPOEntry>> {
        let mut entries = Vec::new();
        for entry in self.0.obsolete_entries() {
            entries.push(PyPOEntry::from(entry));
        }
        Ok(entries)
    }

    fn fuzzy_entries(&self) -> PyResult<Vec<PyPOEntry>> {
        let mut entries = Vec::new();
        for entry in self.0.fuzzy_entries() {
            entries.push(PyPOEntry::from(entry));
        }
        Ok(entries)
    }

    fn metadata_as_entry(&self) -> PyResult<PyPOEntry> {
        Ok(PyPOEntry::from(&self.0.metadata_as_entry()))
    }

    fn as_bytes_with(
        &self,
        magic_number: u32,
        revision_number: u32,
    ) -> PyResult<Vec<u8>> {
        Ok(MOFile::from(&self.0)
            .as_bytes_with(magic_number, revision_number))
    }

    fn as_bytes(&self) -> PyResult<Vec<u8>> {
        Ok(MOFile::from(&self.0).as_bytes())
    }

    fn as_bytes_le(&self) -> PyResult<Vec<u8>> {
        Ok(MOFile::from(&self.0).as_bytes_le())
    }

    fn as_bytes_be(&self) -> PyResult<Vec<u8>> {
        Ok(MOFile::from(&self.0).as_bytes_be())
    }

    fn append(&mut self, entry: &PyPOEntry) -> PyResult<()> {
        self.0.entries.push(entry._inner());
        Ok(())
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.0.entries.len())
    }

    fn __contains__(&self, entry: &PyPOEntry) -> PyResult<bool> {
        Ok(self
            .0
            .find_by_msgid_msgctxt(
                &entry._inner().msgid,
                &entry._inner().msgctxt.unwrap_or("".to_string()),
            )
            .is_some())
    }

    fn __getitem__(&self, index: usize) -> PyResult<PyPOEntry> {
        match self.0.entries.get(index) {
            Some(entry) => Ok(PyPOEntry::from(entry)),
            None => Err(PyErr::new::<
                pyo3::exceptions::PyIndexError,
                _,
            >("Index out of range")),
        }
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.0.to_string())
    }

    fn __eq__(&self, other: &PyPOFile) -> PyResult<bool> {
        Ok(self.0 == other.0)
    }

    fn __ne__(&self, other: &PyPOFile) -> PyResult<bool> {
        Ok(self.0 != other.0)
    }
}
