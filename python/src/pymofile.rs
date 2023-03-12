use std::collections::HashMap;

use pyo3::prelude::*;

use crate::exceptions;
use crate::pymoentry::PyMOEntry;
use rspolib::{
    mofile, AsBytes, FileOptions, MOEntry, MOFile, Save,
    SaveAsMOFile, SaveAsPOFile, MAGIC, MAGIC_SWAPPED,
};

#[pyclass]
struct PyMOEntriesIter {
    inner: std::vec::IntoIter<MOEntry>,
}

#[pymethods]
impl PyMOEntriesIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyMOEntry> {
        slf.inner.next().map(|entry| PyMOEntry::from(&entry))
    }
}

#[pyfunction]
#[pyo3(name = "mofile")]
#[pyo3(signature = (path_or_content, wrapwidth=78))]
pub fn py_mofile(
    path_or_content: &str,
    wrapwidth: usize,
) -> PyResult<PyMOFile> {
    let result =
        mofile(FileOptions::from((path_or_content, wrapwidth)));
    match result {
        Ok(mofile) => Ok(PyMOFile(mofile)),
        Err(e) => {
            Err(PyErr::new::<exceptions::IOError, _>(e.to_string()))
        }
    }
}

#[pyclass]
#[pyo3(name = "MOFile")]
pub struct PyMOFile(MOFile);

#[pymethods]
impl PyMOFile {
    #[new]
    #[pyo3(signature = (path_or_content="", wrapwidth=78))]
    fn new(
        path_or_content: &str,
        wrapwidth: usize,
    ) -> PyResult<Self> {
        let result =
            mofile(FileOptions::from((path_or_content, wrapwidth)));
        match result {
            Ok(mofile) => Ok(PyMOFile(mofile)),
            Err(e) => Err(PyErr::new::<exceptions::IOError, _>(
                e.to_string(),
            )),
        }
    }

    #[allow(non_snake_case)]
    #[classattr]
    fn MAGIC() -> PyResult<u32> {
        Ok(MAGIC)
    }

    #[allow(non_snake_case)]
    #[classattr]
    fn MAGIC_SWAPPED() -> PyResult<u32> {
        Ok(MAGIC_SWAPPED)
    }

    #[getter]
    fn magic_number(&self) -> PyResult<u32> {
        Ok(self.0.magic_number.unwrap_or(MAGIC))
    }

    #[getter]
    fn version(&self) -> PyResult<u32> {
        Ok(self.0.version.unwrap_or(0))
    }

    fn get_entries(&self) -> PyResult<Vec<PyMOEntry>> {
        let mut entries = Vec::new();
        for entry in &self.0.entries {
            entries.push(PyMOEntry::from(entry));
        }
        Ok(entries)
    }

    #[setter]
    fn set_entries(&mut self, entries: Vec<PyMOEntry>) {
        let mut new_entries = Vec::new();
        for entry in entries {
            new_entries.push(entry._inner());
        }
        self.0.entries = new_entries;
    }

    fn get_metadata(&self) -> PyResult<HashMap<String, String>> {
        Ok(self.0.metadata.clone())
    }

    #[setter]
    fn set_metadata(&mut self, metadata: HashMap<String, String>) {
        self.0.metadata = metadata;
    }

    fn update_metadata(&mut self, metadata: HashMap<String, String>) {
        for (key, value) in metadata {
            self.0.metadata.insert(key, value);
        }
    }

    fn remove_metadata_field(&mut self, key: &str) {
        self.0.metadata.remove(key);
    }

    #[getter]
    fn path_or_content(&self) -> PyResult<String> {
        Ok(self.0.options.path_or_content.clone())
    }

    #[getter]
    fn wrapwidth(&self) -> PyResult<usize> {
        Ok(self.0.options.wrapwidth)
    }

    #[setter]
    fn set_wrapwidth(&mut self, wrapwidth: usize) {
        self.0.options.wrapwidth = wrapwidth;
    }

    #[getter]
    fn byte_content(&self) -> PyResult<Option<Vec<u8>>> {
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

    fn metadata_as_entry(&self) -> PyResult<PyMOEntry> {
        Ok(PyMOEntry::from(&self.0.metadata_as_entry()))
    }

    fn as_bytes_with(
        &self,
        magic_number: u32,
        revision_number: u32,
    ) -> PyResult<Vec<u8>> {
        Ok(self
            .0
            .as_bytes_with(magic_number, revision_number)
            .to_vec())
    }

    fn as_bytes(&self) -> PyResult<Vec<u8>> {
        Ok(self.0.as_bytes().to_vec())
    }

    fn as_bytes_le(&self) -> PyResult<Vec<u8>> {
        Ok(self.0.as_bytes_le().to_vec())
    }

    fn as_bytes_be(&self) -> PyResult<Vec<u8>> {
        Ok(self.0.as_bytes_be().to_vec())
    }

    fn remove(&mut self, entry: &PyMOEntry) -> PyResult<()> {
        self.0.remove(&entry._inner());
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

    fn append(&mut self, entry: &PyMOEntry) -> PyResult<()> {
        self.0.entries.push(entry._inner());
        Ok(())
    }

    #[pyo3(signature=(value, by="msgid", msgctxt=None))]
    fn find(
        &self,
        value: &str,
        by: &str,
        msgctxt: Option<&str>,
    ) -> PyResult<Vec<PyMOEntry>> {
        let mut entries: Vec<PyMOEntry> = vec![];
        for entry in self.0.find(value, by, msgctxt) {
            entries.push(PyMOEntry::from(entry));
        }
        Ok(entries)
    }

    fn find_by_msgid(
        &self,
        msgid: &str,
    ) -> PyResult<Option<PyMOEntry>> {
        match self.0.find_by_msgid(msgid) {
            Some(entry) => Ok(Some(PyMOEntry::from(entry))),
            None => Ok(None),
        }
    }

    fn find_by_msgid_msgctxt(
        &self,
        msgid: &str,
        msgctxt: &str,
    ) -> PyResult<Option<PyMOEntry>> {
        match self.0.find_by_msgid_msgctxt(msgid, msgctxt) {
            Some(entry) => Ok(Some(PyMOEntry::from(entry))),
            None => Ok(None),
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.0.entries.len())
    }

    fn __contains__(&self, entry: &PyMOEntry) -> PyResult<bool> {
        Ok(match entry._inner().msgctxt {
            Some(msgctxt) => self
                .0
                .find_by_msgid_msgctxt(
                    &entry._inner().msgid,
                    &msgctxt,
                )
                .is_some(),
            None => {
                self.0.find_by_msgid(&entry._inner().msgid).is_some()
            }
        })
    }

    fn __getitem__(&self, index: usize) -> PyResult<PyMOEntry> {
        match self.0.entries.get(index) {
            Some(entry) => Ok(PyMOEntry::from(entry)),
            None => Err(PyErr::new::<
                pyo3::exceptions::PyIndexError,
                _,
            >("Index out of range")),
        }
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.0.to_string())
    }

    fn __eq__(&self, other: &PyMOFile) -> PyResult<bool> {
        Ok(self.0.to_string() == other.0.to_string())
    }

    fn __ne__(&self, other: &PyMOFile) -> PyResult<bool> {
        Ok(self.0.to_string() != other.0.to_string())
    }

    fn __iter__(
        slf: PyRef<'_, Self>,
    ) -> PyResult<Py<PyMOEntriesIter>> {
        let iter = PyMOEntriesIter {
            inner: slf.0.entries.clone().into_iter(),
        };
        Py::new(slf.py(), iter)
    }
}
