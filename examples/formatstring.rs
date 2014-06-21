extern crate iron;
extern crate logger;

use std::io::net::ip::Ipv4Addr;

use iron::{Iron, ServerT};

use logger::Logger;
use logger::format::Format;

fn main() {
    let format_str =
        "@[red]URI: {uri}@@, @[blue]Method: {method}@@, @[yellow]Status: {status}@@, @[green]Time: {response_time}@@";
    let logger = Logger::new(Format::from_format_string(format_str, &mut vec![]));
    let mut server: ServerT = Iron::new();
    server.smelt(logger);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}
