use super::response::Response;
use super::request::Request;
use super::alloy::Alloy;
use super::ingot::Ingot;

pub mod ironfurnace;

pub trait Furnace<Rq: Request, Rs: Response>: Send + Clone {
    fn forge(&mut self, _request: &mut Rq, _response: &mut Rs, Option<&mut Alloy>);
    fn smelt<I: Ingot<Rq, Rs>>(&mut self, _ingot: I);
}

