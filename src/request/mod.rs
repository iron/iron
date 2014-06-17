//! Exposes the `Request` trait and `IronRequest` type.

use http::headers::request::HeaderCollection;
use http::server::request::RequestUri;
use http::method::Method;
use HttpRequest = http::server::request::Request;

pub mod ironrequest;

pub trait Request {
    fn headers_mut<'a>(&'a mut self)          -> &'a mut Box<HeaderCollection>;
    fn body_mut<'a>(&'a mut self)             -> &'a mut String;
    fn method_mut<'a>(&'a mut self)           -> &'a mut Method;
    fn uri_mut<'a>(&'a mut self)              -> &'a mut RequestUri;
    fn close_connection_mut<'a>(&'a mut self) -> &'a mut bool;

    fn headers<'a>(&'a self)          -> &'a HeaderCollection;
    fn body<'a>(&'a self)             -> &'a str;
    fn method<'a>(&'a self)           -> &'a Method;
    fn uri<'a>(&'a self)              -> &'a RequestUri;
    fn close_connection<'a>(&'a self) -> bool;

    #[inline]
    fn version(&self) -> (uint, uint) { (1, 1) }

    fn from_http(&HttpRequest) -> Self;
}

