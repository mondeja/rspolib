pub mod mofile;
pub mod pofile;

use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Write;

use natord::compare as compare_natural_order;

const METADATA_KEYS_ORDER: [&str; 11] = [
    "Project-Id-Version",
    "Report-Msgid-Bugs-To",
    "POT-Creation-Date",
    "PO-Revision-Date",
    "Last-Translator",
    "Language-Team",
    "Language",
    "MIME-Version",
    "Content-Type",
    "Content-Transfer-Encoding",
    "Plural-Forms",
];

pub trait SaveAsPOFile {
    fn save_as_pofile(&self, path: &str)
    where
        Self: fmt::Display,
    {
        let mut file = File::create(path).unwrap();
        file.write_all(self.to_string().as_bytes()).ok();
    }
}

pub trait Save {
    fn save(&self, path: &str);
}

pub trait SaveAsMOFile {
    fn save_as_mofile(&self, path: &str);
}

pub trait AsBytes {
    fn as_bytes(&self) -> Vec<u8>;
    fn as_bytes_le(&self) -> Vec<u8>;
    fn as_bytes_be(&self) -> Vec<u8>;
}

#[derive(Clone, Default)]
pub struct Options<'a> {
    pub path_or_content: &'a str,
    pub check_for_duplicates: bool,
    pub wrapwidth: usize,
    pub byte_content: Option<Vec<u8>>,
}

impl<'a> From<&Options<'a>> for Options<'a> {
    fn from(options: &Self) -> Self {
        Self {
            path_or_content: options.path_or_content,
            check_for_duplicates: options.check_for_duplicates,
            wrapwidth: options.wrapwidth,
            ..Default::default()
        }
    }
}

impl<'a> From<&'a str> for Options<'a> {
    fn from(path_or_content: &'a str) -> Self {
        Self {
            path_or_content,
            wrapwidth: 78,
            check_for_duplicates: false,
            ..Default::default()
        }
    }
}

impl<'a> From<(&'a str, usize)> for Options<'a> {
    fn from(opts: (&'a str, usize)) -> Self {
        Self {
            path_or_content: opts.0,
            wrapwidth: opts.1,
            check_for_duplicates: false,
            ..Default::default()
        }
    }
}

impl<'a> From<(&'a str, bool)> for Options<'a> {
    fn from(opts: (&'a str, bool)) -> Self {
        Self {
            path_or_content: opts.0,
            wrapwidth: 78,
            check_for_duplicates: opts.1,
            ..Default::default()
        }
    }
}

impl<'a> From<Vec<u8>> for Options<'a> {
    fn from(byte_content: Vec<u8>) -> Self {
        Self {
            path_or_content: "",
            wrapwidth: 78,
            check_for_duplicates: false,
            byte_content: Some(byte_content),
        }
    }
}

fn metadata_hashmap_to_msgstr(
    metadata: &HashMap<String, String>,
) -> String {
    let mut msgstr = String::new();
    for (key, value) in metadata_hashmap_to_ordered(metadata) {
        msgstr.push_str(&key);
        msgstr.push_str(": ");
        msgstr.push_str(&value);
        msgstr.push('\n');
    }
    msgstr.pop();
    msgstr
}

fn metadata_hashmap_to_ordered(
    metadata: &HashMap<String, String>,
) -> Vec<(String, String)> {
    let mut ret: Vec<(String, String)> = vec![];
    for key in METADATA_KEYS_ORDER {
        if metadata.contains_key(key) {
            let value = metadata.get(key).unwrap();
            ret.push((key.to_string(), value.to_string()));
        }
    }

    let mut metadata_keys = metadata.keys().collect::<Vec<&String>>();
    metadata_keys.sort_by(|&a, &b| compare_natural_order(a, b));

    for key in metadata_keys {
        if !METADATA_KEYS_ORDER.contains(&key.as_str()) {
            let value = metadata.get(key).unwrap();
            ret.push((key.to_string(), value.to_string()));
        }
    }

    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_from() {
        // Options from &Options
        let options = Options {
            wrapwidth: 50,
            path_or_content: "foobar",
            check_for_duplicates: true,
            byte_content: None,
        };

        let options_from_options = Options::from(&options);
        assert_eq!(options_from_options.wrapwidth, 50);
        assert_eq!(options_from_options.path_or_content, "foobar");
        assert_eq!(options_from_options.check_for_duplicates, true);

        // Options from &str
        let options_from_str = Options::from("foobar");
        assert_eq!(options_from_str.wrapwidth, 78);
        assert_eq!(options_from_str.path_or_content, "foobar");
        assert_eq!(options_from_str.check_for_duplicates, false);

        // Options from (&str, usize)
        let options_from_str_and_usize =
            Options::from(("foobar", 50));
        assert_eq!(options_from_str_and_usize.wrapwidth, 50);
        assert_eq!(
            options_from_str_and_usize.path_or_content,
            "foobar"
        );
        assert_eq!(
            options_from_str_and_usize.check_for_duplicates,
            false
        );

        // Options from (&str, bool)
        let options_from_str_and_bool =
            Options::from(("foobar", true));
        assert_eq!(options_from_str_and_bool.wrapwidth, 78);
        assert_eq!(
            options_from_str_and_bool.path_or_content,
            "foobar"
        );
        assert_eq!(
            options_from_str_and_bool.check_for_duplicates,
            true
        );
    }
}
