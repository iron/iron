use std::sync::{Arc, Mutex};

pub struct Persistent<Data, Phantom> {
    data: Arc<Mutex<Data>>
}

impl<D: Send, P> Clone for Persistent<D, P> {
    fn clone(&self) -> Persistent<D, P> {
        Persistent {
            data: self.data.clone()
        }
    }
}

