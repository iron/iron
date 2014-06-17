use http::server::response::ResponseWriter;
use http::server::request::Request;
use http::headers::response::HeaderCollection;
use http::status::Status;

use std::io::IoResult;
use std::mem;

use super::{Response, HttpResponse};

/// The default `Response` for `Iron`.
///
/// `IronResponse` is a wrapper for the rust-http `ResponseWriter`.
/// It (mostly) abstracts the lifetimes necessary to store a TcpStream
/// so that `Ingots` only see a `&mut Response` generic (with no lifetimes).
pub struct IronResponse<'a, 'b> {
    writer: &'a mut ResponseWriter<'b>
}

impl <'a, 'b> Writer for IronResponse<'a, 'b> {
    // Write to the wrapped TcpStream.
    fn write(&mut self, content: &[u8]) -> IoResult<()> {
        self.writer.write(content)
    }
}

impl<'a, 'b> Response for IronResponse<'a, 'b> {
    #[inline]
    fn headers_mut<'a>(&'a mut self) -> &'a mut Box<HeaderCollection> { &mut self.writer.headers }

    #[inline]
    fn status_mut<'a>(&'a mut self) -> &'a mut Status { &mut self.writer.status }

    #[inline]
    fn request<'a>(&'a self) -> &'a Request { self.writer.request }

    #[inline]
    fn headers<'a>(&'a self) -> &'a HeaderCollection { &*self.writer.headers }

    #[inline]
    fn status<'a>(&'a self) -> &'a Status { &self.writer.status }
}

impl <'a, 'b> HttpResponse<'a, 'b> for IronResponse<'a, 'b> {
    #[inline]
    fn from_http(writer: &mut ResponseWriter) -> IronResponse<'a, 'b> {
        unsafe {
            mem::transmute(
                IronResponse {
                    writer: writer
                }
            )
        }
    }
}

