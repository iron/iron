Iron
====

[![Build Status](https://secure.travis-ci.org/iron/iron.svg?branch=master)](https://travis-ci.org/iron/iron)
[![Crates.io Status](http://meritbadge.herokuapp.com/iron)](https://crates.io/crates/iron)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/iron/iron/master/LICENSE)

> Extensible, Concurrency Focused Web Development in Rust.

## Response Timer Example

```rust
extern crate iron;
extern crate time;

use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use time::precise_time_ns;

struct ResponseTime;

impl typemap::Key for ResponseTime { type Value = u64; }

impl BeforeMiddleware for ResponseTime {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<ResponseTime>(precise_time_ns());
        Ok(())
    }
}

impl AfterMiddleware for ResponseTime {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        let delta = precise_time_ns() - *req.extensions.get::<ResponseTime>().unwrap();
        println!("Request took: {} ms", (delta as f64) / 1000000.0);
        Ok(res)
    }
}

fn hello_world(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((iron::status::Ok, "Hello World")))
}

fn main() {
    let mut chain = Chain::new(hello_world);
    chain.link_before(ResponseTime);
    chain.link_after(ResponseTime);
    Iron::new(chain).http("localhost:3000").unwrap();
}
```

## Overview

Iron is a high level web framework built in and for Rust, built on
[hyper](https://github.com/hyperium/hyper). Iron is designed to take advantage
of Rust's greatest features - its excellent type system and its principled
approach to ownership in both single threaded and multi threaded contexts.

Iron is highly concurrent and can scale horizontally on more machines behind a
load balancer or by running more threads on a more powerful machine. Iron
avoids the bottlenecks encountered in highly concurrent code by avoiding shared
writes and locking in the core framework.

Iron is 100% safe code:

```sh
$ rg unsafe src | wc
       0       0       0
```

## Philosophy

Iron is meant to be as extensible and pluggable as possible; Iron's core is
concentrated and avoids unnecessary features by leaving them to middleware,
plugins, and modifiers.

Middleware, Plugins, and Modifiers are the main ways to extend Iron with new
functionality. Most extensions that would be provided by middleware in other
web frameworks are instead addressed by the much simpler Modifier and Plugin
systems.

Modifiers allow external code to manipulate Requests and Response in an ergonomic
fashion, allowing third-party extensions to get the same treatment as modifiers
defined in Iron itself. Plugins allow for lazily-evaluated, automatically cached
extensions to Requests and Responses, perfect for parsing, accessing, and
otherwise lazily manipulating an http connection.

Middleware are only used when it is necessary to modify the control flow of a
Request flow, hijack the entire handling of a Request, check an incoming
Request, or to do final post-processing. This covers areas such as routing,
mounting, static asset serving, final template rendering, authentication, and
logging.

Iron comes with only basic modifiers for setting the status, body, and various
headers, and the infrastructure for creating modifiers, plugins, and
middleware. No plugins or middleware are bundled with Iron.

## Performance

Iron averages [72,000+ requests per second for hello world](https://github.com/iron/iron/wiki/How-to-Benchmark-hello.rs-Example)
and is mostly IO-bound, spending over 70% of its time in the kernel send-ing or
recv-ing data.\*

\* *Numbers from profiling on my OS X machine, your mileage may vary.*

## Core Extensions

Iron aims to fill a void in the Rust web stack - a high level framework that is
*extensible* and makes organizing complex server code easy.

Extensions are painless to build. Some important ones are:

Middleware:
- [Routing](https://github.com/iron/router)
- [Mounting](https://github.com/iron/mount)
- [Static File Serving](https://github.com/iron/staticfile)
- [Logging](https://github.com/iron/logger)

Plugins:
- [JSON Body Parsing](https://github.com/iron/body-parser)
- [URL Encoded Data Parsing](https://github.com/iron/urlencoded)
- [All-In-One (JSON, URL, & Form Data) Parameter Parsing](https://github.com/iron/params)

Both:
- [Shared Memory (also used for Plugin configuration)](https://github.com/iron/persistent)
- [Sessions](https://github.com/iron/iron-sessionstorage)

This allows for extremely flexible and powerful setups and allows nearly all
of Iron's features to be swappable - you can even change the middleware
resolution algorithm by swapping in your own `Chain`.

\* Due to the rapidly evolving state of the Rust ecosystem, not everything
builds all the time. Please be patient and file issues for breaking builds,
we're doing our best.

## Underlying HTTP Implementation

Iron is based on and uses [`hyper`](https://github.com/hyperium/hyper) as its
HTTP implementation, and lifts several types from it, including its header
representation, status, and other core HTTP types. It is usually unnecessary to
use `hyper` directly when using Iron, since Iron provides a facade over
`hyper`'s core facilities, but it is sometimes necessary to depend on it as
well.

<!--
FIXME: expand on when it is necessary to user hyper for serving,
e.g. when doing HTTPS.
-->

## Installation

If you're using `Cargo`, just add Iron to your `Cargo.toml`:

```toml
[dependencies.iron]
version = "*"
```

## [Documentation](http://ironframework.io/doc/iron)

The documentation is hosted [online](http://ironframework.io/doc/iron) and
auto-updated with each successful release. You can also use `cargo doc` to
build a local copy.

## [Examples](/examples)

Check out the [examples](/examples) directory!

You can run an individual example using `cargo run --example example-name`.
Note that for benchmarking you should make sure to use the `--release` flag,
which will cause cargo to compile the entire toolchain with optimizations.
Without `--release` you will get truly sad numbers.

## Getting Help

Feel free to ask questions as github issues in this or other related repos.

The best place to get immediate help is on IRC, on any of these channels on the
mozilla network:

- `#rust-webdev`
- `#iron`
- `#rust`

One of the maintainers or contributors is usually around and can probably help.
We encourage you to stop by and say hi and tell us what you're using Iron for,
even if you don't have any questions. It's invaluable to hear feedback from users
and always nice to hear if someone is using the framework we've worked on.

## Maintainers

Jonathan Reem ([reem](https://github.com/reem)) is the core maintainer and
author of Iron.

Commit Distribution (as of `8e55759`):

```
Jonathan Reem (415)
Zach Pomerantz (123)
Michael Sproul (9)
Patrick Tran (5)
Corey Richardson (4)
Bryce Fisher-Fleig (3)
Barosl Lee (2)
Christoph Burgdorf (2)
da4c30ff (2)
arathunku (1)
Cengiz Can (1)
Darayus (1)
Eduardo Bautista (1)
Mehdi Avdi (1)
Michael Sierks (1)
Nerijus Arlauskas (1)
SuprDewd (1)
```

## License

MIT

