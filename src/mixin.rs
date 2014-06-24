use iron::Chain;
use shared::{ShareableMiddleware, Shared};

pub trait SharedLink {
    fn link_shared<S: ShareableMiddleware + Send + Share>(&mut self, S);
}

impl<C: Chain> SharedLink for C {
    fn link_shared<S: ShareableMiddleware + Send + Share>(&mut self, s: S) {
        self.link(Shared::new(s))
    }
}

