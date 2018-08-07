extern crate futures_cpupool;
extern crate hyper;
extern crate iron;

use std::time::Duration;

use futures_cpupool::CpuPool;

use iron::prelude::*;
use iron::StatusCode;
use iron::Timeouts;

fn main() {
    let mut iron =
        Iron::new(|_: &mut Request| Ok(Response::with((StatusCode::OK, "Hello world!"))));
    iron.pool = CpuPool::new(8);
    iron.timeouts = Timeouts {
        keep_alive: Some(Duration::from_secs(10)),
    };

    let addr = "127.0.0.1:3000".parse().unwrap();
    iron.local_address = Some(addr);

    iron.http("127.0.0.1:3000");
}
