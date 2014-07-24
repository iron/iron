
extern crate http;
extern crate iron;

use std::io::net::ip::Ipv4Addr;

use iron::{Iron, Server, Status};

fn main() {
    let mut server: Server = Iron::new();
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}

