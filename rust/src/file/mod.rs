pub mod mofile;
pub mod pofile;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::Path;

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

/// Save file as a PO file with the `save_as_pofile` method
pub trait SaveAsPOFile {
    /// Save the file as a PO file to the given path
    fn save_as_pofile(&self, path: &str)
    where
        Self: fmt::Display,
    {
        let mut file = File::create(path).unwrap();
        file.write_all(self.to_string().as_bytes()).ok();
    }
}

/// Save file with the `save` method
pub trait Save {
    /// Save the file to the given path
    fn save(&self, path: &str);
}

/// Save file as a MO file with the `save_as_mofile` method
pub trait SaveAsMOFile {
    /// Save the file as a MO file to the given path
    fn save_as_mofile(&self, path: &str);
}

/// Provides functions to convert to MO files content as bytes
///
/// * `as_bytes` method as an alias to `as_bytes_le`.
/// * `as_bytes_le` method to return the content as bytes in
///   little endian byte order.
/// * `as_bytes_be` method to return the content as bytes in
///   big endian byte order.
pub trait AsBytes {
    /// Return the content as bytes
    fn as_bytes(&self) -> Cow<'_, [u8]>;
    /// Return the content as bytes in little endian encoding
    fn as_bytes_le(&self) -> Cow<'_, [u8]>;
    /// Return the content as bytes in big endian encoding
    fn as_bytes_be(&self) -> Cow<'_, [u8]>;
}

/// File options struct passed when creating a new PO or MO file
///
/// # Examples
///
/// ```rust
/// use std::fs;
/// use rspolib::FileOptions;
///
/// // From path
/// let opts = FileOptions::from("tests-data/all.po");
/// assert_eq!(opts.path_or_content, "tests-data/all.po");
/// assert_eq!(opts.wrapwidth, 78);
///
/// // From path and wrap width
/// let opts = FileOptions::from(("tests-data/obsoletes.po", 80));
/// assert_eq!(opts.path_or_content, "tests-data/obsoletes.po");
/// assert_eq!(opts.wrapwidth, 80);
///
/// // From bytes
/// let bytes = fs::read("tests-data/obsoletes.po").unwrap();
/// let opts = FileOptions::from(bytes);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct FileOptions {
    /// Path or content to the file
    pub path_or_content: String,
    /// Wrap width for the PO file, used when converted as a string
    pub wrapwidth: usize,
    /// Content as bytes, used by MO files when the content is passed as bytes
    pub byte_content: Option<Vec<u8>>,
}

impl Default for FileOptions {
    fn default() -> Self {
        Self {
            path_or_content: "".to_string(),
            wrapwidth: 78,
            byte_content: None,
        }
    }
}

impl From<&FileOptions> for FileOptions {
    fn from(options: &Self) -> Self {
        Self {
            path_or_content: options.path_or_content.clone(),
            wrapwidth: options.wrapwidth,
            ..Default::default()
        }
    }
}

impl<'a> From<&'a str> for FileOptions {
    fn from(path_or_content: &'a str) -> Self {
        Self {
            path_or_content: path_or_content.to_string(),
            ..Default::default()
        }
    }
}

impl<'a> From<(&'a str, usize)> for FileOptions {
    fn from(opts: (&'a str, usize)) -> Self {
        Self {
            path_or_content: opts.0.to_string(),
            wrapwidth: opts.1,
            ..Default::default()
        }
    }
}

impl From<Vec<u8>> for FileOptions {
    fn from(byte_content: Vec<u8>) -> Self {
        Self {
            byte_content: Some(byte_content),
            ..Default::default()
        }
    }
}

impl From<(Vec<u8>, usize)> for FileOptions {
    fn from((byte_content, wrapwidth): (Vec<u8>, usize)) -> Self {
        Self {
            path_or_content: "".to_string(),
            wrapwidth,
            byte_content: Some(byte_content),
        }
    }
}

impl From<&Path> for FileOptions {
    fn from(path: &Path) -> Self {
        Self {
            path_or_content: path.to_str().unwrap().to_string(),
            ..Default::default()
        }
    }
}

fn metadata_hashmap_to_msgstr(
    metadata: &HashMap<String, String>,
) -> String {
    let ordered_map = metadata_hashmap_to_ordered(metadata);
    let mut parts: Vec<String> =
        Vec::with_capacity(ordered_map.len());
    for (key, value) in ordered_map {
        let mut msgstr =
            String::with_capacity(key.len() + value.len() + 2);
        msgstr.push_str(&key);
        msgstr.push_str(": ");
        msgstr.push_str(&value);
        parts.push(msgstr);
    }
    parts.join("\n")
}

fn metadata_hashmap_to_ordered(
    metadata: &HashMap<String, String>,
) -> Vec<(String, String)> {
    let mut ret: Vec<(String, String)> =
        Vec::with_capacity(METADATA_KEYS_ORDER.len());
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
        // FileOptions from &FileOptions
        let options = FileOptions {
            wrapwidth: 50,
            path_or_content: "foobar".to_string(),
            byte_content: None,
        };

        let options_from_options = FileOptions::from(&options);
        assert_eq!(options_from_options.wrapwidth, 50);
        assert_eq!(options_from_options.path_or_content, "foobar");

        // FileOptions from &str
        let options_from_str = FileOptions::from("foobar");
        assert_eq!(options_from_str.wrapwidth, 78);
        assert_eq!(options_from_str.path_or_content, "foobar");

        // FileOptions from (&str, usize)
        let options_from_str_and_usize =
            FileOptions::from(("foobar", 50));
        assert_eq!(options_from_str_and_usize.wrapwidth, 50);
        assert_eq!(
            options_from_str_and_usize.path_or_content,
            "foobar"
        );
    }
}
