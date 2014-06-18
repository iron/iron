use std::io::{IoResult, File};
use std::io::util::copy;

//! An alias of the rust-http Response struct.

pub use Response = http::server::response::ResponseWriter;

pub trait ServeFile: Writer {
    fn serve_file(&mut self, &Path) -> IoResult<()>;
}

impl<'a> ServeFile for Response<'a> {
    fn serve_file(&mut self, path: &Path) -> IoResult<()> {
        let mut file = try!(File::open(path));
        copy(&mut file, self)
    }
}
