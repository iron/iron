//! Iron's HTTP Request representation and associated methods.

use std::fmt::{self, Debug};
use std::net::SocketAddr;

use futures::Stream;

use hyper::HttpVersion;

use typemap::{Key, TypeMap};
use method::Method;

pub use hyper::Body;
pub use hyper::server::Request as HttpRequest;

#[cfg(test)]
use std::net::ToSocketAddrs;

pub use self::url::Url;

use {Protocol, Plugin, Headers, Set, headers};

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

    /// The local address of the request.
    pub local_addr: SocketAddr,

    /// The request headers.
    pub headers: Headers,

    /// The request body as a reader.
    pub body: Option<Body>,

    /// The request method.
    pub method: Method,

    /// Extensible storage for data passed between middleware.
    pub extensions: TypeMap,

    /// The version of the HTTP protocol used.
    pub version: HttpVersion,

    _p: (),
}

impl Debug for Request {
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

impl Request {
    /// Create a request from an HttpRequest.
    ///
    /// This constructor consumes the HttpRequest.
    pub fn from_http(req: HttpRequest, local_addr: SocketAddr, protocol: &Protocol)
                     -> Result<Request, String> {
        let addr = req.remote_addr().cloned();
        let (method, uri, version, headers, body) = req.deconstruct();
        let url = {
            let path = uri.path();
            let url_string = match (version, headers.get::<headers::Host>()) {
                (_, Some(host)) => {
                    // Attempt to prepend the Host header (mandatory in HTTP/1.1)
                    if let Some(port) = host.port() {
                        format!("{}://{}:{}{}", protocol.name(), host.hostname(), port, path)
                    } else {
                        format!("{}://{}{}", protocol.name(), host.hostname(), path)
                    }
                },
                (v, None) if v < HttpVersion::Http11 => {
                    // Attempt to use the local address? (host header is not required in HTTP/1.0).
                    match local_addr {
                        SocketAddr::V4(addr4) => format!("{}://{}:{}{}", protocol.name(), addr4.ip(), local_addr.port(), path),
                        SocketAddr::V6(addr6) => format!("{}://[{}]:{}{}", protocol.name(), addr6.ip(), local_addr.port(), path),
                    }
                },
                (_, None) => {
                    return Err("No host specified in request".into())
                }
            };

            match Url::parse(&url_string) {
                Ok(url) => url,
                Err(e) => return Err(format!("Couldn't parse requested URL: {}", e))
            }
        };

        Ok(Request {
            url: url,
            remote_addr: addr,
            local_addr: local_addr,
            headers: headers,
            body: Some(body),
            method: method,
            extensions: TypeMap::custom(),
            version: version,
            _p: (),
        })
    }

    /// Get the contents of the body as a Vec<u8>
    ///
    /// This consumes the body future and turns it into Vec<u8>.  Note this should not be called
    /// from the main hyper thread, as it will potentially deadlock.
    pub fn get_body_contents(&mut self) -> &Vec<u8> {
        if let Some(reader) = self.body.take() {
            let body = reader.wait().fold(Vec::new(), |mut v, input| { v.extend_from_slice(&input.unwrap()); v });
            self.extensions.insert::<RequestBodyKey>(body);
        }
        self.extensions.get::<RequestBodyKey>().unwrap()
    }

    #[cfg(test)]
    pub fn stub() -> Request {
        Request {
            url: Url::parse("http://www.rust-lang.org").unwrap(),
            remote_addr: "localhost:3000".to_socket_addrs().unwrap().next(),
            local_addr: "localhost:3000".to_socket_addrs().unwrap().next().unwrap(),
            headers: Headers::new(),
            body: Some(Body::empty()),
            method: Method::Get,
            extensions: TypeMap::custom(),
            version: HttpVersion::Http11,
            _p: (),
        }
    }
}

struct RequestBodyKey;

impl Key for RequestBodyKey {
    type Value = Vec<u8>;
}

impl Plugin for Request {}
impl Set for Request {}
