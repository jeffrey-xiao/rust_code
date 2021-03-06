//! Hybrid tree comprised of disk-resident sorted runs of data and memory-resident tree.

pub mod compaction;
mod map;
mod sstable;

pub use self::map::LsmMap;
use self::sstable::{SSTable, SSTableBuilder, SSTableDataIter, SSTableValue};
use bincode;
use std::error;
use std::fmt;
use std::io;
use std::result;

/// Convenience `Error` enum for `lsm_tree`.
#[derive(Debug)]
pub enum Error {
    /// An input or output error.
    IOError(io::Error),
    /// A serialization or deserialization error.
    SerdeError(bincode::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IOError(err)
    }
}

impl From<bincode::Error> for Error {
    fn from(err: bincode::Error) -> Error {
        Error::SerdeError(err)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::IOError(ref error) => error.source(),
            Error::SerdeError(ref error) => error.source(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IOError(ref error) => write!(f, "{}", error),
            Error::SerdeError(ref error) => write!(f, "{}", error),
        }
    }
}

/// Convenience `Result` type for `lsm_tree`.
pub type Result<T> = result::Result<T, Error>;
