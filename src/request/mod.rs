//! Exposes the `Request` trait and `IronRequest` type.

use http::headers::request::HeaderCollection;
use http::server::request::RequestUri;
use http::method::Method;
use http::server::request;

/// The default implementation of Request.
pub mod ironrequest;

/// A generic trait for a minimally-parsed HTTP request.
///
/// Requests must expose their
/// headers, HTTP method, uri, and body (as a String).
/// Any further parsing should be done by `Ingots`.
///
/// `Iron` can use custom HTTP implementations so long
/// as they define certain functions, in order to remain
/// usable by available `Ingots`.
pub trait Request {
    /// Get a mutable reference to the headers.
    fn headers_mut<'a>(&'a mut self)          -> &'a mut Box<HeaderCollection>;
    /// Get a mutable reference to the body.
    ///
    /// The body is not parsed. This reference allows `Ingots` to
    /// parse and expose the body in various ways, as well as hide
    /// things from `Ingots` that come later in the `Furnace`.
    fn body_mut<'a>(&'a mut self)             -> &'a mut String;
    /// Get a mutable reference to the HTTP verb.
    ///
    /// `Ingots` can mutate this according to internal logic to
    /// control available verbs and their definitions.
    ///
    /// HTTP verbs are currently taken from rust-http.
    fn method_mut<'a>(&'a mut self)           -> &'a mut Method;
    /// Get a mutable reference to the uri/path.
    ///
    /// This is most useful for mounting. Mutating the uri allows hiding
    /// the original uri from future `Ingots` (although that information
    /// will always be available through the `Response` generic).
    fn uri_mut<'a>(&'a mut self)              -> &'a mut RequestUri;
    /// Get a mutable reference to the close_connection boolean.
    ///
    /// This determines whether to close the TCP connection when the
    /// request has been served. If it is not closed by an `Ingot`,
    /// the stream will remain open listening for another request.
    fn close_connection_mut<'a>(&'a mut self) -> &'a mut bool;

    /// Get a reference to the headers.
    fn headers<'a>(&'a self)          -> &'a HeaderCollection;
    /// Get a reference to the raw body.
    fn body<'a>(&'a self)             -> &'a str;
    /// Get a reference to the HTTP verb.
    fn method<'a>(&'a self)           -> &'a Method;
    /// Get a reference to the uri.
    fn uri<'a>(&'a self)              -> &'a RequestUri;
    /// Get a reference to the close_connection boolean.
    fn close_connection<'a>(&'a self) -> bool;

    /// Get a reference to the HTTP version.
    ///
    /// rust-http currently only uses 1.1.
    #[inline]
    fn version(&self) -> (uint, uint) { (1, 1) }
}

/// A trait to create a wrapped request from a rust-http Request.
///
/// This trait is separated for consistency with Response.
pub trait HttpRequest {
    /// Create a wrapped `Request` from a rust-http request.
    fn from_http(&request::Request) -> Self;
}
