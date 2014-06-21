//! Exposes the `Furnace` trait and `StackFurnace` type.

use super::response::Response;
use super::request::Request;
use super::alloy::Alloy;
use super::middleware::{Middleware, Status};

/// The default `Furnace` used by `Iron`.
pub mod stackfurnace;

/// `Furnaces` are the backbone of `Iron`. They coordinate `Middleware`
/// to ensure they are resolved and called in the right order,
/// create and distribute `Alloys`, and handle incoming requests.
///
/// `Furnaces` are internal tools. Unless you want additional
/// or unusual behavior such as enhanced debug logging you
/// probably don't need to mess with `Furnace` internals.
///
/// That being said, custom `Furnaces` are extremely powerful.
/// They allow you to completely control the resolution of `Middleware`.
pub trait Furnace: Send + Clone {
    /// `forge` will be called once per `Request`, and may be
    /// called either with or without an existing `Alloy`. `Forge` is responsible
    /// for delegating the request to the correct `Middleware` and in the correct
    /// order. Effectively, 99% of the work done by a `Furnace` is done here.
    fn forge(&mut self,
             _request: &mut Request,
             _response: &mut Response,
             Option<&mut Alloy>) -> Status;

    /// `smelt` is responsible for adding new `Middleware` to the `Furnace's` internal
    /// storage of `Middleware`. Different `Furnaces` may implement different behavior
    /// for `smelt`, but - ideally - `Middleware` added here will be delegated to during
    /// `Requests`.
    fn smelt<I: Middleware>(&mut self, _middleware: I);

    /// Create a new instance of `Furnace`.
    /// If you are making your own furnace, you'll need to
    /// pass a new instance of it to `Iron`, otherwise,
    /// this function will only be used internally.
    fn new() -> Self;
}

