//! Iron's HTTP Request representation and associated methods.
use std::fmt::{self, Debug};
use std::net::SocketAddr;

use futures::Stream;

use http;
use http::version::Version as HttpVersion;

use method::Method;
use plugin::Extensible;
use typemap::{Key, TypeMap};

pub use hyper::Body;
pub use hyper::Request as HttpRequest;

#[cfg(test)]
use std::net::ToSocketAddrs;

pub use self::url::Url;

use error::HttpError;
use headers::{self, HeaderMap};
use {Plugin, Protocol, Set};

mod url;

/// The `Request` given to all `Middleware`.
///
/// Stores all the properties of the client's request plus
/// an `TypeMap` for data communication between middleware.
pub struct Request {
    /// The requested URL.
    pub url: Url,

    /// The local address of the request.
    pub local_addr: Option<SocketAddr>,

    /// The request headers.
    pub headers: HeaderMap,

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
        writeln!(f, "Request {{")?;

        writeln!(f, "    url: {:?}", self.url)?;
        writeln!(f, "    method: {:?}", self.method)?;
        writeln!(f, "    local_addr: {:?}", self.local_addr)?;

        write!(f, "}}")?;
        Ok(())
    }
}

impl Request {
    /// Create a request from an HttpRequest.
    ///
    /// This constructor consumes the HttpRequest.
    pub fn from_http(
        req: HttpRequest<Body>,
        local_addr: Option<SocketAddr>,
        protocol: &Protocol,
    ) -> Result<Request, String> {
        let (
            http::request::Parts {
                method,
                uri,
                version,
                headers,
                ..
            },
            body,
        ) = req.into_parts();

        let url = {
            let path = uri.path_and_query().expect("expected path and query but found None").as_str();

            let query = uri.query();

            let mut socket_ip = String::new();
            let (host, port) = if let Some(host) = uri.host() {
                (host, uri.port_part().map(|p| p.as_u16()))
            } else if let Some(host) = headers.get(headers::HOST).and_then(|h| h.to_str().ok()) {
                let mut parts = host.split(':');
                let hostname = parts.next().unwrap();
                let port = parts.next().and_then(|p| p.parse::<u16>().ok());
                (hostname, port)
            } else if version < HttpVersion::HTTP_11 {
                if let Some(local_addr) = local_addr {
                    match local_addr {
                        SocketAddr::V4(addr4) => socket_ip.push_str(&format!("{}", addr4.ip())),
                        SocketAddr::V6(addr6) => socket_ip.push_str(&format!("[{}]", addr6.ip())),
                    }
                    (socket_ip.as_ref(), Some(local_addr.port()))
                } else {
                    return Err("No fallback host specified".into());
                }
            } else {
                return Err("No host specified in request".into());
            };

            let url_string = if let Some(port) = port {
                if let Some(query) = query {
                    format!("{}://{}:{}{}?{}", protocol.name(), host, port, path, query)
                } else {
                    format!("{}://{}:{}{}", protocol.name(), host, port, path)
                }
            } else {
                if let Some(query) = query {
                    format!("{}://{}{}?{}", protocol.name(), host, path, query)
                } else {
                    format!("{}://{}{}", protocol.name(), host, path)  
                }
            };

            match Url::parse(&url_string) {
                Ok(url) => url,
                Err(e) => return Err(format!("Couldn't parse requested URL: {}", e)),
            }
        };

        Ok(Request {
            url,
            local_addr,
            headers,
            body: Some(body),
            method,
            extensions: TypeMap::new(),
            version,
            _p: (),
        })
    }

    /// Get the contents of the body as a Vec<u8>
    ///
    /// This consumes the body future and turns it into Vec<u8>.  Note this should not be called
    /// from the main hyper thread, as it will potentially deadlock.
    pub fn get_body_contents(&mut self) -> Result<&Vec<u8>, HttpError> {
        if let Some(reader) = self.body.take() {
            let body = reader.wait().fold(Ok(Vec::new()), |r, input| {
                if let Ok(mut v) = r {
                    input.map(move |next_body_chunk| {
                        v.extend_from_slice(&next_body_chunk);
                        v
                    })
                } else {
                    r
                }
            });
            match body {
                Ok(body) => self.extensions.insert::<RequestBodyKey>(body),
                Err(e) => return Err(e),
            };
        }
        Ok(self.extensions.get::<RequestBodyKey>().unwrap())
    }

    #[cfg(test)]
    pub fn stub() -> Request {
        Request {
            url: Url::parse("http://www.rust-lang.org").unwrap(),
            local_addr: "localhost:3000".to_socket_addrs().unwrap().next(),
            headers: HeaderMap::new(),
            body: Some(Body::empty()),
            method: Method::GET,
            extensions: TypeMap::new(),
            version: HttpVersion::HTTP_11,
            _p: (),
        }
    }
}

struct RequestBodyKey;

impl Key for RequestBodyKey {
    type Value = Vec<u8>;
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
impl Set for Request {}

#[cfg(test)]
mod test {
    use super::*;

    use headers;

    use url_ext::Host::*;

    #[test]
    fn test_request_parse_absolute_uri() {
        let mut hyper_request = HttpRequest::new(Body::empty());
        *hyper_request.method_mut() = Method::GET;
        *hyper_request.uri_mut() = "http://my-host/path".parse().unwrap();

        let iron_request = Request::from_http(hyper_request, None, &Protocol::http())
            .expect("A valid Iron request");

        assert_eq!(iron_request.url.host(), Domain("my-host"));
    }

    #[test]
    fn test_request_with_query_string() {
        let mut hyper_request = HttpRequest::new(Body::empty());
        *hyper_request.uri_mut() = "http://my-host/path?param=value".parse().unwrap();

        let iron_request = Request::from_http(hyper_request, None, &Protocol::http())
            .expect("A valid Iron request");

        assert_eq!(iron_request.url.query(), Some("param=value"));
    }

    #[test]
    fn test_request_parse_host_header_only() {
        let mut hyper_request = HttpRequest::new(Body::empty());
        *hyper_request.method_mut() = Method::GET;
        *hyper_request.uri_mut() = "/path".parse().unwrap();
        hyper_request
            .headers_mut()
            .insert(headers::HOST, "my-host".parse().unwrap());

        let iron_request = Request::from_http(hyper_request, None, &Protocol::http())
            .expect("A valid Iron request");

        assert_eq!(iron_request.url.host(), Domain("my-host"));
    }

    #[test]
    fn test_request_parse_host_header_and_absolute_uri() {
        let mut hyper_request = HttpRequest::new(Body::empty());
        *hyper_request.method_mut() = Method::GET;
        *hyper_request.uri_mut() = "http://my-host-uri/path".parse().unwrap();
        hyper_request
            .headers_mut()
            .insert(headers::HOST, "my-host-header".parse().unwrap());

        let iron_request = Request::from_http(hyper_request, None, &Protocol::http())
            .expect("A valid Iron request");

        assert_eq!(iron_request.url.host(), Domain("my-host-uri"));
    }

    #[test]
    fn test_request_parse_ipv4_socket_only() {
        let mut hyper_request = HttpRequest::new(Body::empty());
        *hyper_request.method_mut() = Method::GET;
        *hyper_request.uri_mut() = "/path".parse().unwrap();
        *hyper_request.version_mut() = HttpVersion::HTTP_10;

        let socket_addr = Some("1.2.3.4:80".parse().unwrap());
        let iron_request = Request::from_http(hyper_request, socket_addr, &Protocol::http())
            .expect("A valid Iron request");

        assert_eq!(iron_request.url.host(), Ipv4([1, 2, 3, 4].into()));
    }

    #[test]
    fn test_request_parse_ipv6_socket_only() {
        let mut hyper_request = HttpRequest::new(Body::empty());
        *hyper_request.method_mut() = Method::GET;
        *hyper_request.uri_mut() = "/path".parse().unwrap();
        *hyper_request.version_mut() = HttpVersion::HTTP_10;

        let socket_addr = Some("[1:2:3:4:5:6:7:8]:80".parse().unwrap());
        let iron_request = Request::from_http(hyper_request, socket_addr, &Protocol::http())
            .expect("A valid Iron request");

        assert_eq!(
            iron_request.url.host(),
            Ipv6([1, 2, 3, 4, 5, 6, 7, 8].into())
        );
    }

    #[test]
    fn test_request_parse_host_header_ipv4_socket_and_absolute_uri() {
        let mut hyper_request = HttpRequest::new(Body::empty());
        *hyper_request.method_mut() = Method::GET;
        *hyper_request.uri_mut() = "http://my-host-uri/path".parse().unwrap();
        hyper_request
            .headers_mut()
            .insert(headers::HOST, "my-host-header".parse().unwrap());

        let socket_addr = Some("1.2.3.4:80".parse().unwrap());
        let iron_request = Request::from_http(hyper_request, socket_addr, &Protocol::http())
            .expect("A valid Iron request");

        assert_eq!(iron_request.url.host(), Domain("my-host-uri"));
    }
}
