use super::{Request, HttpRequest};
use http::headers::request::HeaderCollection;
use http::server::request::{RequestUri, Star, AbsoluteUri, AbsolutePath, Authority};
use http::method::Method;
use http::server::request;

/// The default `Request` for `Iron`.
///
/// `IronRequest` is a wrapper for the rust-http `Request`.
pub struct IronRequest {
    req: request::Request
}

impl Request for IronRequest {
    #[inline]
    fn headers_mut<'a>(&'a mut self) -> &'a mut Box<HeaderCollection> { &mut self.req.headers }

    #[inline]
    fn body_mut<'a>(&'a mut self) -> &'a mut String { &mut self.req.body }

    #[inline]
    fn method_mut<'a>(&'a mut self) -> &'a mut Method { &mut self.req.method }

    #[inline]
    fn uri_mut<'a>(&'a mut self) -> &'a mut RequestUri { &mut self.req.request_uri }

    #[inline]
    fn close_connection_mut<'a>(&'a mut self) -> &'a mut bool { &mut self.req.close_connection }

    #[inline]
    fn headers<'a>(&'a self) -> &'a HeaderCollection { & *self.req.headers }

    #[inline]
    fn body<'a>(&'a self) -> &'a str { self.req.body.as_slice() }

    #[inline]
    fn method<'a>(&'a self) -> &'a Method { &self.req.method }

    #[inline]
    fn uri<'a>(&'a self) -> &'a RequestUri { &self.req.request_uri }

    #[inline]
    fn close_connection<'a>(&'a self) -> bool { self.req.close_connection }

    #[inline]
    fn version(&self) -> (uint, uint) { (1, 1) }
}

impl HttpRequest for IronRequest {
    // rust-http exposes a `Request` that cannot be cloned, so `IronRequest`
    // wraps it by cloning or otherwise copying its fields to make it mutable.
    #[inline]
    fn from_http(request: &request::Request) -> IronRequest {
        IronRequest {
            req: request::Request {
                remote_addr: request.remote_addr,
                headers: request.headers.clone(),
                body: request.body.clone(),
                method: request.method.clone(),
                request_uri: match request.request_uri {
                    Star => Star,
                    AbsoluteUri(ref u) => AbsoluteUri(u.clone()),
                    AbsolutePath(ref p) => AbsolutePath(p.clone()),
                    Authority(ref s) => Authority(s.clone())
                },
                close_connection: request.close_connection,
                version: (1, 1)
            }
        }
    }
}
