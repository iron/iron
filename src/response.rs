//! Iron's HTTP Response representation and associated methods.

use std::io::{IoResult, File, MemReader};
use std::path::BytesContainer;

use http::status::{Status, InternalServerError, NotFound};
use OkStatus = http::status::Ok;
use http::headers::response::HeaderCollection;
use http::headers::content_type::MediaType;

pub use HttpResponse = http::server::response::ResponseWriter;

use contenttype::get_content_type;

/// The response representation given to `Middleware`
pub struct Response<'a, 'b> {
    http_res: &'a mut HttpResponse<'b>,

    /// The body of the response.
    ///
    /// This is a Reader for generality, most data should
    /// be sent using either `serve` or `serve_file`.
    ///
    /// Arbitrary Readers can be sent by assigning to body.
    pub body: Box<Reader>,

    /// The headers of the response.
    pub headers: Box<HeaderCollection>,

    /// The response status-code.
    pub status: Option<Status>
}

impl<'a, 'b> Response<'a, 'b> {
    /// Construct a Response from an HttpResponse reference
    pub fn from_http(http_res: &'a mut HttpResponse<'b>) -> Response<'a, 'b> {
        Response {
            headers: http_res.headers.clone(),
            status: None, // Start with no response code.
            http_res: http_res,
            body: box MemReader::new(vec![]) as Box<Reader>
        }
    }

    /// Write the `Status` and data to the `Response`.
    pub fn serve<S: BytesContainer>(&mut self, status: Status, body: S) {
        self.status = Some(status);
        self.body = box MemReader::new(body.container_as_bytes().to_vec()) as Box<Reader>;
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
        self.body = box file as Box<Reader>;
        self.status = Some(OkStatus);
        Ok(())
    }

    // `write_back` is used to put all the data added to `self`
    // back onto an `HttpResponse` so that it is sent back to the
    // client.
    //
    // `write_back` consumes the `Response`.
    #[doc(hidden)]
    pub fn write_back(mut self) {
        self.http_res.headers = self.headers.clone();

        // Default to a 404 if no response code was set
        self.http_res.status = self.status.clone().unwrap_or(NotFound);

        // Read the body into the http_res body
        let _ = match self.body.read_to_end() {
            Ok(body) => {
                let plain_txt: MediaType = get_content_type("txt").unwrap();

                // Set content length and type
                self.http_res.headers.content_length =
                    Some(body.len());
                self.http_res.headers.content_type =
                    Some(self.http_res.headers.content_type.clone().unwrap_or(plain_txt));

                // Write the body
                self.http_res.write(body.as_slice())
            },
            Err(e) => Err(e)
        // Catch errors from reading + writing
        }.map_err(|e| {
            error!("Error reading/writing body: {}", e);
            self.http_res.status = InternalServerError;
            let _ = self.http_res.write(b"Internal Server Error")
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
