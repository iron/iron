use iron::Chain;
use shared::{ShareableMiddleware, Shared};

/// A mixin trait for providing the `link_shared` method to `Chain`, allowing
/// for convenient additions of `ShareableMiddleware` as `Middleware`.
pub trait SharedLink {
    /// Attach a `ShareableMiddleware` as a `Middleware`.
    fn link_shared<S: ShareableMiddleware + Send + Sync>(&mut self, S);
}

impl<C: Chain> SharedLink for C {
    fn link_shared<S: ShareableMiddleware + Send + Sync>(&mut self, s: S) {
        self.link(Shared::new(s))
    }
}

