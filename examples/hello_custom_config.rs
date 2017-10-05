extern crate futures_cpupool;
extern crate hyper;
extern crate iron;
extern crate tokio_proto;

use std::time::Duration;

use futures_cpupool::CpuPool;

use hyper::server::Http;

use iron::prelude::*;
use iron::status;
use iron::Timeouts;

fn main() {
    let mut iron = Iron::new(|_: &mut Request| {
        Ok(Response::with((status::Ok, "Hello world!")))
    });
    iron.pool = CpuPool::new(8);
    iron.timeouts = Timeouts {
        keep_alive: Some(Duration::from_secs(10)),
        read: Some(Duration::from_secs(10)),
        write: Some(Duration::from_secs(10))
    };
    let addr = "127.0.0.1:3000".parse().unwrap();
    iron.local_address = Some(addr);

    let mut hyper = Http::new();
    hyper.keep_alive(false);

    let tcp = tokio_proto::TcpServer::new(hyper, addr);

    tcp.serve(iron);
}

