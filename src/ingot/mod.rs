use super::response::Response;
use super::request::Request;
use super::alloy::Alloy;

/// The Status returned by `Ingot's` `enter` and `exit` methods. This indicates
/// to the `Furnace` whether this is a terminal `Ingot` or whether to continue
/// passing the `Request` and `Response` down the `Furnace's` stack.
///
/// Most `Furnaces` will ignore the returned `Status` from the `exit` method of
/// `Ingots`.
pub enum Status {
    /// `Continue` indicates that this is an intermediate `Ingot` in the stack
    /// and the `Furnace` should continue passing requests down the `Furnace's`
    /// stack.
    ///
    /// Most `Ingots` will return `Continue` from both `enter` and `exit`.
    Continue,

    /// `Unwind` indicates that this is a terminal `Ingot` or something went
    /// wrong. It can be used to immediately stop passing requests down the
    /// `Furnace's` stack and start calling `exit` of all previous `Ingots`.
    ///
    /// For instance, an authorization `Ingot` might return `Unwind` if the
    /// request fails an authentication check, and `Continue` otherwise.
    Unwind
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
