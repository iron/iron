use http::headers::request::HeaderCollection;
use http::server::request::RequestUri;
use http::method::Method;

pub trait Request: Clone + Send {
    fn headers_mut<'a>(&'a self)          -> &'a mut HeaderCollection;
    fn body_mut<'a>(&'a self)             -> &'a mut String;
    fn method_mut<'a>(&'a self)           -> &'a mut Method;
    fn uri_mut<'a>(&'a self)              -> &'a mut RequestUri;
    fn close_connection_mut<'a>(&'a self) -> &'a mut bool;

    fn headers<'a>(&'a self)          -> &'a HeaderCollection;
    fn body<'a>(&'a self)             -> String;
    fn method<'a>(&'a self)           -> Method;
    fn uri<'a>(&'a self)              -> RequestUri;
    fn close_connection<'a>(&'a self) -> bool;

    #[inline]
    fn version(&self) -> (uint, uint) { (1, 1) }
}
