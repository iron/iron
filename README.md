Iron [![Build Status](https://secure.travis-ci.org/iron/iron.png?branch=master)](https://travis-ci.org/iron/iron)
====

> Express inspired, rapid, scalable, concurrent and safe server development

Iron is a high level web framework built in and for Rust.
Iron does not come bundled with any middleware - instead,
Iron is a robust and efficient framework for plugging in middleware.

After spawning, handling a single request through Iron’s middleware stack
with a single no-op middleware takes only _47 nanoseconds_.

## Installation

```bash
./configure   # Gets all dependencies and builds them
make lib      # Build Iron itself -- you can stop here if you just want the library
make test     # Build and run tests
make examples # Build the examples
make doc      # Build documentation using rustdoc
```

## Get Help

One of us (@reem, @zzmp, @theptrk, @mcreinhard) is usually on `#iron` on the
mozilla irc. Come say hi and ask any questions you might have. We are also
usually on `#rust` and `#rust-webdev`.

## Overview

Iron aims to fill a void in the Rust web stack - a high level framework that is
*extensible* and makes organizing complex server code easy.

Whereas other web frameworks have focused mostly on creating an easy-to-use
routing system, Iron focuses on providing a clean API for creating
middleware and integrating them in Iron servers.

In fact, Routing is middleware in Iron, as are Mounting, Body Parsing, and most
other features. This allows for insanely flexible setups and allows almost all
of Iron’s features to be swappable - you can even change the middleware
resolution algorithm by swapping in your own Chain.

## Examples

Here’s a setup for an api with two different versions:

```rust
extern crate iron;
extern crate router;
extern crate mount;
extern crate logger;

use std::io::net::ip::Ipv4Addr;

use iron::{Chain, Request, Response, Alloy, ServerT};
use router::{Router, Params};
use logger:Logger;
use hypothetical::database;

fn setup_api_v1(router: &mut Router) {
    fn get_user(_req: &mut Request, res: &mut Response, alloy: &mut Alloy) {
        let params = alloy.find::<Params>().unwrap();
        res.write(database::get("Users", params.get("userid").unwrap()).as_bytes());
    }
    router.get("/users/:userid", get_user);
}
fn setup_api_v2(router: &mut Router) { ... }

fn main() {
    let api_v1_router = Router::new();
    setup_api_v1(&mut api_v1_router);

    let api_v2_router = Router::new();
    setup_api_v2(&mut api_v2_router);

    let mut server: ServerT = Iron::new();

    // Setup Logging middleware
    server.link(Logger::new());

    // Mount sub-middleware.
    server.chain.link(Mount::new("/api/v1", api_v1_router));
    server.chain.link(Mount::new("/api/v2", api_v2_router));

    // Since Box<Chain>'s are middleware, we can use them in *any* place a
    // middleware is expected.
    server.chain.link(
        Mount::new(
            "/secret", box vec![authorize, secrets].iter().collect::<StackChain>()
        )
    );

    server.listen(Ipv4addr(127, 0, 0, 1), 3000);
}

```

Here’s a sample middleware implementation of a RequestTimer middleware:

```rust
extern crate iron;
extern crate time;

use std::io::net::ip::Ipv4Addr;
use iron::{Chain, Request, Response, Middleware, Alloy, ServerT};
use iron::middleware::{Status, Continue};

use time::precise_time_ns;

#[deriving(Clone)]
struct ResponseTime {
    entry: u64
};

impl ResponseTime { fn new() -> ResponseTime { ResponseTime(0u64) } }

impl Middleware for ResponseTime {
    fn enter(&mut self, _req: &mut Request, _res: &mut Response, _al: &mut Alloy) -> Status {
        self.entry = precise_time_ns();
        Continue
    }

    fn exit(&mut self, _req: &mut Request, _res: &mut Respose, _al: &mut Alloy) -> Status {
        let delta = precise_time_ns() - self.enty;
        println!("Request took: {} ms", (delta as f64) / 100000.0);
        Continue
    }
}

fn main() {
    let mut server: ServerT = Iron::new();

    // This adds the ResponseTime middleware so that
    // all requests and responses are passed through it.
    server.chain.link(ResponseTime::new());

    // Start the server on localhost:3000
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}
```

