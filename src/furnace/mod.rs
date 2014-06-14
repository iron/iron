use super::response::Response;
use super::request::Request;
use super::alloy::Alloy;
use super::ingot::Ingot;

pub mod ironfurnace;

/// `Furnaces` are the backbone of `Iron`. They coordinate `Ingots`
/// and ensure they are resolved and called in the right order,
/// create and distribute `Alloys`, and handle incoming requests.
///
/// `Furnaces` are internal tools. Unless you want additional
/// or unusual behavior such as enhanced debug logging you
/// probably don't need to mess with `Furnace` internals.
///
/// That being said, custom `Furnaces` are extremely powerful as
/// they allow you to completely control the resolution of `Ingots`.
///
/// The default `Furnace` used in `Iron` is the `IronFurnace`.
pub trait Furnace<'a, Rq: Request, Rs: Response<'a>>: Send + Clone {
    /// A `Furnace's` forge method will get called once per Request, and may be
    /// called either with or without an existing `Alloy`. `Forge` is responsible
    /// for delegating the request to the correct `Ingots` and in the correct
    /// order. Effectively, 99% of the work done by a `Furnace` is done here.
    fn forge(&mut self, _request: &mut Rq, _response: &mut Rs, Option<&mut Alloy>);

    /// `Smelt` is responsible for adding an `Ingot` to the `Furnace's` internal
    /// storage of `Ingots`. Different `Furnaces` may implement different behavior
    /// for smelt, but ideally an `Ingot` added here will be delegated to during
    /// Requests.
    fn smelt<I: Ingot<'a, Rq, Rs>>(&mut self, _ingot: I);
}

