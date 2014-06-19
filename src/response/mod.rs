//! An augmentation of the rust-http Response struct.

use std::io::{IoResult, File};
use std::io::util::copy;

use http::status::Status;

pub use Response = http::server::response::ResponseWriter;

use self::mimes::get_content_type;

mod mimes;

/// Allow file-serving
pub trait ServeFile: Writer {
    /// Serve the file located at `path`.
    ///
    /// This is usually a terminal process, and `Middleware` may want to
    /// call `Unwind` after a file is served. If the status should be
    /// anything other than `200`, the `Middleware` must set it, including in
    /// the case of an `Err`.
    ///
    /// `serve_file` will err out if the file does not exist, the process
    /// does not have correct permissions, or it has other issues in reading
    /// from the file. Middleware should handle this gracefully.
    fn serve_file(&mut self, &Path) -> IoResult<()>;
}

impl<'a> ServeFile for Response<'a> {
    fn serve_file(&mut self, path: &Path) -> IoResult<()> {
        let mut file = try!(File::open(path));
        self.headers.content_type = get_content_type(path);
        copy(&mut file, self)
    }
}

/// Send data with a statuscode quickly.
pub trait SendData {
    /// Write the statuscode and data to the response.
    ///
    /// send will forward write errors to the request to its
    /// caller.
    fn send(&mut self, status: Status, body: &str) -> IoResult<()>;
}

impl<'a> SendData for Response<'a> {
    fn send(&mut self, status: Status, body: &str) -> IoResult<()> {
        self.status = status;
        Ok(try!(self.write(body.as_bytes())))
    }
}

