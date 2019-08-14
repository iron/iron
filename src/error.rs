use std::fmt;

use modifier::Modifier;
use {Response};

pub use std::error::Error;
pub use hyper::Error as HttpError;
pub use hyper::error::Result as HttpResult;

/// The type of Errors inside and when using Iron.
///
/// `IronError` informs its receivers of two things:
///
/// * What went wrong
/// * What to do about it
///
/// The `error` field is responsible for informing receivers of which
/// error occured, and receivers may also modify the error field by layering
/// it (building up a cause chain).
///
/// The `response` field provides a tangible action to be taken if this error
/// is not otherwise handled.
#[derive(Debug)]
pub struct IronError {
    /// The underlying error
    ///
    /// This can be layered and will be logged at the end of an errored
    /// request.
    pub error: Box<Error + Send>,

    /// What to do about this error.
    ///
    /// This Response will be used when the error-handling flow finishes.
    pub response: Response
}

impl IronError {
    /// Create a new `IronError` from an error and a modifier.
    pub fn new<E: 'static + Error + Send, M: Modifier<Response>>(e: E, m: M) -> IronError {
        IronError {
            error: Box::new(e),
            response: Response::with(m)
        }
    }
}

impl fmt::Display for IronError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(&*self.error, f)
    }
}

impl Error for IronError {
    fn description(&self) -> &str {
        self.error.description()
    }

    #[allow(deprecated)]
    fn cause(&self) -> Option<&Error> {
        self.error.cause()
    }
}

