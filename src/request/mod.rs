//! Iron's HTTP Request representation and associated methods.

use std::io::net::ip::SocketAddr;
use std::fmt::{Show, Formatter, FormatError};

use http::server::request::{AbsoluteUri, AbsolutePath};
use http::headers::request::HeaderCollection;
use http::method::Method;

use typemap::TypeMap;
use plugin::Extensible;

pub use http::server::request::Request as HttpRequest;

pub use self::url::Url;

mod url;

/// The `Request` given to all `Middleware`.
///
/// Stores all the properties of the client's request plus
/// an `TypeMap` for data communication between middleware.
pub struct Request {
    /// The requested URL.
    pub url: Url,

    /// The originating address of the request.
    pub remote_addr: Option<SocketAddr>,

    /// The request headers.
    pub headers: Box<HeaderCollection>,

    /// The request body.
    pub body: String,

    /// The request method.
    pub method: Method,

    /// Extensible storage for data passed between middleware.
    pub extensions: TypeMap
}

impl Show for Request {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        try!(writeln!(f, "Request {{"));

        try!(writeln!(f, "    url: {}", self.url));
        try!(writeln!(f, "    method: {}", self.method));
        try!(writeln!(f, "    remote_addr: {}", self.remote_addr));
        try!(writeln!(f, "    body: {}", self.body));

        try!(write!(f, "}}"));
        Ok(())
    }
}

impl Request {
    /// Create a request from an HttpRequest.
    ///
    /// This constructor consumes the HttpRequest.
    pub fn from_http(req: HttpRequest) -> Result<Request, String> {
        match req.request_uri {
            AbsoluteUri(url) => {
                let url = match Url::from_generic_url(url) {
                    Ok(url) => url,
                    Err(e) => return Err(e)
                };

                Ok(Request {
                    url: url,
                    remote_addr: req.remote_addr,
                    headers: req.headers,
                    body: req.body,
                    method: req.method,
                    extensions: TypeMap::new()
                })
            },
            AbsolutePath(path) => {
                // Attempt to prepend the Host header (mandatory in HTTP/1.1)
                // XXX: HTTPS incompatible, update when switching to Teepee.
                let url_string = match req.headers.host {
                    Some(ref host) => format!("http://{}{}", host, path),
                    None => return Err("No host specified in request".to_string())
                };

                let url = match Url::parse(url_string.as_slice()) {
                    Ok(url) => url,
                    Err(e) => return Err(format!("Couldn't parse requested URL: {}", e))
                };

                Ok(Request {
                    url: url,
                    remote_addr: req.remote_addr,
                    headers: req.headers,
                    body: req.body,
                    method: req.method,
                    extensions: TypeMap::new()
                })
            },
            _ => Err("Unsupported request URI".to_string())
        }
    }
}

// Allow plugins to attach to requests.
impl Extensible for Request {
    fn extensions(&self) -> &TypeMap {
        &self.extensions
    }

    fn extensions_mut(&mut self) -> &mut TypeMap {
        &mut self.extensions
    }
}

