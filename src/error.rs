use std::error::Error as StdError;

use rmap::replace_map;

use modifier::Modifier;
use {Response};

pub use err::Error;
pub use hyper::{HttpResult, HttpError};

/// The type of Errors inside and when using Iron.
///
/// IronError informs its receivers of two things:
///   - What went wrong
///   - What to do about it
///
/// The `error` field is responsible for informing receivers of which
/// error occured, and receivers may also modify the error field by layering
/// it (building up a cause chain).
///
/// The `response` field provides a tangible action to be taken if this error
/// is not otherwise handled.
#[derive(Show)]
pub struct IronError {
    /// The underlying error
    ///
    /// This can be layered and will be logged at the end of an errored
    /// request.
    pub error: Box<Error>,

    /// What to do about this error.
    ///
    /// This Response will be used when the error-handling flow finishes.
    pub response: Response
}

impl IronError {
    /// Create a new IronError from an error and a modifier.
    pub fn new<E: Error, M: Modifier<Response>>(e: E, m: M) -> IronError {
        IronError {
            error: Box::new(e),
            response: Response::with(m)
        }
    }

    /// Use the provided function to wrap the error field.
    ///
    /// Can be used to move the error into another error for cause
    /// chaining.
    pub fn wrap<F>(&mut self, wrapper: F)
    where F: FnOnce(Box<Error>) -> Box<Error> {
        replace_map(&mut self.error, wrapper);
    }
}

impl StdError for IronError {
    fn description(&self) -> &str {
        self.error.description()
    }

    fn detail(&self) -> Option<String> {
        self.error.detail()
    }

    fn cause(&self) -> Option<&StdError> {
        self.error.cause()
    }
}

