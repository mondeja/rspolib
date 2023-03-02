use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Write;

use crate::entry::{
    mo_metadata_entry_to_string, MOEntry, POEntry, Translated,
};
use crate::errors::SyntaxError;
use crate::file::{
    metadata_hashmap_to_msgstr, mofile::MOFile, AsBytes, Options,
    Save, SaveAsMOFile, SaveAsPOFile,
};
use crate::moparser::{MAGIC, MAGIC_SWAPPED};
use crate::poparser::POFileParser;
use crate::traits::Merge;

pub fn pofile<'a, Opt>(
    options: Opt,
) -> Result<POFile<'a>, SyntaxError>
where
    Opt: Into<Options<'a>>,
{
    let mut parser = POFileParser::new(options.into());
    parser.parse()?;
    Ok(parser.file)
}

pub struct POFile<'a> {
    pub header: Option<String>,
    pub metadata: HashMap<String, String>,
    pub metadata_is_fuzzy: bool,
    pub entries: Vec<POEntry>,
    pub options: Options<'a>,
}

impl<'a> POFile<'a> {
    pub fn new(options: Options<'a>) -> Self {
        Self {
            options,
            header: None,
            metadata: HashMap::new(),
            metadata_is_fuzzy: false,
            entries: Vec::new(),
        }
    }

    pub fn remove(&mut self, entry: &POEntry) {
        // Remove only the first occurrence
        if let Some(index) =
            self.entries.iter().position(|e| e == entry)
        {
            self.entries.remove(index);
        }
    }

    pub fn find_by_msgid(&self, msgid: &str) -> Option<POEntry> {
        self.entries.iter().find(|e| e.msgid == msgid).cloned()
    }

    pub fn find_by_msgid_msgctxt(
        &self,
        msgid: &str,
        msgctxt: &str,
    ) -> Option<POEntry> {
        self.entries
            .iter()
            .find(|e| {
                e.msgid == msgid
                    && e.msgctxt.as_ref().unwrap_or(&"".to_string())
                        == msgctxt
            })
            .cloned()
    }

    pub fn percent_translated(&self) -> f32 {
        let translated = self.translated_entries().len();
        let total = self.entries.len();
        if total == 0 {
            0.0
        } else {
            (translated as f32 / total as f32) * 100.0
        }
    }

    pub fn translated_entries(&self) -> Vec<&POEntry> {
        let mut entries: Vec<&POEntry> = Vec::new();
        for entry in &self.entries {
            if entry.translated() {
                entries.push(entry);
            }
        }
        entries
    }

    pub fn untranslated_entries(&self) -> Vec<&POEntry> {
        let mut entries: Vec<&POEntry> = Vec::new();
        for entry in &self.entries {
            if !entry.translated() {
                entries.push(entry);
            }
        }
        entries
    }

    pub fn obsolete_entries(&self) -> Vec<&POEntry> {
        let mut entries: Vec<&POEntry> = Vec::new();
        for entry in &self.entries {
            if entry.obsolete {
                entries.push(entry);
            }
        }
        entries
    }

    pub fn fuzzy_entries(&self) -> Vec<&POEntry> {
        let mut entries: Vec<&POEntry> = Vec::new();
        for entry in &self.entries {
            if entry.fuzzy() && !entry.obsolete {
                entries.push(entry);
            }
        }
        entries
    }

    pub fn metadata_as_entry(&self) -> POEntry {
        let mut entry = POEntry::new(0);
        if self.metadata_is_fuzzy {
            entry.flags.push("fuzzy".to_string());
        }

        if !self.metadata.is_empty() {
            entry.msgstr =
                Some(metadata_hashmap_to_msgstr(&self.metadata))
        }

        entry
    }

    pub fn save(&self, path: &str) {
        let mut file = File::create(path).unwrap();
        file.write_all(self.to_string().as_bytes()).ok();
    }

    pub fn save_as_pofile(&self, path: &str) {
        self.save(path);
    }
}

impl<'a> fmt::Display for POFile<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ret = match self.header {
            Some(ref header) => {
                let mut header_repr = String::new();
                for line in header.lines() {
                    if line.is_empty() {
                        header_repr.push_str("#\n");
                    } else {
                        header_repr.push_str("# ");
                        header_repr.push_str(line);
                        header_repr.push('\n');
                    }
                }
                header_repr
            }
            None => "".to_string(),
        };

        // Metadata should not include spaces after values
        ret.push_str(&mo_metadata_entry_to_string(&MOEntry::from(
            &self.metadata_as_entry(),
        )));
        ret.push('\n');

        let mut entries_ret = String::new();
        let mut obsolete_entries_ret = String::new();
        for entry in &self.entries {
            if entry.obsolete {
                obsolete_entries_ret.push_str(&entry.to_string());
                obsolete_entries_ret.push('\n');
            } else {
                entries_ret.push_str(&entry.to_string());
                entries_ret.push('\n');
            }
        }
        ret.push_str(&entries_ret);
        ret.push_str(&obsolete_entries_ret);
        ret.pop();
        write!(f, "{}", ret)
    }
}

impl<'a> SaveAsPOFile for POFile<'a> {}

impl<'a> Save for POFile<'a> {
    fn save(&self, path: &str) {
        self.save_as_pofile(path);
    }
}

impl<'a> SaveAsMOFile for POFile<'a> {
    fn save_as_mofile(&self, path: &str) {
        MOFile::from(self).save(path);
    }
}

impl<'a> From<&'a str> for POFile<'a> {
    fn from(path_or_content: &'a str) -> Self {
        pofile(path_or_content).unwrap()
    }
}

impl<'a> Merge for POFile<'a> {
    fn merge(&mut self, other: POFile) {
        for other_entry in other.entries.as_slice() {
            let entry: Option<POEntry> = match other_entry.msgctxt {
                Some(ref msgctxt) => self.find_by_msgid_msgctxt(
                    &other_entry.msgid,
                    msgctxt,
                ),
                None => self.find_by_msgid(&other_entry.msgid),
            };

            if let Some(e) = entry {
                let mut entry = e;
                entry.merge(other_entry.clone());
            } else {
                let mut entry = POEntry::new(0);
                entry.merge(other_entry.clone());
                self.entries.push(entry);
            }
        }

        let self_entries: &mut Vec<POEntry> = self.entries.as_mut();
        for entry in self_entries {
            if other.find_by_msgid(&entry.msgid).is_none() {
                entry.obsolete = true;
            }
        }
    }
}

impl<'a> AsBytes for POFile<'a> {
    fn as_bytes(&self) -> Vec<u8> {
        MOFile::from(self).as_bytes_with(MAGIC, 0)
    }

    fn as_bytes_le(&self) -> Vec<u8> {
        MOFile::from(self).as_bytes_with(MAGIC, 0)
    }

    fn as_bytes_be(&self) -> Vec<u8> {
        MOFile::from(self).as_bytes_with(MAGIC_SWAPPED, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::mofile::mofile;
    use std::env;
    use std::fs;
    use std::path::Path;
    use unicode_segmentation::UnicodeSegmentation;

    #[test]
    fn pofile_test() {
        let path = "tests-data/all.po";
        let file = pofile(path).unwrap();

        assert_eq!(file.entries.len(), 9);
    }

    #[test]
    fn pofile_metadata_as_entry() {
        // File with metadata
        let path = "tests-data/all.po";
        let file = pofile(path).unwrap();
        let entry = file.metadata_as_entry();

        assert_eq!(entry.msgid, "");
        assert_eq!(entry.msgstr.unwrap().lines().count(), 11);

        // File without metadata
        let path = "tests-data/empty-metadata.po";
        let file = pofile(path).unwrap();
        let entry = file.metadata_as_entry();

        assert_eq!(entry.msgid, "");
        assert_eq!(entry.msgstr.is_none(), true);

        // File with fuzzy metadata
        let path = "tests-data/fuzzy-header.po";
        let file = pofile(path).unwrap();
        let entry = file.metadata_as_entry();

        assert_eq!(entry.msgid, "");
        assert_eq!(entry.fuzzy(), true);
        assert_eq!(entry.msgstr.unwrap().lines().count(), 12);
    }

    #[test]
    fn metadata_keys_are_natural_sorted() {
        let path = "tests-data/natural-unsorted-metadata.po";
        let file = pofile(path).unwrap();

        file.save("foobar-2-out.po");
        assert_eq!(
            file.to_string(),
            "msgid \"\"
msgstr \"\"
\"Project-Id-Version: PACKAGE VERSION\\n\"
\"Report-Msgid-Bugs-To: \\n\"
\"Language-Team: LANGUAGE <LL@li.org>\\n\"
\"Content-Type: text/plain; charset=UTF-8\\n\"
\"Content-Transfer-Encoding: 8bit\\n\"
\"X-Poedit-SearchPath-1: Foo\\n\"
\"X-Poedit-SearchPath-2: Bar\\n\"
\"X-Poedit-SearchPath-10: Baz\\n\"
",
        );
    }

    #[test]
    fn mofile_from_pofile() {
        let path = "tests-data/all.po";
        let po_file = pofile(path).unwrap();
        let mo_file = MOFile::from(&po_file);

        assert_eq!(
            mo_file.entries.len(),
            po_file.translated_entries().len(),
        );
        assert_eq!(mo_file.metadata.len(), po_file.metadata.len());
    }

    #[test]
    fn pofile_percent_translated() {
        let path = "tests-data/2-translated-entries.po";
        let file = pofile(path).unwrap();

        assert_eq!(file.percent_translated(), 40 as f32);
    }

    #[test]
    fn pofile_translated_entries() {
        let path = "tests-data/2-translated-entries.po";
        let file = pofile(path).unwrap();

        let translated_entries = file.translated_entries();
        assert_eq!(file.entries.len(), 5);
        assert_eq!(translated_entries.len(), 2);
        assert_eq!(file.entries[0].msgid, "msgid 1");
        assert_eq!(translated_entries[0].msgid, "msgid 2");
    }

    #[test]
    fn pofile_untranslated_entries() {
        let path = "tests-data/2-translated-entries.po";
        let file = pofile(path).unwrap();

        let untranslated_entries = file.untranslated_entries();
        assert_eq!(file.entries.len(), 5);
        assert_eq!(untranslated_entries.len(), 3);
        assert_eq!(file.entries[0].msgid, "msgid 1");
        assert_eq!(untranslated_entries[0].msgid, "msgid 1");
        assert_eq!(untranslated_entries[1].msgid, "msgid 3");
    }

    #[test]
    fn pofile_obsolete_entries() {
        let path = "tests-data/obsoletes.po";
        let file = pofile(path).unwrap();

        let obsolete_entries = file.obsolete_entries();
        assert_eq!(file.entries.len(), 3);
        assert_eq!(obsolete_entries.len(), 2);
    }

    #[test]
    fn pofile_to_string() {
        let po_path = "tests-data/all.po";
        let file = pofile(po_path).unwrap();

        let file_as_string = file.to_string();

        for line in file_as_string.lines() {
            let n_chars = line.graphemes(true).count();
            assert!(n_chars <= file.options.wrapwidth + 2);
        }
    }

    fn pofile_save_test(save_fn_name: &str, fname: &str) {
        let tmpdir = env::temp_dir();

        let path = "tests-data/all.po";
        let file = pofile(path).unwrap();
        let file_as_string = file.to_string();

        // Here the file name is parametrized to avoid data races
        // when running tests in parallel
        let tmp_path = Path::new(&tmpdir).join(fname);
        let tmp_path_str = tmp_path.to_str().unwrap();

        if save_fn_name == "save" {
            file.save(tmp_path_str);
        } else {
            file.save_as_pofile(tmp_path_str);
        }

        assert_eq!(
            file_as_string,
            fs::read_to_string(tmp_path_str).unwrap()
        );
        fs::remove_file(tmp_path_str).ok();
    }

    #[test]
    fn pofile_save() {
        pofile_save_test("save", "all-1.po")
    }

    #[test]
    fn pofile_save_as_pofile() {
        pofile_save_test("save_as_pofile", "all-2.po")
    }

    #[test]
    fn pofile_save_as_mofile() {
        let tmpdir = env::temp_dir();

        let content =
            concat!("msgid \"foo bar\"\n", "msgstr \"foo bar\"\n",);
        let po_file = pofile(content).unwrap();

        let tmp_path = Path::new(&tmpdir)
            .join("pofile_save_as_mofile-simple.mo");
        let tmp_path_str = tmp_path.to_str().unwrap();
        po_file.save_as_mofile(tmp_path_str);

        assert!(tmp_path.exists());

        let mo_file = mofile(tmp_path_str).unwrap();
        assert_eq!(mo_file.entries.len(), po_file.entries.len());
        assert_eq!(mo_file.metadata.len(), po_file.metadata.len());

        assert_eq!(mo_file.entries[0].msgid, "foo bar");
        assert_eq!(
            mo_file.entries[0].msgstr.as_ref().unwrap(),
            "foo bar"
        );
    }

    #[test]
    fn set_fuzzy() {
        let path = "tests-data/fuzzy-no-fuzzy.po";

        let mut file = pofile(path).unwrap();

        assert!(!file.entries[0].fuzzy());
        assert!(file.entries[1].fuzzy());

        // set fuzzy
        file.entries[0].flags.push("fuzzy".to_string());

        // unset fuzzy
        let fuzzy_position = file.entries[1]
            .flags
            .iter()
            .position(|p| p == "fuzzy")
            .unwrap();
        file.entries[1].flags.remove(fuzzy_position);

        assert!(file.entries[0].fuzzy());
        assert!(!file.entries[1].fuzzy());

        assert_eq!(
            file.entries[0].to_string(),
            "#, fuzzy\nmsgid \"a\"\nmsgstr \"a\"\n",
        );
        assert_eq!(
            file.entries[1].to_string(),
            "msgid \"Line\"\nmsgstr \"Ligne\"\n",
        );
    }
}
