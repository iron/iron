extern crate iron;
extern crate logger;

use std::io::net::ip::Ipv4Addr;

use iron::{Iron, ServerT};

use logger::Logger;

fn main() {
    let format_str =
        "@[red]URI: {uri}@@, @[blue]Method: {method}@@, @[yellow]Status: {status}@@, @[green]Time: {response_time}@@";
    let logger = Logger::new(from_str(format_str));
    let mut server: ServerT = Iron::new();
    server.smelt(logger);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}
