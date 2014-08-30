//! Iron's HTTP Response representation and associated methods.

use std::io::{IoResult, File, MemReader};
use std::path::BytesContainer;

use typemap::TypeMap;
use plugin::Extensible;
use super::status;
use super::status::Status;
use http::headers::response::HeaderCollection;
use http::headers::content_type::MediaType;

pub use http::server::response::ResponseWriter as HttpResponse;

use contenttype::get_content_type;

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
    pub fn status(status: status::Status) -> Response {
        Response {
            body: None,
            headers: box HeaderCollection::new(),
            status: Some(status),
            extensions: TypeMap::new()
        }
    }


    /// Create a new response with the specified body and status.
    pub fn with<B: BytesContainer>(status: status::Status, body: B) -> Response {
        Response {
            body: Some(box MemReader::new(body.container_as_bytes().to_vec()) as Box<Reader + Send>),
            headers: box HeaderCollection::new(),
            status: Some(status),
            extensions: TypeMap::new()
        }
    }

    /// Write the `Status` and data to the `Response`.
    pub fn serve<S: BytesContainer>(&mut self, status: Status, body: S) {
        self.status = Some(status);
        self.body = Some(box MemReader::new(body.container_as_bytes().to_vec()) as Box<Reader + Send>);
    }

    /// Serve the file located at `path`.
    ///
    /// This usually means a request has been handled, and `Middleware`
    /// may want to `Unwind` after a file is served. If the status should be
    /// anything other than `200`, `Middleware` must set it, including in
    /// the case of an `Err`.
    ///
    /// `serve_file` will error if the file does not exist, the process
    /// does not have correct permissions, or it has other issues in reading
    /// from the file. `Middleware` should handle this gracefully.
    pub fn serve_file(&mut self, path: &Path) -> IoResult<()> {
        let file = try!(File::open(path));
        self.headers.content_type = path.extension_str().and_then(get_content_type);
        self.body = Some(box file as Box<Reader + Send>);
        self.status = Some(status::Ok);
        Ok(())
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
