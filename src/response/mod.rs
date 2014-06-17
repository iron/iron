//! Exposes the `Response` trait and `IronResponse` type.

use http::server::request::Request;
use http::headers::response::HeaderCollection;
use http::status::Status;
use  http::server::response::ResponseWriter;

/// The default implementation of Response.
pub mod ironresponse;

/// A generic trait for a TcpStream HTTP response.
///
/// `Iron` can use custom HTTP implementations so long
/// as they define certain functions, in order to remain
/// usable by available `Ingots`.
///
/// `IronResponse` is the default implementation.
/// Most uses will only need `IronResponse`.
pub trait Response: Writer {
    /// Get a mutable reference to the headers.
    ///
    /// Any new headers should be set here, to be written
    /// when the response is completely unwound.
    fn headers_mut<'a>(&'a mut self) -> &'a mut Box<HeaderCollection>;
    /// Get a mutable reference to the status.
    ///
    /// The HTTP status should be set here, to be written
    /// when the response is completely unwound.
    ///
    /// Status codes are currently taken from rust-http.
    fn status_mut<'a>(&'a mut self) -> &'a mut Status;

    /// Get an immutable reference to the request.
    ///
    /// This is an artifact of the rust-http framework.
    /// It will always hold the original request,
    /// regardless of any routing/mounting `Ingots` or
    /// others that mutate the `Request` generic.
    fn request<'a>(&'a self) -> &'a Request;
    /// Get a reference to the headers.
    fn headers<'a>(&'a self) -> &'a HeaderCollection;
    /// Get a reference to the status.
    fn status<'a>(&'a self) -> &'a Status;
}

/// A trait to create a wrapped response from a rust-http ResponseWriter.
///
/// This trait is separated to shield users from lifetimes.
/// The ResponseWriter wraps a TcpStream Writer, and cannot be wrapped
/// without using lifetimes. Separating this out into a separate trait
/// allows `Response` to be implemented without lifetimes.
pub trait HttpResponse<'a, 'b> {
    /// Create a wrapped `Response` from a rust-http response.
    fn from_http(&mut ResponseWriter) -> Self;
}
