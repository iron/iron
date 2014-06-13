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
    fn clone_box(&self) -> Box<Ingot<Rq, Rs>> { box self.clone() as Box<Ingot<Rq, Rs>> }
}

impl<Rq: Request, Rs: Response> Clone for Box<Ingot<Rq, Rs>> {
    fn clone(&self) -> Box<Ingot<Rq, Rs>> { self.clone_box() }
}
