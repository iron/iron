use iron::{Middleware, Request, Response, Alloy};
use iron::middleware::Status;
use std::sync::Arc;

pub trait ShareableMiddleware {
    fn shared_enter(&self, req: &mut Request, res: &mut Response, alloy: &mut Alloy) -> Status;
    fn shared_exit(&self, req: &mut Request, res: &mut Response, alloy: &mut Alloy) -> Status;
}

pub struct Shared {
    pub middleware: Arc<Box<ShareableMiddleware + Send + Share>>
}

impl Shared {
    pub fn new<S: ShareableMiddleware + Send + Share>(s: S) -> Shared {
        Shared {
            middleware: Arc::new(box s as Box<ShareableMiddleware + Send + Share>)
        }
    }
}

// Needed to hack name resolution in our Clone impl
fn clone<T: Clone>(t: &T) -> T { t.clone() }

impl Clone for Shared {
    fn clone(&self) -> Shared {
        Shared {
            middleware: clone(&self.middleware)
        }
    }
}

impl Middleware for Shared {
    fn enter(&mut self, req: &mut Request, res: &mut Response, alloy: &mut Alloy) -> Status {
        self.middleware.shared_enter(req, res, alloy)
    }

    fn exit(&mut self, req: &mut Request, res: &mut Response, alloy: &mut Alloy) -> Status {
        self.middleware.shared_exit(req, res, alloy)
    }
}

