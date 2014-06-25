//! Example of a simple logger
extern crate iron;
extern crate logger;

use std::io::net::ip::Ipv4Addr;

use iron::{Iron, ServerT, Chain};

use logger::Logger;

// Logger has a default formatting of the strings printed 
// to console. 
fn main() {
    let logger = Logger::new(None);
    let mut server: ServerT = Iron::new();
    server.chain.link(logger);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}
