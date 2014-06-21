//! Exposes the `chain` trait and `StackFurnace` type.

use super::response::Response;
use super::request::Request;
use super::alloy::Alloy;
use super::middleware::{Middleware, Status, Continue};

/// The default `chain` used by `Iron`.
pub mod stackchain;

/// `chains` are the backbone of `Iron`. They coordinate `Middleware`
/// to ensure they are resolved and called in the right order,
/// create and distribute `Alloys`, and handle incoming requests.
///
/// `chains` are internal tools. Unless you want additional
/// or unusual behavior such as enhanced debug logging you
/// probably don't need to mess with `chain` internals.
///
/// That being said, custom `chains` are extremely powerful as they
/// allow you to completely control the resolution of `Middleware`.
pub trait Chain: Send + Clone {
    /// `dispatch` will be called once per `Request`, and may be
    /// called either with or without an existing `Alloy`. `dispatch` is responsible
    /// for delegating the request to the correct `Middleware` and in the correct
    /// order. Effectively, 99% of the work done by a `chain` is done here.
    fn dispatch(&mut self,
                request: &mut Request,
                response: &mut Response,
                opt_alloy: Option<&mut Alloy>) -> Status {
        let mut alloy = &mut Alloy::new();
        match opt_alloy {
            Some(a) => alloy = a,
            None => ()
        };

        let status = self.chain_enter(request, response, alloy);
        let _ = self.chain_exit(request, response, alloy);

        status
    }

    #[doc(hidden)]
    fn chain_enter(&mut self,
             request: &mut Request,
             response: &mut Response,
             alloy: &mut Alloy) -> Status;

    #[doc(hidden)]
    fn chain_exit(&mut self,
                  _request: &mut Request,
                  _response: &mut Response,
                  _alloy: &mut Alloy) -> Status {
        Continue
    }

    /// `link` is responsible for adding new `Middleware` to the `chain's` internal
    /// storage of `Middleware`. Different `chains` may implement different behavior
    /// for `link`, but - ideally - `Middleware` added here will be delegated to during
    /// `Requests`.
    fn link<M: Middleware>(&mut self, _middleware: M);

    /// Create a new instance of `chain`.
    /// If you are making your own chain, you'll need to
    /// pass a new instance of it to `Iron`, otherwise,
    /// this function will only be used internally.
    fn new() -> Self;
}
