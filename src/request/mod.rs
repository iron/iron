//! Iron's HTTP Request representation and associated methods.

use std::old_io::net::ip::SocketAddr;
use std::old_io::IoResult;
use std::fmt::{self, Debug};

use hyper::uri::RequestUri::{AbsoluteUri, AbsolutePath};
use hyper::http::HttpReader;

use typemap::TypeMap;
use plugin::Extensible;
use method::Method;

pub use hyper::server::request::Request as HttpRequest;

pub use self::url::Url;

use {Protocol, Plugin, Headers, Set, headers};

mod url;

/// The `Request` given to all `Middleware`.
///
/// Stores all the properties of the client's request plus
/// an `TypeMap` for data communication between middleware.
pub struct Request<'a> {
    /// The requested URL.
    pub url: Url,

    /// The originating address of the request.
    pub remote_addr: SocketAddr,

    /// The local address of the request.
    pub local_addr: SocketAddr,

    /// The request headers.
    pub headers: Headers,

    /// The request body as a reader.
    pub body: Body<'a>,

    /// The request method.
    pub method: Method,

    /// Extensible storage for data passed between middleware.
    pub extensions: TypeMap
}

impl<'a> Debug for Request<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "Request {{"));

        try!(writeln!(f, "    url: {:?}", self.url));
        try!(writeln!(f, "    method: {:?}", self.method));
        try!(writeln!(f, "    remote_addr: {:?}", self.remote_addr));
        try!(writeln!(f, "    local_addr: {:?}", self.local_addr));

        try!(write!(f, "}}"));
        Ok(())
    }
}

impl<'a> Request<'a> {
    /// Create a request from an HttpRequest.
    ///
    /// This constructor consumes the HttpRequest.
    pub fn from_http(req: HttpRequest<'a>, local_addr: SocketAddr, protocol: &Protocol)
                     -> Result<Request<'a>, String> {
        let (addr, method, headers, uri, _, reader) = req.deconstruct();

        let url = match uri {
            AbsoluteUri(ref url) => {
                match Url::from_generic_url(url.clone()) {
                    Ok(url) => url,
                    Err(e) => return Err(e)
                }
            },

            AbsolutePath(ref path) => {
                // Attempt to prepend the Host header (mandatory in HTTP/1.1)
                let url_string = match headers.get::<headers::Host>() {
                    Some(ref host) => {
                        format!("{}://{}:{}{}", protocol.name(), host.hostname, local_addr.port,
                                path)
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

        Ok(Request {
            url: url,
            remote_addr: addr,
            local_addr: local_addr,
            headers: headers,
            body: Body::new(reader),
            method: method,
            extensions: TypeMap::new()
        })
    }
}

/// The body of an Iron request,
pub struct Body<'a>(HttpReader<&'a mut (Reader + 'a)>);

impl<'a> Body<'a> {
    /// Create a new reader for use in an Iron request from a hyper HttpReader.
    pub fn new(reader: HttpReader<&'a mut (Reader + 'a)>) -> Body<'a> {
        Body(reader)
    }
}

impl<'a> Reader for Body<'a> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.0.read(buf)
    }
}

// Allow plugins to attach to requests.
impl<'a> Extensible for Request<'a> {
    fn extensions(&self) -> &TypeMap {
        &self.extensions
    }

    fn extensions_mut(&mut self) -> &mut TypeMap {
        &mut self.extensions
    }
}

impl<'a> Plugin for Request<'a> {}
impl<'a> Set for Request<'a> {}

