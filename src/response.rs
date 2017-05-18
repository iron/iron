//! Iron's HTTP Response representation and associated methods.

use std::io::{self, Write};
use std::fmt::{self, Debug};
use std::fs::File;

use typemap::TypeMap;
use modifier::{Set, Modifier};
use hyper::header::Headers;

use status::{self, Status};
use {Plugin, headers};

pub use hyper::server::Response as HttpResponse;
use hyper::Body;

/// Wrapper type to set `Read`ers as response bodies
pub struct BodyReader<R: Send>(pub R);

/// A trait which writes the body of an HTTP response.
pub trait WriteBody: Send {
    /// Writes the body to the provided `Write`.
    fn write_body(&mut self, res: &mut Write) -> io::Result<()>;
}

impl WriteBody for String {
    fn write_body(&mut self, res: &mut Write) -> io::Result<()> {
        self.as_bytes().write_body(res)
    }
}

impl<'a> WriteBody for &'a str {
    fn write_body(&mut self, res: &mut Write) -> io::Result<()> {
        self.as_bytes().write_body(res)
    }
}

impl WriteBody for Vec<u8> {
    fn write_body(&mut self, res: &mut Write) -> io::Result<()> {
        res.write_all(self)
    }
}

impl<'a> WriteBody for &'a [u8] {
    fn write_body(&mut self, res: &mut Write) -> io::Result<()> {
        res.write_all(self)
    }
}

impl WriteBody for File {
    fn write_body(&mut self, res: &mut Write) -> io::Result<()> {
        io::copy(self, res).map(|_| ())
    }
}

impl WriteBody for Box<io::Read + Send> {
    fn write_body(&mut self, res: &mut Write) -> io::Result<()> {
        io::copy(self, res).map(|_| ())
    }
}

impl <R: io::Read + Send> WriteBody for BodyReader<R> {
    fn write_body(&mut self, res: &mut Write) -> io::Result<()> {
        io::copy(&mut self.0, res).map(|_| ())
    }
}

/* Needs specialization :(
impl<R: Read + Send> WriteBody for R {
    fn write_body(&mut self, res: &mut Write) -> io::Result<()> {
        io::copy(self, res)
    }
}
*/

/// The response representation given to `Middleware`
pub struct Response {
    /// The response status-code.
    pub status: Option<Status>,

    /// The headers of the response.
    pub headers: Headers,

    /// A TypeMap to be used as an extensible storage for data
    /// associated with this Response.
    pub extensions: TypeMap,

    /// The body of the response.
    pub body: Option<Box<WriteBody>>
}

impl Response {
    /// Construct a blank Response
    pub fn new() -> Response {
        Response {
            status: None, // Start with no response code.
            body: None, // Start with no body.
            headers: Headers::new(),
            extensions: TypeMap::custom()
        }
    }

    /// Construct a Response with the specified modifier pre-applied.
    pub fn with<M: Modifier<Response>>(m: M) -> Response {
        Response::new().set(m)
    }

    // `write_back` is used to put all the data added to `self`
    // back onto an `HttpResponse` so that it is sent back to the
    // client.
    //
    // `write_back` consumes the `Response`.
    #[doc(hidden)]
    pub fn write_back(self, http_res: &mut HttpResponse<Body>) {
        *http_res.headers_mut() = self.headers;

        // Default to a 404 if no response code was set
        http_res.set_status(self.status.unwrap_or(status::NotFound));

        let out = match self.body {
            Some(body) => write_with_body(http_res, body),
            None => {
                http_res.headers_mut().set(headers::ContentLength(0));
                Ok(())
            }
        };

        if let Err(e) = out {
            error!("Error writing response: {}", e);
        }
    }
}

fn write_with_body(res: &mut HttpResponse<Body>, mut body: Box<WriteBody>)
                   -> io::Result<()> {
    let content_type = res.headers().get::<headers::ContentType>()
                           .map_or_else(|| headers::ContentType("text/plain".parse().unwrap()),
                                        |cx| cx.clone());
    res.headers_mut().set(content_type);

    let mut body_contents: Vec<u8> = vec![];
    try!(body.write_body(&mut body_contents));
    res.set_body(Body::from(body_contents));
    Ok(())
}

impl Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "HTTP/1.1 {}\n{}",
            self.status.unwrap_or(status::NotFound),
            self.headers
        )
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Plugin for Response {}
impl Set for Response {}
