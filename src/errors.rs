//! Some common error types for use within Iron and in downstream middleware.

use std::io::IoError;
use {Error};

/// The standard Error implementation used to wrap IoErrors
/// that occur when reading or writing files or otherwise
/// interacting with the file system.
#[deriving(Show, Clone, PartialEq)]
pub struct FileError(pub IoError);

impl FileError {
    /// Create a new FileError from an IoError.
    pub fn new(err: IoError) -> FileError { FileError(err) }

    /// Access the original IoError.
    pub fn unwrap(self) -> IoError {
        let FileError(err) = self;
        err
    }
}

impl Error for FileError {
    fn name(&self) -> &'static str {
        let FileError(ref err) = *self;
        err.desc
    }

    fn description(&self) -> Option<&str> {
        let FileError(ref err) = *self;
        err.detail.as_ref().map(|s| s.as_slice())
    }
}

