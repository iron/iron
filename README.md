Iron [![Build Status](https://secure.travis-ci.org/iron/iron.png?branch=master)](https://travis-ci.org/iron/iron)
====

> Express inspired, rapid, scalable, concurrent and safe server development

Iron is a high level web framework built in and for Rust. Iron does not come
bundled with any middleware, which Iron calls Ingots - instead, Iron is a
robust and efficient framework for plugging in middleware.

After spawning, handling a single request through Iron’s middleware stack
with a single no-op middleware takes only _300 nanoseconds_.

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
usually on `#rust`.

## Overview

Iron aims to fill a void in the Rust web stack - a high level framework that is
*extensible* and makes organizing complex server code easy.

Whereas other web frameworks have focused mostly on creating an easy-to-use
routing system, Iron focuses on providing a clean API for creating
Ingots/middleware and integrating them in Iron servers.

In fact, Routing is middleware in Iron, as are Mounting, Body Parsing, and most
other features. This allows for insanely flexible setups and allows almost all
of Iron’s features to be swappable - you can even change the middleware
resolution algorithm by swapping in your own Furnace.

## Examples

Here’s an as of yet hypothetical\* setup for an api with two different
versions:

```rust
extern crate iron;
extern crate router = “iron-router”;
extern crate mount = “iron-mount”;
extern crate bodyparser = “iron-body-parser”;
extern crate json = “iron-json”;

use json::JSON;
use iron::{Request, Response, Alloy, ServerT};

use router::Router;
use mount::Mount;
use bodyparser::{BodyParser, Parsed};
use hypothetical::database;

fn setup_api_v1<Rq: Request, Rs: Response>(&mut Router<Rq, Rs>) {
    Router.get(match!(‘/users/’), |_req: &mut Rq, res: &mut Rs, alloy: &mut Alloy| {
        // Will be `depends_on!(JSON -> send)` in the future
        let send = alloy.find::<JSON>().unwrap().send;

        send(res, 200, database::read(‘Users’));
    });
}
fn setup_api_v2(&mut Router) { ... }

fn main() {
    let api_v1_router = setup_api_v1(Router::new());
    let api_v2_router = setup_api_v2(Router::new());

    let mut server: ServerT = Iron::new();

    // Setup JSON middleware
    server.smelt(JSON::new());

    // Mount sub-instances of Iron.
    // mount! is a macro from Mount that creates a sub-instance of Iron
    // with the second argument smelted on to it.
    server.smelt(mount!(match!(‘/api/v1’), api_v1_router));
    server.smelt(mount!(match!(‘/api/v2’), api_v2_router));
}

```

\* Most of these middleware are in development and not finished yet.

Here’s a sample Ingot/middleware implementation of a RequestTimer Ingot:

```rust
extern crate iron;
extern crate time;

use iron::{Request, Response, Ingot, Alloy};

use time::precise_time_ns;

#[deriving(Clone)]
struct ResponseTime(u64);

impl ResponseTime { fn new() -> ResponseTime { ResponseTime(0u64) } }

impl<Rq: Request, Rs: Response> Ingot<Rq, Rs> for ResponseTime {
    fn enter(&mut self, _req: &mut Rq, _res: &mut Rs, _al: &mut Alloy) -> ingot::Status {
        self.entry = precise_time_ns();
        Continue
    }

    fn exit(&mut self, _req: &mut Rq, _res: &mut Rs, _al: &mut Alloy) -> ingot::Status {
        let delta = precise_time_ns() - self.enty;
        println!(“Request took: {} ms”, (delta as f64) / 100000.0);
        Continue
    }
}

fn main() {
    // This type is long, but there will be a type shortcut fot this default
    // in the future.
    let mut server: Iron<IronRequest, IronResponse<’static>,
                    IronFurnace<IronRequest, IronResponse<’static>>> = Iron::new();

    // This adds the ResponseTime ingot to have all requests and responses be
    // passed through it.
    server.smelt(ResponseTime::new());

    // Start the server on localhost:3000
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}
```

