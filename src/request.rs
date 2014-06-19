//! An alias of the rust-http Request struct.

use http::server::request::{AbsolutePath};
pub use Request = http::server::request::Request;

/// Adds a url getter method.
pub trait GetUrl {
    /// A url getter method for requests or responses.
    fn url<'a>(&'a self) -> Option<&'a String>;

    /// A mutable url getter method for requests or responses.
    fn url_mut<'a>(&'a mut self) -> Option<&'a mut String>;
}

impl GetUrl for Request {
    /// Get the url from a Request
    ///
    /// Returns Some(&url) if this is an AbsolutePath
    /// request, otherwise it returns None.
    fn url<'a>(&'a self) -> Option<&'a String> {
        match self.request_uri {
            AbsolutePath(ref path) => Some(path),
            _ => None
        }
    }

    /// Get a mutable url from a Request
    ///
    /// Returns Some(&mut url) if this is an AbsolutePath
    /// request, otherwise it returns None.
    fn url_mut<'a>(&'a mut self) -> Option<&'a mut String> {
        match self.request_uri {
            AbsolutePath(ref mut path) => Some(path),
            _ => None
        }
    }
}

