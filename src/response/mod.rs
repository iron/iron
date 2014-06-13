use http::server::response::ResponseWriter;
use http::server::request::Request;
use http::headers::response::HeaderCollection;
use http::status::Status;

pub trait Response: Writer {
    fn request_mut<'a>(&'a self) -> &'a mut Request;
    fn headers_mut<'a>(&'a self) -> &'a mut Box<HeaderCollection>;
    fn status_mut<'a>(&'a self) -> &'a mut Status;

    fn request<'a>(&'a self) -> &'a Request;
    fn headers<'a>(&'a self) -> &'a HeaderCollection;
    fn status<'a>(&'a self) -> Status;

    fn from_http(&ResponseWriter) -> Self;
    fn to_http(&self, &mut ResponseWriter);
}
