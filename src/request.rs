//! Iron's HTTP Request representation and associated methods.

use std::io::net::ip::SocketAddr;
use http::server::request::AbsolutePath;
use http::headers::request::HeaderCollection;
use http::method::Method;
pub use HttpRequest = http::server::request::Request;

use super::alloy::Alloy;

/// The `Request` given to all `Middleware`
pub struct Request {
    /// The requested url
    pub url: String,

    /// The originating address of the request.
    pub remote_addr: Option<SocketAddr>,

    /// The request headers
    pub headers: Box<HeaderCollection>,

    /// The request body
    pub body: String,

    /// The request method
    pub method: Method,

    /// Data storage for extensions to the request object.
    pub alloy: Alloy
}

impl Request {
    /// Create a request from an HttpRequest.
    ///
    /// This constructor consumes the HttpRequest.
    pub fn from_http(req: HttpRequest) -> Option<Request> {
        match req.request_uri {
            AbsolutePath(path) => {
                Some(Request {
                    url: path,
                    remote_addr: req.remote_addr,
                    headers: req.headers,
                    body: req.body,
                    method: req.method,
                    alloy: Alloy::new()
                })
            },
            _ => None
        }
    }
}

