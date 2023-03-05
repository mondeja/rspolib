/// Merge entries and files
pub trait Merge {
    /// Merge a struct with another of the same type
    fn merge(&mut self, other: Self);
}

use std::io::{Read, Seek};

// Implementation to use `read_` and `seek_` methods
// for different types of readers
pub(crate) trait SeekRead: Seek + Read {}
impl<T: Seek + Read> SeekRead for T {}
