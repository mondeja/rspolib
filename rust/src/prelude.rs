//! rspolib prelude
//!
//! It includes traits to make use of the methods of files and entries.
//!
//! - [Save] trait to save a POFile or MOFile to a file using the `save` method.
//! - [SaveAsMOFile] trait to use the method `save_as_mofile`.
//! - [SaveAsPOFile] trait to use the method `save_as_pofile`.
//! - [Merge] trait to use the method `merge`.
//! - [TranslatedEntry] trait to use the method `translated` on entries.
//! - [AsBytes] trait to use the methods `as_bytes*` on POFile and MOFile.
//! - [MsgidEotMsgctxt] trait to use the method `msgid_eot_msgctxt` on entries.
pub use crate::{
    AsBytes, Merge, MsgidEotMsgctxt, Save, SaveAsMOFile,
    SaveAsPOFile, TranslatedEntry,
};
