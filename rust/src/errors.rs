use std::fmt;

use snafu::prelude::*;

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

#[derive(Debug, PartialEq, Snafu)]
pub enum IOError {
    #[snafu(display("Invalid mo file, magic number is incorrect ({magic_number_le} read as le, {magic_number_be} read as be)"))]
    IncorrectMagicNumber {
        magic_number_le: u32,
        magic_number_be: u32,
    },
    #[snafu(display("Invalid mo file, error reading magic number"))]
    ErrorReadingMagicNumber {},
    #[snafu(display("Invalid mo file, malformed or corrupted data found when {context}"))]
    CorruptedMOData { context: String },
    #[snafu(display("Invalid mo file, unexpected revision number 0 or 1, found {version}"))]
    UnsupportedMORevisionNumber { version: u32 },
}

#[derive(Debug, PartialEq, Snafu)]
pub enum SyntaxError {
    #[snafu(display("Syntax error found{maybe_filename} at line {line} (index {index}): unescaped double quote found"))]
    UnescapedDoubleQuoteFound {
        maybe_filename: MaybeFilename,
        line: usize,
        index: usize,
    },

    #[snafu(display("Syntax error found{maybe_filename} at line {line} (index {index})"))]
    Generic {
        maybe_filename: MaybeFilename,
        line: usize,
        index: usize,
    },

    #[snafu(display("Syntax error found{maybe_filename} at line {line} (index {index}): {message}"))]
    Custom {
        maybe_filename: MaybeFilename,
        line: usize,
        index: usize,
        message: String,
    },
}
