use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use crate::entry::{
    po_metadata_entry_to_string, POEntry, Translated,
};
use crate::errors::SyntaxError;
use crate::file::{
    metadata_hashmap_to_msgstr, mofile::MOFile, AsBytes, FileOptions,
    Save, SaveAsMOFile, SaveAsPOFile,
};
use crate::moparser::{MAGIC, MAGIC_SWAPPED};
use crate::poparser::POFileParser;
use crate::traits::Merge;

fn empty_msgctxt_predicate(_: &POEntry, _: &str) -> bool {
    true
}

fn msgctxt_predicate(entry: &POEntry, msgctxt: &str) -> bool {
    entry.msgctxt.as_ref().unwrap_or(&"".to_string()) == msgctxt
}

fn by_msgid_predicate(entry: &POEntry, value: &str) -> bool {
    entry.msgid == value
}

fn by_msgstr_predicate(entry: &POEntry, value: &str) -> bool {
    entry.msgstr.as_ref().unwrap_or(&"".to_string()) == value
}

fn by_msgctxt_predicate(entry: &POEntry, value: &str) -> bool {
    entry.msgctxt.as_ref().unwrap_or(&"".to_string()) == value
}

fn by_msgid_plural_predicate(entry: &POEntry, value: &str) -> bool {
    entry.msgid_plural.as_ref().unwrap_or(&"".to_string()) == value
}

fn by_previous_msgid_predicate(entry: &POEntry, value: &str) -> bool {
    entry.previous_msgid.as_ref().unwrap_or(&"".to_string()) == value
}

fn by_previous_msgid_plural_predicate(
    entry: &POEntry,
    value: &str,
) -> bool {
    entry
        .previous_msgid_plural
        .as_ref()
        .unwrap_or(&"".to_string())
        == value
}

fn by_previous_msgctxt_predicate(
    entry: &POEntry,
    value: &str,
) -> bool {
    entry.previous_msgctxt.as_ref().unwrap_or(&"".to_string())
        == value
}

/// PO files factory function.
///
/// It takes an argument that could be either:
///
/// * A string as path to an existent file.
/// * The content of a PO file as string.
/// * The content of a PO file as bytes.
/// * A [FileOptions] struct.
///
/// # Examples
///
/// ## Open from path
///
/// ```rust
/// use rspolib::pofile;
///
/// let file = pofile("tests-data/obsoletes.po").unwrap();
/// ```
///
/// ## Open from content
///
/// ```rust
/// use rspolib::pofile;
///
/// let content = r#"#
/// msgid ""
/// msgstr ""
///
/// msgid "A message"
/// msgstr "Un mensaje"
/// "#;
///
/// let file = pofile(content).unwrap();
/// ```
///
/// ## Open from bytes
///
/// ```rust
/// use std::fs;
/// use rspolib::pofile;
///
/// let bytes_content = fs::read("tests-data/all.po").unwrap();
/// let file = pofile(bytes_content).unwrap();
/// ```
///
/// ## Tuples into [FileOptions]
///
/// ```rust
/// use rspolib::pofile;
///
/// // Wrap width
/// let file = pofile(("tests-data/all.po", 75)).unwrap();
/// ```
///
/// ## Explicitly from [FileOptions]
///
/// ```rust
/// use std::fs;
/// use rspolib::{pofile, FileOptions as POFileOptions};
///
/// let file = pofile(POFileOptions::default()).unwrap();
///
/// // Path or content
/// let opts = POFileOptions::from("tests-data/obsoletes.po");
/// let file = pofile(opts).unwrap();
///
/// // Wrap width
/// let opts = POFileOptions::from(("tests-data/all.po", 75));
/// let file = pofile(opts).unwrap();
/// ```
pub fn pofile<Opt>(options: Opt) -> Result<POFile, SyntaxError>
where
    Opt: Into<FileOptions>,
{
    let mut parser = POFileParser::new(options.into());
    parser.parse()?;
    Ok(parser.file)
}

/// PO file
#[derive(Clone, Debug, PartialEq)]
pub struct POFile {
    /// Entries of the file.
    pub entries: Vec<POEntry>,
    /// Header of the file, if any. Optionally defined
    /// in PO files before the first entry.
    pub header: Option<String>,
    /// First optional field of PO files that describes
    /// the metadata of the file stored as a hash map.
    pub metadata: HashMap<String, String>,
    /// Whether the metadata is marked with the `fuzzy`
    /// flag or not.
    pub metadata_is_fuzzy: bool,
    /// Options defined for the file. See [FileOptions].
    pub options: FileOptions,
}

impl POFile {
    pub fn new(options: FileOptions) -> Self {
        Self {
            options,
            header: None,
            metadata: HashMap::new(),
            metadata_is_fuzzy: false,
            entries: Vec::new(),
        }
    }

    /// Remove an entry from the file
    pub fn remove(&mut self, entry: &POEntry) {
        self.entries.retain(|e| e != entry);
    }

    /// Remove the first entry that has the same msgid
    pub fn remove_by_msgid(&mut self, msgid: &str) {
        self.entries.retain(|e| e.msgid != msgid);
    }

    /// Remove the first entry that has the same msgid and msgctxt
    pub fn remove_by_msgid_msgctxt(
        &mut self,
        msgid: &str,
        msgctxt: &str,
    ) {
        self.entries.retain(|e| {
            e.msgid != msgid
                || e.msgctxt.as_ref().unwrap_or(&"".to_string())
                    != msgctxt
        });
    }

    /// Find entries by a given field and value
    ///
    /// The field defined in the `by` argument can be one of:
    ///
    /// * `msgid`
    /// * `msgstr`
    /// * `msgctxt`
    /// * `msgid_plural`
    /// * `previous_msgid`
    /// * `previous_msgid_plural`
    /// * `previous_msgctxt`
    ///
    /// Passing the optional `msgctxt` argument the entry
    /// will also must match with the given context.
    ///
    /// If `include_obsolete_entries` is set to `true` the
    /// search will include obsolete entries.
    pub fn find(
        &self,
        value: &str,
        by: &str,
        msgctxt: Option<&str>,
        include_obsolete_entries: bool,
    ) -> Vec<&POEntry> {
        let mut entries: Vec<&POEntry> = Vec::new();

        let msgctxt_predicate: &dyn Fn(&POEntry, &str) -> bool =
            match msgctxt {
                Some(_) => &msgctxt_predicate,
                None => &empty_msgctxt_predicate,
            };

        let by_predicate: &dyn Fn(&POEntry, &str) -> bool = match by {
            "msgid" => &by_msgid_predicate,
            "msgstr" => &by_msgstr_predicate,
            "msgctxt" => &by_msgctxt_predicate,
            "msgid_plural" => &by_msgid_plural_predicate,
            "previous_msgid" => &by_previous_msgid_predicate,
            "previous_msgid_plural" => {
                &by_previous_msgid_plural_predicate
            }
            "previous_msgctxt" => &by_previous_msgctxt_predicate,
            _ => &|_: &POEntry, _: &str| false,
        };

        for entry in &self.entries {
            if !include_obsolete_entries && entry.obsolete {
                continue;
            }
            if by_predicate(entry, value)
                && msgctxt_predicate(entry, msgctxt.unwrap_or(""))
            {
                entries.push(entry);
            }
        }
        entries
    }

    /// Find an entry by his msgid
    pub fn find_by_msgid(&self, msgid: &str) -> Option<POEntry> {
        self.entries.iter().find(|e| e.msgid == msgid).cloned()
    }

    /// Find an entry by msgid and msgctxt
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

    /// Returns the percent of the entries translated in the file
    pub fn percent_translated(&self) -> f32 {
        let translated = self.translated_entries().len();
        let total = self.entries.len();
        if total == 0 {
            0.0
        } else {
            (translated as f32 / total as f32) * 100.0
        }
    }

    /// Returns references to the translated entries of the file
    pub fn translated_entries(&self) -> Vec<&POEntry> {
        let mut entries: Vec<&POEntry> = Vec::new();
        for entry in &self.entries {
            if entry.translated() {
                entries.push(entry);
            }
        }
        entries
    }

    /// Returns references to the untranslated entries of the file
    pub fn untranslated_entries(&self) -> Vec<&POEntry> {
        let mut entries: Vec<&POEntry> = Vec::new();
        for entry in &self.entries {
            if !entry.translated() {
                entries.push(entry);
            }
        }
        entries
    }

    /// Returns references to the obsolete entries of the file
    pub fn obsolete_entries(&self) -> Vec<&POEntry> {
        let mut entries: Vec<&POEntry> = Vec::new();
        for entry in &self.entries {
            if entry.obsolete {
                entries.push(entry);
            }
        }
        entries
    }

    /// Returns references to the fuzzy entries of the file
    pub fn fuzzy_entries(&self) -> Vec<&POEntry> {
        let mut entries: Vec<&POEntry> = Vec::new();
        for entry in &self.entries {
            if entry.fuzzy() && !entry.obsolete {
                entries.push(entry);
            }
        }
        entries
    }

    /// Returns the metadata of the file as an entry.
    ///
    /// This method is not really useful because the
    /// ``to_string()`` version will not be guaranteed to be
    /// correct.
    ///
    /// If you want to manipulate the metadata, change
    /// the content of the field `metadata` in the file.
    ///
    /// If you still want to render a metadata entry as
    /// a string, use the function [po_metadata_entry_to_string]:
    ///
    /// ```rust
    /// use rspolib::{
    ///     pofile,
    ///     po_metadata_entry_to_string,
    /// };
    ///
    /// let file = pofile("tests-data/metadata.po").unwrap();
    /// let entry = file.metadata_as_entry();
    /// let entry_str = po_metadata_entry_to_string(&entry, true);
    /// assert!(entry_str.starts_with("#, fuzzy\nmsgid \"\""));
    /// ```
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
}

impl fmt::Display for POFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ret: String = match self.header {
            Some(ref header) => {
                if header.is_empty() {
                    "#\n".to_string()
                } else {
                    let mut header_repr = String::new();
                    for line in header.lines() {
                        if line.is_empty() {
                            header_repr.push_str("#\n");
                        } else {
                            header_repr.reserve(line.len() + 3);
                            header_repr.push_str("# ");
                            header_repr.push_str(line);
                            header_repr.push('\n');
                        }
                    }
                    header_repr
                }
            }
            None => "#\n".to_string(),
        };

        // Metadata should not include spaces after values
        ret.push_str(&po_metadata_entry_to_string(
            &self.metadata_as_entry(),
            self.metadata_is_fuzzy,
        ));
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
        write!(f, "{ret}")
    }
}

// Method `save_as_pofile` is implemented in the trait
impl SaveAsPOFile for POFile {}

impl Save for POFile {
    /// Save the PO file as the given path
    fn save(&self, path: &str) {
        self.save_as_pofile(path);
    }
}

impl SaveAsMOFile for POFile {
    /// Save the PO file as a MO file as the given path
    fn save_as_mofile(&self, path: &str) {
        MOFile::from(self).save(path);
    }
}

impl<'a> From<&'a str> for POFile {
    fn from(path_or_content: &'a str) -> Self {
        pofile(path_or_content).unwrap()
    }
}

impl Merge for POFile {
    /// Merge another PO file into this one and return a new one
    ///
    /// Recursively calls `merge` on each entry if they are found
    /// in the current file searching by msgid and msgctxt. If not
    /// found, generates a new entry.
    ///
    /// This method is commonly used to merge a POT reference file
    /// with a PO file.
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

impl AsBytes for POFile {
    /// Return the PO file content as a bytes vector of the MO file version
    ///
    /// The MO file is encoded with little
    /// endian magic number and revision number 0
    ///
    /// Use directly [MOFile::as_bytes_with] to customize
    /// the magic number and revision number:
    ///
    /// ```rust
    /// use rspolib::{pofile, MAGIC_SWAPPED, MOFile};
    ///
    /// let file = pofile("tests-data/all.po").unwrap();
    /// let bytes = MOFile::from(&file).as_bytes_with(MAGIC_SWAPPED, 1);
    /// ```
    fn as_bytes(&self) -> Cow<'_, [u8]> {
        let mofile = MOFile::from(self);
        let result = mofile.as_bytes_with(MAGIC, 0);
        Cow::Owned(result.into_owned())
    }

    /// Return the PO file content as a bytes vector of the MO file version
    ///
    /// Just an alias for [POFile::as_bytes], for consistency with [MOFile].
    fn as_bytes_le(&self) -> Cow<'_, [u8]> {
        self.as_bytes()
    }

    /// Return the PO file content as a bytes vector of
    /// the MO file version with big endianess
    fn as_bytes_be(&self) -> Cow<'_, [u8]> {
        let mofile = MOFile::from(self);
        let result = mofile.as_bytes_with(MAGIC_SWAPPED, 0);
        Cow::Owned(result.into_owned())
    }
}

impl From<Vec<&POEntry>> for POFile {
    fn from(entries: Vec<&POEntry>) -> Self {
        let mut file = POFile::new("".into());
        for entry in entries {
            file.entries.push(entry.clone());
        }
        file
    }
}

impl From<&Path> for POFile {
    fn from(path: &std::path::Path) -> Self {
        POFile::from(path.to_str().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::mofile::mofile;
    use std::fs;
    use std::path::Path;
    use unicode_width::UnicodeWidthStr;

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
            "#
msgid \"\"
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
            let width = UnicodeWidthStr::width(line);
            assert!(width <= file.options.wrapwidth + 2);
        }
    }

    fn pofile_save_test(save_fn_name: &str, fname: &str) {
        let tmpdir = "tests-data/tests";

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
        let tmpdir = "tests-data/tests";

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

    #[test]
    fn format_fuzzy_metadata() {
        let path = "tests-data/fuzzy-header.po";

        let file = pofile(path).unwrap();
        let expected_start = concat!(
            "# Po file with\n# a fuzzy header\n#, fuzzy\n",
            "msgid \"\"\nmsgstr \"\"\n\"Project-Id-Version:",
        );
        assert!(file.to_string().starts_with(expected_start));
    }

    #[test]
    fn format_comment_ordering() {
        let path = "tests-data/comment-ordering.po";

        let file = pofile(path).unwrap();
        let expected_content = r#"#
msgid ""
msgstr ""
"Content-Type: text/plain; charset=UTF-8\n"

# First comment line
#. Second comment line
msgid "foo"
msgstr "oof"
"#;
        assert_eq!(file.to_string(), expected_content);
    }

    #[test]
    fn remove() {
        let mut entry_1 = POEntry::from("msgid 1");
        entry_1.msgstr = Some("msgstr 1".to_string());

        let mut entry_2 = POEntry::from("msgid 2");
        entry_2.msgstr = Some("msgstr 2".to_string());

        let mut file = POFile::from(vec![&entry_1, &entry_2]);
        assert_eq!(file.entries.len(), 2);

        // remove by entry
        file.remove(&entry_1);

        assert_eq!(file.entries.len(), 1);
        assert_eq!(file.entries[0].msgid, "msgid 2");

        file.entries.push(entry_1);
        assert_eq!(file.entries.len(), 2);

        // remove by msgid
        file.remove_by_msgid("msgid 2");
        assert_eq!(file.entries.len(), 1);
        assert_eq!(file.entries[0].msgid, "msgid 1");

        // remove by msgid and msgctxt
        entry_2.msgctxt = Some("msgctxt 2".to_string());
        entry_2.msgid = "msgid 1".to_string();
        file.entries.push(entry_2);
        assert_eq!(file.entries.len(), 2);
        file.remove_by_msgid_msgctxt("msgid 1", "msgctxt 2");

        assert_eq!(file.entries.len(), 1);
        assert_eq!(file.entries[0].msgid, "msgid 1");
        assert_eq!(
            file.entries[0].msgstr.as_ref().unwrap(),
            "msgstr 1",
        );
    }

    #[test]
    fn find() {
        let mut entry_1 = POEntry::new(0);
        entry_1.msgid = "msgid 1".to_string();
        entry_1.msgstr = Some("msgstr 1".to_string());

        let mut entry_2 = POEntry::new(3);
        entry_2.msgid = "msgid 2".to_string();
        entry_2.msgstr = Some("msgstr 2".to_string());

        let mut file = POFile::from(vec![&entry_1, &entry_2]);
        assert_eq!(file.entries.len(), 2);

        // find by msgid
        assert_eq!(
            file.find_by_msgid("msgid 2").unwrap().msgid,
            "msgid 2"
        );

        // find by msgid and msgctxt
        entry_2.msgctxt = Some("msgctxt 2".to_string());
        entry_2.msgid = "msgid 1".to_string();
        file.entries.push(entry_2);
        assert_eq!(file.entries.len(), 3);
        assert_eq!(
            file.find_by_msgid_msgctxt("msgid 1", "msgctxt 2")
                .unwrap()
                .msgstr
                .as_ref()
                .unwrap(),
            "msgstr 2",
        );

        // find by msgid_plural, msgctxt, msgid...
        let mut entry_3 = POEntry::new(6);
        entry_3.msgid = "msgid for msgid_plural 1".to_string();
        entry_3.msgid_plural = Some("msgid_plural 1".to_string());
        entry_3.msgctxt =
            Some("msgctxt for msgid_plural 1".to_string());
        file.entries.push(entry_3);

        let mut entry_4 = POEntry::new(6);
        entry_4.msgid = "msgid for msgid_plural 1".to_string();
        entry_4.msgid_plural = Some("msgid_plural 1".to_string());
        entry_4.msgctxt = Some("other_msgctxt".to_string());
        file.entries.push(entry_4);

        let entries = file.find(
            "msgid for msgid_plural 1",
            "msgid",
            None,
            false,
        );
        assert_eq!(entries.len(), 2);

        let entries = file.find(
            "msgid for msgid_plural 1",
            "msgid",
            Some("msgctxt for msgid_plural 1"),
            false,
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].msgctxt.as_ref().unwrap(),
            "msgctxt for msgid_plural 1"
        );
    }

    #[test]
    fn parse_escapes_are_unescaped_on_format() {
        let path = "tests-data/escapes.po";
        let file = pofile(path).unwrap();

        let expected_content =
            "\\ \t \r \u{8} \n \\\n \u{11} \u{12} \\\\";

        assert_eq!(file.entries.len(), 1);
        assert_eq!(file.entries[0].msgid, expected_content);
        assert_eq!(
            file.entries[0].msgstr.as_ref().unwrap(),
            expected_content,
        );
    }

    #[test]
    fn parse_and_format_escapes() {
        let path = "tests-data/escapes.po";
        let out_path = "tests-data/tests/parse_and_format_escapes.po";

        let file = pofile(path).unwrap();
        file.save(out_path);

        let escapes_content = fs::read_to_string(path).unwrap();
        let out_content = fs::read_to_string(out_path).unwrap();

        assert_eq!(
            escapes_content.replace("\r\n", "\n"),
            out_content
        );
    }
}
