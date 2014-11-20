//! Iron's HTTP Response representation and associated methods.

use std::io::{mod, File, MemReader};
use std::path::BytesContainer;
use std::fmt::{Show, Formatter, FormatError};

use typemap::TypeMap;
use plugin::Extensible;

use http::headers::response::HeaderCollection;
use http::headers::content_type::MediaType;

use errors::FileError;
use status::{mod, Status};
use {Url};

pub use http::server::response::ResponseWriter as HttpResponse;

use content_type::get_content_type;

pub mod modifiers;

/// The response representation given to `Middleware`
pub struct Response {
    /// The body of the response.
    ///
    /// This is a Reader for generality, most data should
    /// be sent using either `serve` or `serve_file`.
    ///
    /// Arbitrary Readers can be sent by assigning to body.
    pub body: Option<Box<Reader + Send>>,

    /// The headers of the response.
    pub headers: Box<HeaderCollection>,

    /// The response status-code.
    pub status: Option<Status>,

    /// A TypeMap to be used as an extensible storage for data
    /// associated with this Response.
    pub extensions: TypeMap
}

impl Response {
    /// Construct a blank Response
    pub fn new() -> Response {
        Response {
            headers: box HeaderCollection::new(),
            status: None, // Start with no response code.
            body: None, // Start with no body.
            extensions: TypeMap::new()
        }
    }

    /// Create a new response with the status.
    #[deprecated = "Use `Response::new().set(Status(status))` instead."]
    pub fn status(status: status::Status) -> Response {
        Response {
            body: None,
            headers: box HeaderCollection::new(),
            status: Some(status),
            extensions: TypeMap::new()
        }
    }


    /// Create a new response with the specified body and status.
    #[deprecated = "Use `Response::new().set(Status(status)).set(Body(body))` instead."]
    pub fn with<B: BytesContainer>(status: status::Status, body: B) -> Response {
        Response {
            body: Some(box MemReader::new(body.container_as_bytes().to_vec()) as Box<Reader + Send>),
            headers: box HeaderCollection::new(),
            status: Some(status),
            extensions: TypeMap::new()
        }
    }

    /// Create a new Response with the `location` header set to the specified url.
    #[deprecated = "Use `Response::new().set(Status(status)).set(Redirect(url))` instead."]
    pub fn redirect(status: status::Status, url: Url) -> Response {
        let mut headers = box HeaderCollection::new();
        headers.location = Some(url.into_generic_url());
        Response {
            body: None,
            headers: headers,
            status: Some(status),
            extensions: TypeMap::new()
        }
    }

    /// Create a response from a file on disk.
    ///
    /// The status code is set to 200 OK and the content type is autodetected based on
    /// the file extension.
    #[deprecated = "Use `Response::new().set(Status(status)).set(Body(path))` instead"]
    pub fn from_file(path: &Path) -> Result<Response, FileError> {
        let file = try!(File::open(path).map_err(FileError::new));
        let mut response = Response::new();
        response.status = Some(status::Ok);
        response.body = Some(box file as Box<Reader + Send>);
        response.headers.content_type = path.extension_str().and_then(get_content_type);
        Ok(response)
    }

    // `write_back` is used to put all the data added to `self`
    // back onto an `HttpResponse` so that it is sent back to the
    // client.
    //
    // `write_back` consumes the `Response`.
    #[doc(hidden)]
    pub fn write_back(self, http_res: &mut HttpResponse) {
        http_res.headers = *self.headers.clone();

        // Default to a 404 if no response code was set
        http_res.status = self.status.clone().unwrap_or(status::NotFound);

        let out = match self.body {
            Some(mut body) => {
                http_res.headers.content_type =
                    Some(http_res.headers
                            .content_type
                            .clone()
                            .unwrap_or_else(||
                                MediaType::new("text".into_string(),
                                               "plain".into_string(),
                                               vec![])
                            ));

                // FIXME: Manually inlined io::util::copy
                // because Box<Reader + Send> does not impl Reader.
                let mut buf = &mut [0, ..1024 * 64];
                let mut out = Ok(());
                loop {
                    let len = match body.read(buf) {
                        Ok(len) => len,
                        Err(ref e) if e.kind == io::EndOfFile => break,
                        Err(e) => { out = Err(e); break; },
                    };

                    match http_res.write(buf[..len]) {
                        Err(e) => {
                            out = Err(e);
                            break;
                        },
                        _ => {}
                    };
                }

                out.and_then(|_| http_res.finish_response())
            },

            None => {
                http_res.headers.content_length = Some(0u);
                http_res.write_headers()
                    .and_then(|_| http_res.finish_response())
            }
        };

        match out {
            Err(e) => {
                error!("Error reading/writing body: {}", e);

                // Can't do anything else here since all headers/status have
                // already been sent.
            },
            _ => {}
        }
    }
}

impl Show for Response {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        try!(writeln!(f, "Response {{"));

        try!(writeln!(f, "    status: {}", self.status));

        try!(write!(f, "}}"));
        Ok(())
    }
}

// Allow plugins to attach to responses.
impl Extensible for Response {
    fn extensions(&self) -> &TypeMap {
        &self.extensions
    }

    fn extensions_mut(&mut self) -> &mut TypeMap {
        &mut self.extensions
    }
}

#[test]
fn matches_content_type () {
    let path = &Path::new("test.txt");
    let content_type = path.extension_str().and_then(get_content_type).unwrap();

    assert_eq!(content_type.type_.as_slice(), "text");
    assert_eq!(content_type.subtype.as_slice(), "plain");
}
