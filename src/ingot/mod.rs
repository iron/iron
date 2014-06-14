use super::response::Response;
use super::request::Request;
use super::alloy::Alloy;

pub enum Status {
    Continue, Unwind
}

pub trait Ingot<'a, Rq: Request, Rs: Response<'a>>: Send + Clone {
    fn enter(&mut self, _request: &mut Rq, _response: &mut Rs, _alloy: &mut Alloy) -> Status {
        Continue
    }
    fn exit(&mut self, _request: &mut Rq, _response: &mut Rs, _alloy: &mut Alloy) -> Status {
        Continue
    }
    fn clone_box(&self) -> Box<Ingot<Rq, Rs>> { box self.clone() as Box<Ingot<Rq, Rs>> }
}

impl<'a, Rq: Request, Rs: Response<'a>> Clone for Box<Ingot<'a, Rq, Rs>> {
    fn clone(&self) -> Box<Ingot<Rq, Rs>> { self.clone_box() }
}
