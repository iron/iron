use super::response::Response;
use super::request::Request;

pub enum Status {
    Continue, Unwind
}

pub trait Ingot<Rq: Request, Rs: Response>: Send + Clone {
    fn enter(&mut self, _request: &mut Rq, _response: &mut Rs) -> Status {
        Continue
    }
    fn exit(&mut self, _request: &mut Rq, _response: &mut Rs) -> Status {
        Continue
    }
}

