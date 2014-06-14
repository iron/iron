pub use IronResponse = http::server::response::ResponseWriter;
use http::server::request::Request;
use http::headers::response::HeaderCollection;
use http::status::Status;

use super::Response;

impl<'a> Response<'a> for IronResponse<'a> {
    #[inline]
    fn headers_mut<'a>(&'a mut self) -> &'a mut Box<HeaderCollection> { &mut self.headers }
    
    #[inline]
    fn status_mut<'a>(&'a mut self) -> &'a mut Status { &mut self.status }
    
    #[inline]
    fn request<'a>(&'a self) -> &'a Request { self.request }
    
    #[inline]
    fn headers<'a>(&'a self) -> &'a HeaderCollection { & *self.headers }
    
    #[inline]
    fn status<'a>(&'a self) -> &'a Status { & self.status }
}
