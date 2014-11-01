//! Iron's HTTP Response representation and associated methods.

use std::io::{File, MemReader};
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
    pub headers: HeaderCollection,

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
            headers: HeaderCollection::new(),
            status: None, // Start with no response code.
            body: None, // Start with no body.
            extensions: TypeMap::new()
        }
    }

    /// Create a new response with the status.
    pub fn status(status: status::Status) -> Response {
        Response {
            body: None,
            headers: HeaderCollection::new(),
            status: Some(status),
            extensions: TypeMap::new()
        }
    }


    /// Create a new response with the specified body and status.
    pub fn with<B: BytesContainer>(status: status::Status, body: B) -> Response {
        Response {
            body: Some(box MemReader::new(body.container_as_bytes().to_vec()) as Box<Reader + Send>),
            headers: HeaderCollection::new(),
            status: Some(status),
            extensions: TypeMap::new()
        }
    }

    /// Create a new response with the specified body, status and media type.
    pub fn with_media_type<B: BytesContainer>(status: status::Status, body: B, media_type: MediaType) -> Response {

        let mut headers = HeaderCollection::new();
        headers.content_type = Some(media_type);

        Response {
            body: Some(box MemReader::new(body.container_as_bytes().to_vec()) as Box<Reader + Send>),
            headers: headers,
            status: Some(status),
            extensions: TypeMap::new()
        }

/*        Response {
            body: Some(box MemReader::new(body.container_as_bytes().to_vec()) as Box<Reader + Send>),
            headers: HeaderCollection::new(),
            status: Some(status),
            extensions: TypeMap::new()
        }*/
    }    

    /// Create a new Response with the `location` header set to the specified url.
    pub fn redirect(status: status::Status, url: Url) -> Response {
        let mut headers = HeaderCollection::new();
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
        http_res.headers = self.headers.clone();

        // Default to a 404 if no response code was set
        http_res.status = self.status.clone().unwrap_or(status::NotFound);

        // Read the body into the http_res body
        let mut body = self.body.unwrap_or_else(|| box MemReader::new(vec![]) as Box<Reader + Send>);
        let _ = match body.read_to_end() {
            Ok(body_content) => {
                let plain_txt = MediaType {
                    type_: "text".to_string(),
                    subtype: "plain".to_string(),
                    parameters: vec![]
                };

                // Set content length and type
                http_res.headers.content_length =
                    Some(body_content.len());
                http_res.headers.content_type =
                    Some(http_res.headers.content_type.clone().unwrap_or(plain_txt));

                // Write the body
                http_res.write(body_content.as_slice())
            },
            Err(e) => Err(e)
        // Catch errors from reading + writing
        }.map_err(|e| {
            error!("Error reading/writing body: {}", e);
            http_res.status = status::InternalServerError;
            let _ = http_res.write(b"Internal Server Error")
                // Something is really, really wrong.
                .map_err(|e| error!("Error writing error message: {}", e));
        });
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

#[test]
fn content_type () {
    let json_media_type = MediaType::new("application".to_string(), "json".to_string(), Vec::new());

    let response = Response::with_media_type(status::Ok, "response_body".to_string(), json_media_type);
    let content_type = response.headers.content_type.unwrap();

    assert_eq!(content_type.type_.as_slice(), "application");
    assert_eq!(content_type.subtype.as_slice(), "json");
}

