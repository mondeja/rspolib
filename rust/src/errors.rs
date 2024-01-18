//! Errors generated by the parsers
//!
//! # Complete example
//!
//! ## Read a PO file (SyntaxError)
//!
//! ```rust
//! use rspolib::{pofile, POFile, errors::SyntaxError};
//!
//! let path = "tests-data/unescaped-double-quote-msgid.po";
//!
//! let file: Option<POFile> = match pofile(path) {
//!     Ok(file) => Some(file),
//!     Err(e) => match e {
//!         ref SyntaxError => {
//!             assert!(e.to_string().ends_with("unescaped double quote found"));
//!             None
//!         },
//!     },
//! };
//! ```
//!
//! ## Read a MO file (IOError)
//!
//! ```rust
//! use rspolib::{mofile, MOFile, errors::IOError, MAGIC};
//! use rspolib_testing::create_binary_content;
//!
//! let version = 0;
//! let data = vec![MAGIC, version];
//! let content = create_binary_content(&data, true);
//!
//! let file: Option<MOFile> = match mofile(content) {
//!     Ok(file) => Some(file),
//!     Err(e) => match e {
//!         ref IOError => {
//!             assert!(e.to_string().ends_with("malformed or corrupted data found when parsing number of strings"));
//!             None
//!         },
//!     },
//! };
//! ```
//!
use std::fmt;

use snafu::prelude::*;

/// A struct to represent a path to a file or a file content
#[derive(Debug, PartialEq)]
pub struct MaybeFilename {
    filename: String,
    filename_is_path: bool,
}

impl MaybeFilename {
    pub fn new(filename: &str, filename_is_path: bool) -> Self {
        Self {
            filename: filename.to_string(),
            filename_is_path,
        }
    }
}

impl fmt::Display for MaybeFilename {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.filename_is_path {
            write!(f, " in file {}", self.filename)
        } else {
            Ok(())
        }
    }
}

/// Errors generated when the mo files parser can't parse some content.
///
/// # Examples
///
/// ## Unsupported MO revision number
///
/// ```rust
/// use rspolib::{mofile, errors::IOError, MAGIC};
/// use rspolib_testing::create_binary_content;
///
/// let version = 234;
/// let data = vec![MAGIC, version];
/// let content = create_binary_content(&data, true);
///
/// assert_eq!(
///   mofile(content),
///   Err(IOError::UnsupportedMORevisionNumber { version }),
/// );
/// ```
///
/// ## Incorrect magic number
///
/// ```
/// use rspolib::{mofile, errors::IOError};
/// use rspolib_testing::create_binary_content;
///
/// let magic_number = 800;
/// let data = vec![magic_number];
/// let content = create_binary_content(&data, true);
///
/// assert_eq!(
///     mofile(content),
///     Err(IOError::IncorrectMagicNumber {
///         magic_number_le: magic_number,
///         magic_number_be: 537067520
///     })
/// );
/// ```
#[derive(Debug, PartialEq, Snafu)]
pub enum IOError {
    /// An error has been happened trying to read the four bytes
    /// that should contain the unsigned 32 bits integer magic
    /// number.
    ///
    /// This mainly happens when your file has 0, 1, 2 or 3 bits
    /// long. Usually means that you have tried to read an empty
    /// file as the magic number are the four first bytes of MO files.
    #[snafu(display("Invalid mo file, error reading magic number"))]
    ErrorReadingMagicNumber {},

    /// The magic number read from the MO file is not a valid one.
    ///
    /// The valid numbers are `0x950412de` (little endian)
    /// and `0xde120495` (big endian). If you are getting this
    /// error means that the first 4 bytes of your mo file
    /// converted as an unsigned 32 bits integer are not one
    /// of those numbers.
    ///
    /// Usually this happens because the buffer offset reading the
    /// file is not correct or the file has been saved with an
    /// incorrect magic number.
    #[snafu(display(
        concat!(
            "Invalid mo file, magic number is incorrect",
            " ({{magic_number_le}} read as le, {{magic_number_be}}",
            " read as be)",
        )
    ))]
    IncorrectMagicNumber {
        magic_number_le: u32,
        magic_number_be: u32,
    },

    /// The revision number of the MO file is not supported
    ///
    /// From the beginning, MO files have maintained the specification
    /// without changes. This number was introduced to make possible
    /// changes without breaking old compatibility, but never has been
    /// used. However, the library expects that should be 0 or 1 because
    /// the specification says that only those values are valid.
    ///
    /// This usually happens when the file data is corrupted as probably
    /// no MO files has been created with a revision number different
    /// than 0 or 1 ever.
    #[snafu(display("Invalid mo file, expected revision number 0 or 1, found {version}"))]
    UnsupportedMORevisionNumber { version: u32 },

    /// Some of the data in the MO file is corrupted or malformed.
    ///
    /// This error happens when trying to read the data of the translations
    /// tables. It contains a different error message in each step of the
    /// parsing process.
    ///
    /// It means that data is corrupted or malformed in some way. It can
    /// be produced by reading the file with a wrong offset or by reading
    /// a file that is not a mo file.
    #[snafu(display("Invalid mo file, malformed or corrupted data found when {context}"))]
    CorruptedMOData { context: String },
}

/// Syntax errors generated when the PO parser can't parse some content.
///
/// # Examples
///
/// ## Unescaped double quote found
///
/// ```rust
/// use rspolib::{pofile, errors::{SyntaxError, MaybeFilename}};
///
/// let content = r#"#
/// msgid "Hello"
/// msgstr "Ho"la"
///"#;
///
/// assert_eq!(
///     pofile(content),
///     Err(SyntaxError::UnescapedDoubleQuoteFound {
///         maybe_filename: MaybeFilename::new(content, false),
///         line: 3,
///         index: 11,
///     }),
/// );
/// ```
///
/// ## Unknown keyword
///
/// ```rust
/// use rspolib::{pofile, errors::{SyntaxError, MaybeFilename}};
///
/// let content = r#"#
/// #| previous_message = "Good morning"
/// msgid "Hello"
/// msgstr "Hola"
/// "#;
///
/// assert_eq!(
///     pofile(content),
///     Err(SyntaxError::Custom {
///         maybe_filename: MaybeFilename::new(content, false),
///         line: 2,
///         index: 0,
///         message: "unknown keyword previous_message".to_string(),
///     }),
/// );
#[derive(Debug, PartialEq, Snafu)]
pub enum SyntaxError {
    /// An unescaped double quote has been found in a po field string
    ///
    /// Happens mainly when you edit the file manually and forget
    /// to escape the double quote characters. It can also happen when you
    /// are reading a file that has been saved without escaping the double
    /// quotes in po string fields.
    #[snafu(display("Syntax error found{maybe_filename} at line {line} (index {index}): unescaped double quote found"))]
    UnescapedDoubleQuoteFound {
        maybe_filename: MaybeFilename,
        line: usize,
        index: usize,
    },

    /// A generic syntax error that includes a message about what was
    /// the error has been found parsing a po file
    ///
    /// This happens when the parser finds a syntax error that is a expected
    /// syntax error, so it includes information about the error in the `message`
    /// field
    #[snafu(display("Syntax error found{maybe_filename} at line {line} (index {index}): {message}"))]
    Custom {
        maybe_filename: MaybeFilename,
        line: usize,
        index: usize,
        message: String,
    },

    /// A generic syntax error without information about the line or the index
    #[snafu(display(
        "Syntax error found{maybe_filename}: {message}"
    ))]
    BasicCustom {
        maybe_filename: MaybeFilename,
        message: String,
    },

    /// A generic syntax error has been found parsing a po file
    ///
    /// Happens when the parser finds a syntax error that is not
    /// covered by any other error.
    ///
    /// It can happen when you are reading a file that has been saved
    /// with strange characters in field names like `msgid` or `msgstr`.
    #[snafu(display("Syntax error found{maybe_filename} at line {line} (index {index})"))]
    Generic {
        maybe_filename: MaybeFilename,
        line: usize,
        index: usize,
    },

    /// Unknown parsing state
    #[snafu(display("Unknown state {state}"))]
    UnknownState { state: String },
}

/// Escaping errors generated by escaping functions.
///
/// These errors are not generated by the parser, so you don't
/// need to worry about them if you are using the parser, only
/// if you are using the escaping functions directly.
#[derive(Debug, PartialEq, Snafu)]
pub enum EscapingError {
    #[snafu(display(
        "escape sequence found at end of string '{text}'"
    ))]
    EscapeAtEndOfString { text: String },

    #[snafu(display(
        "invalid escaped character '{character}' found in '{text}'"
    ))]
    InvalidEscapedCharacter { text: String, character: char },
}
