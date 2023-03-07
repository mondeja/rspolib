//! PO and MO files manipulation library.
//!
//! Port to Rust of the Python library [polib].
//! The implementation differs a bit to make it even better.
//!
//! ## Quick start
//!
//! ### Read and save a PO file
//!
//! ```rust
//! use rspolib::{pofile, Save};
//!
//! let file = pofile("tests-data/obsoletes.po").unwrap();
//! for entry in file.translated_entries() {
//!     println!("{}", &entry.msgid);
//! }
//! file.save("tests-data/docs/pofile_save.po");
//! ```
//!
//! ### Read and save a MO file
//!
//! ```rust
//! // You can include the prelude to access to all the methods
//! use rspolib::{mofile, prelude::*};
//!
//! let mut file = mofile("tests-data/all.mo").unwrap();
//! for entry in &file.entries {
//!     // All entries are translated in MO files
//!     println!("{}", entry.msgid);
//! }
//! file.save("tests-data/docs/mofile_save.mo");
//! ```
//!
//! ## Features
//!
//! * Unicode Line Breaking formatting support.
//! * Correct handling of empty and non existent PO fields values.
//! * Detailed error handling parsing PO and MO files.
//! * Custom byte order MO files generation.
//!
//! ## General view
//!
//! * [POFile]s, contains [POEntry]s.
//! * [MOFile]s, contains [MOEntry]s.
//!
//! Items of the same level can be converted between them,
//! for example a [POEntry] can be converted to a [MOEntry] using
//! `MOEntry::from(&POEntry)` because [MOEntry]s implement the
//! [From] trait for &[POEntry].
//!
//! All of the conversions that make sense are implemented for
//! all the structs. For example, to get the representation of a
//! [POFile] just call `to_string()` or to get the binary representation
//! of bytes of a [MOFile] calls `as_bytes()`.
//!
//! [polib]: https://github.com/izimobil/polib

#[doc(hidden)]
pub mod bitwise;
mod entry;
pub mod errors;
#[doc(hidden)]
pub mod escaping;
mod file;
mod moparser;
mod poparser;
pub mod prelude;
mod traits;
mod twrapper;

pub use crate::entry::{
    mo_metadata_entry_to_string, po_metadata_entry_to_string,
    MOEntry, MsgidEotMsgctxt, POEntry, Translated as TranslatedEntry,
};
pub use crate::file::{
    mofile::{mofile, MOFile},
    pofile::{pofile, POFile},
    AsBytes, FileOptions, Save, SaveAsMOFile, SaveAsPOFile,
};
pub use crate::moparser::{MAGIC, MAGIC_SWAPPED};
pub use crate::traits::Merge;
