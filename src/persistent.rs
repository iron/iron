use iron::{Request, Response, Middleware, Alloy};
use iron::middleware::{Status, Continue};
use std::sync::{Arc, RWLock};

pub struct Persistent<Data, Phantom> {
    pub data: Arc<RWLock<Data>>
}

impl<D: Send + Share, P> Clone for Persistent<D, P> {
    fn clone(&self) -> Persistent<D, P> {
        Persistent {
            data: self.data.clone()
        }
    }
}

impl<D: Send + Share, P> Middleware for Persistent<D, P> {
    fn enter(&mut self, _: &mut Request,
             _: &mut Response, alloy: &mut Alloy) -> Status {
        alloy.insert::<Persistent<D, P>>(self.clone());
        Continue
    }
}

impl<D: Send + Share, P> Persistent<D, P> {
    pub fn new(data: D) -> Persistent<D, P> {
        Persistent { data: Arc::new(RWLock::new(data)) }
    }
}

impl<D: Send + Share, P> Persistent<D, P> {
    pub fn new(data: D) -> Persistent<D, P> {
        return Persistent { data: Arc::new(RWLock::new(data)) };
    }
}

