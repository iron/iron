//! An alias of the rust-http Request struct.

use std::io::net::ip::SocketAddr;
use http::server::request::{AbsolutePath};
use http::headers::request::HeaderCollection;
use http::method::Method;
pub use HttpRequest = http::server::request::Request;

pub struct Request {
    pub url: String,

    pub remote_addr: Option<SocketAddr>,

    pub headers: Box<HeaderCollection>,

    pub body: String,

    pub method: Method,
}

    /// Get a mutable url from a Request
impl Request {
    ///
    pub fn from_http(req: HttpRequest) -> Option<Request> {
        match req.request_uri {
            AbsolutePath(path) => {
                Some(Request {
                    url: path,
                    remote_addr: req.remote_addr,
                    headers: req.headers,
                    body: req.body,
                    method: req.method
                })
            },
            _ => None
        }
    }
}

