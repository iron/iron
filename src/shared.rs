use iron::Middleware;
use std::sync::Arc;

pub struct Shared {
    pub middleware: Arc<Box<Middleware + Send + Share>>
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

