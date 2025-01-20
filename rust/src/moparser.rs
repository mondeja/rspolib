use std::borrow::Cow;
use std::fs::File;
use std::io::{Cursor, SeekFrom};
use std::path::Path;

use crate::entry::MOEntry;
use crate::errors::IOError;
use crate::file::{mofile::MOFile, FileOptions};
use crate::traits::SeekRead;

/// Magic number of little endian mo files encoding
///
/// Number that when found reading the four first bits read as unsigned
/// 32 bits little endian of a mo file indicates that the file is in little
/// endian encoding.
///
/// Value as decimal: `2500072158`
pub const MAGIC: u32 = 0x950412de;

/// Magic number of big endian mo files encoding
///
/// Number that when found reading the four first bits read as unsigned
/// 32 bits little endian of a mo file indicates that the file is in big
/// endian encoding.
///
/// Value as decimal: `3725722773`
pub const MAGIC_SWAPPED: u32 = 0xde120495;

type MsgsIndex = Vec<(u32, u32)>;

fn maybe_extract_plurals_from_msgid_msgstr<'a>(
    msgid: &'a str,
    msgstr: &str,
) -> (Cow<'a, str>, Option<String>, Vec<String>) {
    if !msgid.contains('\u{0}') {
        return (msgid.into(), None, vec![]);
    }
    let msgid_tokens = msgid.split('\u{0}').collect::<Vec<&str>>();

    let (msgid, msgid_plural) = (msgid_tokens[0], msgid_tokens[1]);
    let msgstr_plural = msgstr
        .split('\u{0}')
        .map(|s| s.into())
        .collect::<Vec<String>>();

    (msgid.into(), Some(msgid_plural.into()), msgstr_plural)
}

fn maybe_extract_msgctxt_from_msgid(
    msgid: &str,
) -> (Cow<'_, str>, Option<String>) {
    let msgid_tokens = msgid.split('\x04').collect::<Vec<&str>>();

    if msgid_tokens.len() == 2 {
        (msgid_tokens[0].into(), Some(msgid_tokens[1].to_string()))
    } else {
        (msgid.into(), None)
    }
}

/// Parser for MO files
pub(crate) struct MOFileParser<'a> {
    /// File handler
    fhandle: Box<dyn SeekRead + 'a>,
    /// Function to read 4 bytes from the file
    freader: &'a dyn Fn([u8; 4]) -> u32,
    /// Parsed MO file
    pub file: MOFile,
}

impl MOFileParser<'_> {
    pub fn new<'a>(file_options: FileOptions) -> MOFileParser<'a> {
        let mut file = MOFile::new(file_options);
        let fhandle: Box<dyn SeekRead> = match Path::new(
            &file.options.path_or_content,
        )
        .is_file()
        {
            true => Box::new(
                File::open(&file.options.path_or_content).unwrap(),
            ),
            false => Box::new(Cursor::new(
                file.options.byte_content.as_mut().unwrap().clone(),
            )),
        };

        MOFileParser {
            fhandle,
            freader: &u32::from_le_bytes,
            file,
        }
    }

    pub fn parse(&mut self) -> Result<(), IOError> {
        // Parse magic number
        self.parse_magic_number()?;

        // Parse revision number
        self.file.version = Some(self.parse_revision_number()?);

        // Get number of strings
        let number_of_strings = self.parse_numofstrings()?;

        // Get messages offsets
        let (msgids_table_offset, msgstrs_table_offset) =
            self.parse_tables_offsets()?;

        // Parse messages indexes
        let (msgids_index, msgstrs_index) = self.parse_msgs_indexes(
            number_of_strings,
            msgids_table_offset,
            msgstrs_table_offset,
        )?;

        // Parse messages
        self.parse_msgs(
            number_of_strings,
            msgids_index,
            msgstrs_index,
        );
        Ok(())
    }

    fn parse_4_bytes(&mut self) -> Result<u32, std::io::Error> {
        let mut buffer = [0; 4];
        self.fhandle.read_exact(&mut buffer)?;
        Ok((self.freader)(buffer))
    }

    fn parse_magic_number(&mut self) -> Result<(), IOError> {
        match self.parse_4_bytes() {
            Ok(magic_number) => {
                if magic_number == MAGIC_SWAPPED {
                    self.freader = &u32::from_be_bytes;
                } else if magic_number != MAGIC {
                    return Err(IOError::IncorrectMagicNumber {
                        magic_number_le: magic_number,
                        magic_number_be: u32::from_be_bytes(
                            u32::to_le_bytes(magic_number),
                        ),
                    });
                }
                self.file.magic_number = Some(magic_number);
                Ok(())
            }
            Err(_e) => Err(IOError::ErrorReadingMagicNumber {}),
        }
    }

    fn parse_revision_number(&mut self) -> Result<u32, IOError> {
        match self.parse_4_bytes() {
            Ok(version) => {
                // from MO file format specs: "A program seeing an unexpected major
                // revision number should stop reading the MO file entirely"
                let available_versions: [u32; 2] = [0, 1];
                if !available_versions.contains(&version) {
                    return Err(
                        IOError::UnsupportedMORevisionNumber {
                            version,
                        },
                    );
                }
                Ok(version)
            }
            Err(_e) => Err(IOError::CorruptedMOData {
                context: "parsing revision number".to_string(),
            }),
        }
    }

    fn parse_numofstrings(&mut self) -> Result<u32, IOError> {
        match self.parse_4_bytes() {
            Ok(number_of_strings) => Ok(number_of_strings),
            Err(_e) => Err(IOError::CorruptedMOData {
                context: "parsing number of strings".to_string(),
            }),
        }
    }

    fn parse_tables_offsets(
        &mut self,
    ) -> Result<(u32, u32), IOError> {
        let msgids_table_offset = match self.parse_4_bytes() {
            Ok(offset) => offset,
            Err(_e) => {
                return Err(IOError::CorruptedMOData {
                    context: "parsing msgids table offset"
                        .to_string(),
                })
            }
        };
        let msgstrs_table_offset = match self.parse_4_bytes() {
            Ok(offset) => offset,
            Err(_e) => {
                return Err(IOError::CorruptedMOData {
                    context: "parsing msgstrs table offset"
                        .to_string(),
                })
            }
        };

        Ok((msgids_table_offset, msgstrs_table_offset))
    }

    fn parse_indexes_table(
        &mut self,
        number_of_strings: u32,
        table_offset: u32,
        context: &str,
    ) -> Result<Vec<(u32, u32)>, IOError> {
        self.fhandle.seek(SeekFrom::Start(table_offset as u64)).ok();
        let mut indexes: Vec<(u32, u32)> = vec![];
        for i in 0..number_of_strings {
            let msgid_length = match self.parse_4_bytes() {
                Ok(msgid_length) => msgid_length,
                Err(_e) => {
                    return Err(IOError::CorruptedMOData {
                        context: format!(
                            "parsing {context} length at index {i}",
                        ),
                    })
                }
            };
            let msgid_offset = match self.parse_4_bytes() {
                Ok(msgid_offset) => msgid_offset,
                Err(_e) => {
                    return Err(IOError::CorruptedMOData {
                        context: format!(
                            "parsing {context} offset at index {i}",
                        ),
                    })
                }
            };
            indexes.push((msgid_length, msgid_offset));
        }
        Ok(indexes)
    }

    fn parse_msgs_indexes(
        &mut self,
        number_of_strings: u32,
        msgids_table_offset: u32,
        msgstrs_table_offset: u32,
    ) -> Result<(MsgsIndex, MsgsIndex), IOError> {
        let msgids_index = self.parse_indexes_table(
            number_of_strings,
            msgids_table_offset,
            "msgid",
        )?;

        let msgstrs_index = self.parse_indexes_table(
            number_of_strings,
            msgstrs_table_offset,
            "msgstr",
        )?;

        Ok((msgids_index, msgstrs_index))
    }

    fn parse_msgs(
        &mut self,
        number_of_strings: u32,
        msgids_index: Vec<(u32, u32)>,
        msgstrs_index: Vec<(u32, u32)>,
    ) {
        for i in 0..number_of_strings {
            let (msgid_length, msgid_offset) =
                msgids_index[i as usize];
            let (msgstr_length, msgstr_offset) =
                msgstrs_index[i as usize];

            self.fhandle
                .seek(SeekFrom::Start(msgid_offset as u64))
                .ok();
            let mut msgid_buffer = vec![0; msgid_length as usize];
            self.fhandle.read_exact(&mut msgid_buffer).ok();
            let msgid = String::from_utf8_lossy(&msgid_buffer);

            self.fhandle
                .seek(SeekFrom::Start(msgstr_offset as u64))
                .ok();
            let mut msgstr_buffer = vec![0; msgstr_length as usize];
            self.fhandle.read_exact(&mut msgstr_buffer).ok();
            let msgstr = String::from_utf8_lossy(&msgstr_buffer);

            if i == 0 && msgid.is_empty() {
                // metadata entry
                for metadata_line in msgstr.split('\n') {
                    let mut tokens = metadata_line.splitn(2, ':');
                    let metadata_key =
                        tokens.next().unwrap_or("").to_string();
                    let metadata_value = tokens
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    if !metadata_key.is_empty() {
                        self.file
                            .metadata
                            .insert(metadata_key, metadata_value);
                    }
                }
                continue;
            }

            // check if we have a plural entry
            let (msgid, msgid_plural, msgstr_plural) =
                maybe_extract_plurals_from_msgid_msgstr(
                    &msgid, &msgstr,
                );
            let msgctxt_tokens =
                maybe_extract_msgctxt_from_msgid(&msgid);
            let msgctxt = msgctxt_tokens.1;

            let entry = MOEntry::new(
                msgctxt_tokens.0.to_string(),
                Some(msgstr.into_owned()),
                msgid_plural,
                msgstr_plural,
                msgctxt,
            );
            self.file.entries.push(entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rspolib_testing::{
        create_binary_content, create_corrupted_binary_content,
    };
    use std::fs;

    fn all_features_test(parser: &MOFileParser) {
        let po_path = "tests-data/all.po";
        let po_content = fs::read_to_string(po_path).unwrap();

        assert_eq!(parser.file.metadata.len(), 10);
        assert_eq!(
            parser.file.metadata.get("Project-Id-Version"),
            Some(&"django".to_string())
        );

        assert_eq!(parser.file.entries.len(), 7);

        let n_msgid_plural_entries = parser
            .file
            .entries
            .iter()
            .filter(|e| e.msgid_plural.is_some())
            .count();

        // msgid
        assert_eq!(
            po_content.matches("msgid \"").count()
                - 1
                - po_content.matches("#~ msgid \"").count() * 2,
            parser.file.entries.len(),
        );

        // msgstr
        assert_eq!(
            po_content.matches("msgstr \"").count()
                - 1
                - po_content.matches("#~ msgstr \"").count(),
            parser
                .file
                .entries
                .iter()
                .filter(|e| e.msgstr.is_some())
                .count()
                - n_msgid_plural_entries,
        );

        // msgctxt
        assert_eq!(
            po_content.matches("msgctxt \"").count(),
            parser
                .file
                .entries
                .iter()
                .filter(|e| e.msgctxt.is_some())
                .count(),
        );

        // msgid_plural
        assert_eq!(
            // -1 because fuzzy entries are not included in mo files
            po_content.matches("msgid_plural \"").count() - 1,
            n_msgid_plural_entries,
        );
    }

    #[test]
    fn parse_from_file() -> Result<(), IOError> {
        let mo_path = "tests-data/all.mo";
        let mut parser = MOFileParser::new(mo_path.into());
        parser.parse()?;

        all_features_test(&parser);
        Ok(())
    }

    #[test]
    fn parse_from_bytes() -> Result<(), IOError> {
        let bytes = std::fs::read("tests-data/all.mo").ok().unwrap();
        let mut parser = MOFileParser::new(bytes.into());
        parser.parse()?;

        all_features_test(&parser);
        Ok(())
    }

    #[test]
    fn error_on_invalid_magic_number() {
        let magic_number = 800;
        let data = vec![magic_number];
        let content = create_binary_content(&data, true);

        let mut parser = MOFileParser::new(content.into());
        let result = parser.parse();

        assert_eq!(
            result,
            Err(IOError::IncorrectMagicNumber {
                magic_number_le: magic_number,
                magic_number_be: 537067520
            })
        );
    }

    #[test]
    fn error_on_invalid_version_number() {
        let version = 234;
        let data = vec![MAGIC, version];
        let content = create_binary_content(&data, true);

        let mut parser = MOFileParser::new(content.into());
        let result = parser.parse();

        assert_eq!(
            result,
            Err(IOError::UnsupportedMORevisionNumber { version })
        );
    }

    fn valid_revision_number_test(version: u32, magic_number: u32) {
        let data: Vec<u32> = vec![
            magic_number,
            if magic_number == MAGIC {
                version
            } else {
                //v = 0b00000001_00000000_00000000_00000000;
                u32::from_be_bytes(u32::to_le_bytes(version))
            },
        ];
        let content =
            create_binary_content(&data, magic_number == MAGIC);

        let mut parser = MOFileParser::new(content.into());
        let result = parser.parse();
        assert_eq!(
            result,
            Err(IOError::CorruptedMOData {
                context: "parsing number of strings".to_string()
            })
        )
    }

    #[test]
    fn parse_valid_revision_numbers() {
        valid_revision_number_test(0, MAGIC);
        valid_revision_number_test(0, MAGIC_SWAPPED);
        valid_revision_number_test(1, MAGIC);
        valid_revision_number_test(1, MAGIC_SWAPPED);
    }

    fn corrupted_binary_test(
        data: &Vec<u32>,
        additional_bytes: &Vec<u8>,
        expected_context: &str,
    ) {
        for le in [true, false] {
            let content = create_corrupted_binary_content(
                data,
                le,
                // Add a number to the binary to force a byte read error
                additional_bytes,
            );

            let mut parser = MOFileParser::new(content.into());
            let result = parser.parse();

            assert_eq!(
                result,
                Err(IOError::CorruptedMOData {
                    context: expected_context.to_string()
                })
            );
        }
    }

    #[test]
    fn error_corrupted_number_of_strings() {
        corrupted_binary_test(
            &vec![MAGIC, 0],
            &vec![3],
            "parsing number of strings",
        );
    }

    #[test]
    fn error_corrupted_tables_offset() {
        corrupted_binary_test(
            &vec![MAGIC, 0, 7],
            &vec![4],
            "parsing msgids table offset",
        );

        corrupted_binary_test(
            &vec![MAGIC, 0, 7, 50],
            &vec![4],
            "parsing msgstrs table offset",
        );
    }
}
