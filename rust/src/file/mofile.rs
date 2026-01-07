use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::entry::{
    mo_metadata_entry_to_string, MOEntry, MsgidEotMsgctxt,
};
use crate::errors::IOError;
use crate::file::{
    metadata_hashmap_to_msgstr, pofile::POFile, AsBytes, FileOptions,
    Save, SaveAsMOFile, SaveAsPOFile,
};
use crate::moparser::{MOFileParser, MAGIC, MAGIC_SWAPPED};

fn empty_msgctxt_predicate(_: &MOEntry, _: &str) -> bool {
    true
}

fn msgctxt_predicate(entry: &MOEntry, msgctxt: &str) -> bool {
    entry.msgctxt.as_ref().unwrap_or(&"".to_string()) == msgctxt
}

fn by_msgid_predicate(entry: &MOEntry, value: &str) -> bool {
    entry.msgid == value
}

fn by_msgstr_predicate(entry: &MOEntry, value: &str) -> bool {
    entry.msgstr.as_ref().unwrap_or(&"".to_string()) == value
}

fn by_msgctxt_predicate(entry: &MOEntry, value: &str) -> bool {
    entry.msgctxt.as_ref().unwrap_or(&"".to_string()) == value
}

fn by_msgid_plural_predicate(entry: &MOEntry, value: &str) -> bool {
    entry.msgid_plural.as_ref().unwrap_or(&"".to_string()) == value
}

/// MO files factory function
///
/// Read a MO file from a path, parse from content as bytes or
/// from a [FileOptions] struct.
///
/// # Examples
///
/// ## Read a MO file from a path
///
/// ```rust
/// use rspolib::mofile;
///
/// let file = mofile("tests-data/all.mo").unwrap();
/// assert_eq!(file.entries.len(), 7);
/// ```
///
/// ## Read a MO file from bytes
///
/// ```rust
/// use rspolib::mofile;
///
/// let bytes = std::fs::read("tests-data/all.mo").unwrap();
/// let file = mofile(bytes).unwrap();
/// assert_eq!(file.entries.len(), 7);
/// ```
pub fn mofile<Opt>(options: Opt) -> Result<MOFile, IOError>
where
    Opt: Into<FileOptions>,
{
    let mut parser = MOFileParser::new(options.into());
    parser.parse()?;
    Ok(parser.file)
}

/// MO file
#[derive(Clone, Debug, PartialEq)]
pub struct MOFile {
    /// Magic number, either [MAGIC] or [MAGIC_SWAPPED]
    pub magic_number: Option<u32>,
    /// Version number, either 0 or 1
    pub version: Option<u32>,
    /// Metadata as a hash map
    pub metadata: HashMap<String, String>,
    /// Message entries
    pub entries: Vec<MOEntry>,
    /// File options. See [FileOptions].
    pub options: FileOptions,
}

impl MOFile {
    pub fn new(options: FileOptions) -> Self {
        Self {
            options,
            magic_number: None,
            version: None,
            metadata: HashMap::new(),
            entries: Vec::new(),
        }
    }

    /// Returns the metadata as a [MOEntry]
    pub fn metadata_as_entry(&self) -> MOEntry {
        let mut entry =
            MOEntry::new("".to_string(), None, None, vec![], None);
        if !self.metadata.is_empty() {
            entry.msgstr =
                Some(metadata_hashmap_to_msgstr(&self.metadata))
        }

        entry
    }

    /// Find entries by a given field and value
    ///
    /// The field defined in the `by` argument can be one of:
    ///
    /// * `msgid`
    /// * `msgstr`
    /// * `msgctxt`
    /// * `msgid_plural`
    ///
    /// Passing the optional `msgctxt` argument the entry
    /// will also must match with the given context.
    pub fn find(
        &self,
        value: &str,
        by: &str,
        msgctxt: Option<&str>,
    ) -> Vec<&MOEntry> {
        let mut entries: Vec<&MOEntry> = Vec::new();

        let msgctxt_predicate: &dyn Fn(&MOEntry, &str) -> bool =
            match msgctxt {
                Some(_) => &msgctxt_predicate,
                None => &empty_msgctxt_predicate,
            };

        let by_predicate: &dyn Fn(&MOEntry, &str) -> bool = match by {
            "msgid" => &by_msgid_predicate,
            "msgstr" => &by_msgstr_predicate,
            "msgctxt" => &by_msgctxt_predicate,
            "msgid_plural" => &by_msgid_plural_predicate,
            _ => &|_: &MOEntry, _: &str| false,
        };

        for entry in &self.entries {
            if by_predicate(entry, value)
                && msgctxt_predicate(entry, msgctxt.unwrap_or(""))
            {
                entries.push(entry);
            }
        }
        entries
    }

    /// Find an entry by msgid
    pub fn find_by_msgid(&self, msgid: &str) -> Option<&MOEntry> {
        self.entries.iter().find(|e| e.msgid == msgid)
    }

    /// Find an entry by msgid and msgctxt
    pub fn find_by_msgid_msgctxt(
        &self,
        msgid: &str,
        msgctxt: &str,
    ) -> Option<&MOEntry> {
        self.entries.iter().find(|e| {
            e.msgid == msgid && e.msgctxt == Some(msgctxt.to_string())
        })
    }

    /// Remove an entry from the file
    pub fn remove(&mut self, entry: &MOEntry) {
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

    /// Returns the entry as a bytes vector
    ///
    /// Specify the magic number and the revision number
    /// of the generated MO version of the file.
    ///
    /// This method does not check the validity of the values
    /// `magic_number` and `revision_version` to allow the
    /// experimental developing of other revision of MO files,
    /// so be careful about the passed values if you use it.
    ///
    /// Valid values for the magic number are [MAGIC] and [MAGIC_SWAPPED].
    /// Valid values for the revision number are 0 and 1.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rspolib::mofile;
    ///
    /// let file = mofile("tests-data/all.mo").unwrap();
    /// let bytes = file.as_bytes_with(rspolib::MAGIC_SWAPPED, 1);
    /// assert_eq!(bytes.len(), 1327);
    /// ```
    pub fn as_bytes_with(
        &self,
        magic_number: u32,
        revision_number: u32,
    ) -> Cow<'_, [u8]> {
        let metadata_entry = self.metadata_as_entry();

        // Select byte order based on magic number
        let bytes_reader: fn(u32) -> [u8; 4] = match magic_number {
            MAGIC_SWAPPED => u32::to_be_bytes,
            _ => u32::to_le_bytes,
        };

        let mut entries: Vec<&MOEntry> = vec![&metadata_entry];
        entries.extend(&self.entries);
        entries.sort_unstable_by(|a, b| {
            a.msgid_eot_msgctxt().cmp(&b.msgid_eot_msgctxt())
        });
        let entries_length = entries.len();

        let mut offsets: Vec<(usize, usize, usize, usize)> = vec![];

        let mut ids = "".to_string();
        let mut strs = "".to_string();
        for e in entries {
            // For each string, we need size and file offset. Each
            // string is NUL terminated but the NUL does not count
            // into the size.
            let mut msgid = "".to_string();
            let mut msgstr = "".to_string();

            if let Some(msgctxt) = &e.msgctxt {
                msgid.push_str(msgctxt);
                msgid.push('\u{4}');
            }

            if let Some(msgid_plural) = &e.msgid_plural {
                // handle msgid_plural
                msgid.push_str(&e.msgid);
                msgid.push('\u{0}');
                msgid.push_str(msgid_plural);

                // handle msgstr_plural
                let msgstr_plural_length = &e.msgstr_plural.len();
                for (i, v) in e.msgstr_plural.iter().enumerate() {
                    msgstr.push_str(v);
                    if i < msgstr_plural_length - 1 {
                        msgstr.push('\u{0}');
                    }
                }
            } else {
                msgid.push_str(&e.msgid);
                if let Some(m) = &e.msgstr {
                    msgstr.push_str(m);
                }
            }

            offsets.push((
                ids.len(),
                msgid.len(),
                strs.len(),
                msgstr.len(),
            ));

            ids.push_str(&msgid);
            ids.push('\u{0}');
            strs.push_str(&msgstr);
            strs.push('\u{0}');
        }

        // The header is 7 32-bit unsigned integers.
        let keystart = 7 * 4 + 16 * entries_length;
        // and the values start after the keys
        let valuestart = keystart + ids.len();

        // The string table first has the list of keys, then the list of values.
        // Each entry has first the size of the string, then the file offset.
        let mut koffsets: Vec<(usize, usize)> = vec![];
        let mut voffsets: Vec<(usize, usize)> = vec![];
        for (o1, l1, o2, l2) in offsets {
            koffsets.push((l1, o1 + keystart));
            voffsets.push((l2, o2 + valuestart));
        }

        let mut final_offsets: Vec<u8> = vec![];
        for (l, o) in koffsets {
            final_offsets.extend(bytes_reader(l as u32));
            final_offsets.extend(bytes_reader(o as u32));
        }
        for (l, o) in voffsets {
            final_offsets.extend(bytes_reader(l as u32));
            final_offsets.extend(bytes_reader(o as u32));
        }

        let mut output: Vec<u8> = Vec::with_capacity(
            7 * 4 + 8 * entries_length + ids.len() + strs.len(),
        );
        // magic number
        output.extend(bytes_reader(MAGIC));
        // version
        output.extend(bytes_reader(revision_number));
        // number of entries
        output.extend(bytes_reader(entries_length as u32));
        // start of key index
        output.extend(bytes_reader(7 * 4));
        // start of value index
        output.extend(bytes_reader(
            7 * 4 + (entries_length as u32) * 8,
        ));
        // size and offset of hash table, we don't use hash tables
        output.extend([0, 0, 0, 0]);
        output.extend(bytes_reader(keystart as u32));

        output.extend(final_offsets);
        output.extend(ids.as_bytes());
        output.extend(strs.as_bytes());
        output.into()
    }
}

impl fmt::Display for MOFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ret = String::from("#\n");
        ret.push_str(&mo_metadata_entry_to_string(
            &self.metadata_as_entry(),
        ));

        ret.push('\n');
        for entry in &self.entries {
            ret.push_str(&entry.to_string());
            ret.push('\n');
        }
        ret.remove(ret.len() - 1);
        write!(f, "{ret}")
    }
}

// the method save_as_pofile is implemented by the trait
impl SaveAsPOFile for MOFile {}

impl Save for MOFile {
    /// Save the MOFile to a file at the given path
    fn save(&self, path: &str) {
        let mut file = File::create(path).unwrap();
        file.write_all(&self.as_bytes()).ok();
    }
}

impl SaveAsMOFile for MOFile {
    /// Save the MOFile to a file at the given path
    fn save_as_mofile(&self, path: &str) {
        self.save(path);
    }
}

impl AsBytes for MOFile {
    /// Return the MOFile as a vector of bytes in little endian
    fn as_bytes(&self) -> Cow<'_, [u8]> {
        self.as_bytes_with(MAGIC, 0)
    }

    /// Return the MOFile as a vector of bytes in little endian
    fn as_bytes_le(&self) -> Cow<'_, [u8]> {
        self.as_bytes_with(MAGIC, 0)
    }

    /// Return the MOFile as a vector of bytes in big endian
    fn as_bytes_be(&self) -> Cow<'_, [u8]> {
        self.as_bytes_with(MAGIC_SWAPPED, 0)
    }
}

impl From<&POFile> for MOFile {
    fn from(file: &POFile) -> MOFile {
        let mut new_file = MOFile::new(file.options.clone());
        new_file.metadata = file.metadata.clone();
        new_file.entries = file
            .translated_entries()
            .iter()
            .map(|e| MOEntry::from(*e))
            .collect();
        new_file
    }
}

impl From<Vec<&MOEntry>> for MOFile {
    fn from(entries: Vec<&MOEntry>) -> Self {
        let mut file = MOFile::new("".into());
        for entry in entries {
            file.entries.push(entry.clone());
        }
        file
    }
}

impl From<&Path> for MOFile {
    fn from(path: &Path) -> Self {
        MOFile::new(path.to_str().unwrap().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pofile;
    use std::fs;
    use std::io::Read;
    use std::path::Path;
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn mofile_test() {
        let path = "tests-data/all.mo";
        let file = mofile(path).unwrap();

        assert_eq!(file.entries.len(), 7);
    }

    #[test]
    fn mofile_metadata_as_entry() {
        // File with metadata
        let path = "tests-data/all.mo";
        let file = mofile(path).unwrap();
        let entry = file.metadata_as_entry();

        let msgstr = entry.msgstr.unwrap();
        assert_eq!(entry.msgid, "");
        assert_eq!(msgstr.lines().count(), 10);

        // File without metadata
        let path = "tests-data/empty-metadata.mo";
        let file = mofile(path).unwrap();
        let entry = file.metadata_as_entry();

        assert_eq!(entry.msgid, "");
        assert_eq!(entry.msgstr.is_none(), true);
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
    fn mofile_from_std_path() {
        let file = MOFile::from(Path::new("tests-data/all.mo"));
        assert_eq!(file.options.path_or_content, "tests-data/all.mo");
    }

    #[test]
    fn mofile_to_string() {
        let mo_path = "tests-data/all.mo";
        let file = mofile(mo_path).unwrap();

        let file_as_string = file.to_string();

        for line in file_as_string.lines() {
            let width = UnicodeWidthStr::width(line);
            assert!(width <= file.options.wrapwidth + 2);
        }
    }

    #[test]
    fn mofile_as_bytes() {
        // generated by msgfmt
        let path = "tests-data/all.mo";
        let file = mofile(path).unwrap();
        let file_as_bytes = file.as_bytes();

        // generated by polib
        let polib_path = "tests-data/all-polib.mo";
        let polib_file = mofile(polib_path).unwrap();
        let polib_file_as_bytes = polib_file.as_bytes();

        // The same number of bytes
        assert_eq!(file_as_bytes.len(), polib_file_as_bytes.len());
        // and the same bytes
        for (rspolib_byte, polib_byte) in
            file_as_bytes.iter().zip(polib_file_as_bytes.iter())
        {
            assert_eq!(rspolib_byte, polib_byte);
        }

        // the implementation differs from msgfmt
        let buffer: Vec<u8> = fs::read(path).unwrap();
        assert_ne!(file_as_bytes, buffer);
        assert_ne!(polib_file_as_bytes, buffer);
        // msgfmt generates more bytes
        assert!(file_as_bytes.len() < buffer.len());
        assert!(polib_file_as_bytes.len() < buffer.len());
    }

    #[test]
    fn mofile_save_as_pofile() {
        let tmpdir = "tests-data/tests";

        let path = "tests-data/all.mo";
        let file = mofile(path).unwrap();
        let file_as_string = file.to_string();

        let tmp_path = Path::new(&tmpdir).join("all.po");
        let tmp_path_str = tmp_path.to_str().unwrap();
        file.save_as_pofile(tmp_path_str);

        assert_eq!(
            file_as_string,
            fs::read_to_string(tmp_path_str).unwrap()
        );
        fs::remove_file(tmp_path_str).unwrap();
    }

    fn mofile_save_test(
        basename: &str,
        read_bytes_from_file: bool,
        save_method_name: &str,
    ) {
        let tmpdir = "tests-data/tests";

        let path = "tests-data/all.mo";
        let file = mofile(path).unwrap();

        let tmp_path =
            Path::new(&tmpdir).join(format!("{}.mo", basename));
        let tmp_path_str = tmp_path.to_str().unwrap();
        if save_method_name == "save" {
            file.save(tmp_path_str);
        } else {
            file.save_as_mofile(tmp_path_str);
        }

        // exists
        assert!(tmp_path.is_file());

        let file_bytes = match read_bytes_from_file {
            true => fs::read(tmp_path_str).unwrap(),
            false => file.as_bytes().into_owned(),
        };
        let mut file_bytes = file_bytes.as_slice();
        let mut buf: [u8; 4] = [0, 0, 0, 0];

        // has correct magic number
        file_bytes.read_exact(&mut buf).unwrap();
        let magic_number = u32::from_le_bytes(buf);
        assert_eq!(magic_number, MAGIC);

        // has correct revision number
        file_bytes.read_exact(&mut buf).unwrap();
        let revision_number = u32::from_le_bytes(buf);
        assert_eq!(revision_number, 0);

        // has correct number of entries
        file_bytes.read_exact(&mut buf).unwrap();
        let number_of_entries = u32::from_le_bytes(buf);
        assert_eq!(
            number_of_entries,
            // +1 here because includes the header entry
            file.entries.len() as u32 + 1,
        );
    }

    #[test]
    fn mofile_save_as_mofile() {
        mofile_save_test(
            "mofile_save_as_mofile-file",
            true,
            "save_as_mofile",
        );
        mofile_save_test(
            "mofile_save_as_mofile-struct",
            false,
            "save_as_mofile",
        );
    }

    #[test]
    fn mofile_save() {
        mofile_save_test("mofile_save-file", true, "save");
        mofile_save_test("mofile_save-struct", false, "save");
    }

    #[test]
    fn remove() {
        let mut entry_1 = MOEntry::from("msgid 1");
        entry_1.msgstr = Some("msgstr 1".to_string());

        let mut entry_2 = MOEntry::from("msgid 2");
        entry_2.msgstr = Some("msgstr 2".to_string());

        let mut file = MOFile::from(vec![&entry_1, &entry_2]);
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
        let mut entry_1 = MOEntry::from("msgid 1");
        entry_1.msgstr = Some("msgstr 1".to_string());

        let mut entry_2 = MOEntry::from("msgid 2");
        entry_2.msgstr = Some("msgstr 2".to_string());

        let mut file = MOFile::from(vec![&entry_1, &entry_2]);
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
    }
}
