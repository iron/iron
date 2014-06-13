use super::response::Response;
use super::request::Request;
use super::alloy::Alloy;

/// The `Status` returned by `Ingot`.
///
/// This is used to direct the `Furnace` up or down the stack.
pub enum Status {
    /// `Continue` directs `Furnace` to continue down the stack of `Ingots`
    /// (continue down the middleware stack).
    Continue,
    /// `Unwind` directs `Furnace` to stop executing new `Ingots` and reverse
    /// up through the executed `Ingots`
    /// (unwind the executed middleware stack).
    Unwind
}

/// `impl Ingot` for an atomic instance of middleware.
///
/// `Ingots` can be `smelt`ed in a `Furnace` to add them to the middleware
/// stack. All `Requests` and `Responses` will then be routed through that
/// stack until it is unwound.
/// `Ingot`s are expected to store data either on `self` or on `Alloy` to be
/// accessable by other `Ingots`.
pub trait Ingot<Rq: Request, Rs: Response>: Send + Clone {
    /// Function to execute on entering the middleware stack.
    ///
    /// This is called when the `Request` and `Response` first see the `Ingot`.
    /// For example, a timing `Ingot` might record start time on `self`
    /// at this point.
    fn enter(&mut self, _request: &mut Rq, _response: &mut Rs, _alloy: &mut Alloy) -> Status {
        Continue
    }
    /// Function to execute on unwinding the middleware stack.
    ///
    /// This is called when the `Request` and `Response` are returned to
    /// `Ingot` as the stack is being unwound.
    /// For example, a timing `Ingot` might calculate elapsed time based on
    /// the previously recorded start time, and record the elapsed time in
    /// `Alloy`.
    fn exit(&mut self, _request: &mut Rq, _response: &mut Rs, _alloy: &mut Alloy) -> Status {
        Continue
    }
    /// This is necessary to `impl Clone`.
    fn clone_box(&self) -> Box<Ingot<Rq, Rs>> { box self.clone() as Box<Ingot<Rq, Rs>> }
}

impl<Rq: Request, Rs: Response> Clone for Box<Ingot<Rq, Rs>> {
    fn clone(&self) -> Box<Ingot<Rq, Rs>> { self.clone_box() }
}
