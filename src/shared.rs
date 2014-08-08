use iron::{Middleware, Request, Response, Status};
use std::sync::Arc;

/// `Middleware` implementing this trait can be linked using `Shared` so that
/// they are not cloned for each request. This can vastly improve performance
/// for immutable middleware that are not modified while handling requests.
pub trait ShareableMiddleware {
    /// `shared_enter` is an immutable version of `enter` from the `Middleware`
    /// trait - it receives an & reference to self rather than an &mut
    /// reference.
    fn shared_enter(&self, req: &mut Request, res: &mut Response) -> Status;

    /// `shared_exit` is an immutable version of `exit` from the `Middleware`
    /// trait - it receives an & reference to self rather than an &mut
    /// reference.
    fn shared_exit(&self, req: &mut Request, res: &mut Response) -> Status;
}

/// `Shared` is used to wrap `ShareableMiddleware` into `Middleware` so they
/// can be linked onto a `Chain` while still avoiding unnecessary copies.
pub struct Shared {
    /// The wrapped `ShareableMiddleware`
    pub middleware: Arc<Box<ShareableMiddleware + Send + Share>>
}

impl Shared {
    /// Creates a new instance of `Shared` containing the provided
    /// `ShareableMiddleware` and allowing it to be used as `Middleware`.
    pub fn new<S: ShareableMiddleware + Send + Share>(s: S) -> Shared {
        Shared {
            middleware: Arc::new(box s as Box<ShareableMiddleware + Send + Share>)
        }
    }
}

impl Clone for Shared {
    fn clone(&self) -> Shared {
        Shared {
            middleware: self.middleware.clone()
        }
    }
}

impl Middleware for Shared {
    fn enter(&mut self, req: &mut Request, res: &mut Response) -> Status {
        self.middleware.shared_enter(req, res)
    }

    fn exit(&mut self, req: &mut Request, res: &mut Response) -> Status {
        self.middleware.shared_exit(req, res)
    }
}

