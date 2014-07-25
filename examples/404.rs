
extern crate http;
extern crate iron;

use std::io::net::ip::Ipv4Addr;

use iron::{Iron, Server};

fn main() {
    let server: Server = Iron::new();
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}

