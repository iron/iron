use super::response::Response;
use super::request::Request;
use super::alloy::Alloy;

pub enum Status {
    Continue, Unwind
}

pub trait Ingot<Rq: Request, Rs: Response>: Send + Clone {
    fn enter(&mut self, _request: &mut Rq, _response: &mut Rs, _alloy: &mut Alloy) -> Status {
        Continue
    }
    fn exit(&mut self, _request: &mut Rq, _response: &mut Rs, _alloy: &mut Alloy) -> Status {
        Continue
    }
}

