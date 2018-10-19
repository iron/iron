Router [![Build Status](https://secure.travis-ci.org/iron/router.png?branch=master)](https://travis-ci.org/iron/router) [![Crates.io Status](https://meritbadge.herokuapp.com/router)](https://crates.io/crates/router)
====

> Routing handler for the [Iron](https://github.com/iron/iron) web framework.

Router is a fast, convenient, and flexible routing middleware for Iron. It
allows complex glob patterns and named url parameters and also allows handlers
to be any Handler, including all Chains.

## Example

```rust
extern crate iron;
extern crate router;

use iron::prelude::*;
use iron::status;
use router::Router;

fn main() {
    let mut router = Router::new();           // Alternative syntax:
    router.get("/", handler, "index");        // let router = router!(index: get "/" => handler,
    router.get("/:query", handler, "query");  //                      query: get "/:query" => handler);

    Iron::new(router).http("localhost:3000").unwrap();

    fn handler(req: &mut Request) -> IronResult<Response> {
        let ref query = req.extensions.get::<Router>().unwrap().find("query").unwrap_or("/");
        Ok(Response::with((status::Ok, *query)))
    }
}
```

## Overview

Router is a part of Iron's [core bundle](https://github.com/iron/core).

- Route client requests based on their paths
- Parse parameters and provide them to other middleware/handlers

## Installation

If you're using cargo, just add router to your `Cargo.toml`.

```toml
[dependencies]

router = "*"
```

Otherwise, `cargo build`, and the rlib will be in your `target` directory.

## [Documentation](http://ironframework.io/doc/router)

Along with the [online documentation](http://ironframework.io/doc/router),
you can build a local copy with `make doc`.

## [Examples](/examples)

## Get Help

One of us is usually on `#iron` on the mozilla irc.
Come say hi and ask any questions you might have.
We are also usually on `#rust` and `#rust-webdev`.
