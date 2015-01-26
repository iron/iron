//! Iron's HTTP Request representation and associated methods.

use std::io::net::ip::SocketAddr;
use std::fmt::{self, Debug};

use hyper::uri::RequestUri::{AbsoluteUri, AbsolutePath};
use hyper::header::Headers;
use hyper::method::Method;

use typemap::TypeMap;
use plugin::Extensible;

pub use hyper::server::request::Request as HttpRequest;

pub use self::url::Url;

use {Plugin, headers};

mod url;

/// The `Request` given to all `Middleware`.
///
/// Stores all the properties of the client's request plus
/// an `TypeMap` for data communication between middleware.
pub struct Request {
    /// The requested URL.
    pub url: Url,

    /// The originating address of the request.
    pub remote_addr: SocketAddr,

    /// The local address of the request.
    pub local_addr: SocketAddr,

    /// The request headers.
    pub headers: Headers,

    /// The request body.
    pub body: Vec<u8>,

    /// The request method.
    pub method: Method,

    /// Extensible storage for data passed between middleware.
    pub extensions: TypeMap
}

impl Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "Request {{"));

        try!(writeln!(f, "    url: {:?}", self.url));
        try!(writeln!(f, "    method: {:?}", self.method));
        try!(writeln!(f, "    remote_addr: {:?}", self.remote_addr));
        try!(writeln!(f, "    local_addr: {:?}", self.local_addr));
        try!(writeln!(f, "    body: {:?}", self.body));

        try!(write!(f, "}}"));
        Ok(())
    }
}

impl Request {
    /// Create a request from an HttpRequest.
    ///
    /// This constructor consumes the HttpRequest.
    pub fn from_http(mut req: HttpRequest, local_addr: SocketAddr) -> Result<Request, String> {
        let url = match req.uri {
            AbsoluteUri(ref url) => {
                match Url::from_generic_url(url.clone()) {
                    Ok(url) => url,
                    Err(e) => return Err(e)
                }
            },

            AbsolutePath(ref path) => {
                // Attempt to prepend the Host header (mandatory in HTTP/1.1)
                // FIXME: HTTPS incompatible, update when Hyper gains HTTPS support.
                let url_string = match req.headers.get::<headers::Host>() {
                    Some(ref host) => {
                        format!("http://{}:{}{}", host.hostname, local_addr.port, path)
                    },
                    None => return Err("No host specified in request".to_string())
                };

                match Url::parse(url_string.as_slice()) {
                    Ok(url) => url,
                    Err(e) => return Err(format!("Couldn't parse requested URL: {}", e))
                }
            },
            _ => return Err("Unsupported request URI".to_string())
        };

        let body = match req.read_to_end() {
            Ok(body) => body,
            Err(e) => return Err(format!("Couldn't read request body: {}", e))
        };

        Ok(Request {
            url: url,
            remote_addr: req.remote_addr,
            local_addr: local_addr,
            headers: req.headers,
            body: body,
            method: req.method,
            extensions: TypeMap::new()
        })
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

impl Plugin for Request {}

