extern crate iron;
extern crate http;
extern crate mount;

use std::io::net::ip::Ipv4Addr;

use http::status;
use iron::{Iron, Request, Response, Server, Chain, Status, Unwind, FromFn, Url};
use mount::{Mount, OriginalUrl};

fn level_two(req: &mut Request, res: &mut Response) -> Status {
    let _ = res.serve(status::Ok, "Welcome to Level 2.");
    match req.extensions.find::<OriginalUrl, Url>() {
        Some(url) => println!("Original URL: {}", url),
        None => println!("Error: No original URL found.")
    }
    Unwind
}

fn main() {
    let mut server: Server = Iron::new();
    let second_mount = Mount::new("/leveltwo/", FromFn::new(level_two));
    let first_mount = Mount::new("/levelone/", second_mount);
    server.chain.link(first_mount);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}

