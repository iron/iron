//! Iron's HTTP Response representation and associated methods.

use std::io::{IoResult, File, MemReader};
use std::path::BytesContainer;

use anymap::AnyMap;
use http::status::{Status, InternalServerError, NotFound};
use OkStatus = http::status::Ok;
use http::headers::response::HeaderCollection;
use http::headers::content_type::MediaType;

pub use HttpResponse = http::server::response::ResponseWriter;

use contenttype::get_content_type;

/// The response representation given to `Middleware`
pub struct Response {
    /// The body of the response.
    ///
    /// This is a Reader for generality, most data should
    /// be sent using either `serve` or `serve_file`.
    ///
    /// Arbitrary Readers can be sent by assigning to body.
    pub body: Option<Box<Reader>>,

    /// The headers of the response.
    pub headers: Box<HeaderCollection>,

    /// The response status-code.
    pub status: Option<Status>,

    /// An AnyMap to be used as an extensible storage for data
    /// associated with this Response.
    pub extensions: AnyMap
}

impl Response {
    /// Construct a Response from an HttpResponse reference
    pub fn from_http(http_res: &mut HttpResponse) -> Response {
        Response {
            headers: http_res.headers.clone(),
            status: None, // Start with no response code.
            body: None, // Start with no body.
            extensions: AnyMap::new()
        }
    }

    /// Write the `Status` and data to the `Response`.
    pub fn serve<S: BytesContainer>(&mut self, status: Status, body: S) {
        self.status = Some(status);
        self.body = Some(box MemReader::new(body.container_as_bytes().to_vec()) as Box<Reader>);
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
        self.body = Some(box file as Box<Reader>);
        self.status = Some(OkStatus);
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
        http_res.status = self.status.clone().unwrap_or(NotFound);

        // Read the body into the http_res body
        let mut body = self.body.unwrap_or_else(|| box MemReader::new(vec![]) as Box<Reader>);
        let _ = match body.read_to_end() {
            Ok(body_content) => {
                let plain_txt: MediaType = get_content_type("txt").unwrap();

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
            http_res.status = InternalServerError;
            let _ = http_res.write(b"Internal Server Error")
                // Something is really, really wrong.
                .map_err(|e| error!("Error writing error message: {}", e));
        });
    }
}

#[test]
fn matches_content_type () {
    let path = &Path::new("test.txt");
    let content_type = path.extension_str().and_then(get_content_type).unwrap();

    assert_eq!(content_type.type_.as_slice(), "text");
    assert_eq!(content_type.subtype.as_slice(), "plain");
}
