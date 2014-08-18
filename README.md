Iron [![Build Status](https://secure.travis-ci.org/iron/iron.png?branch=master)](https://travis-ci.org/iron/iron)
====

> Express-inspired, rapid, scalable, concurrent and safe server development.

## Simple ResponseTimer Middleware

```rust
#[deriving(Clone)]
struct ResponseTime {
    entry_time: u64
}

impl ResponseTime { fn new() -> ResponseTime { ResponseTime { entry_time: 0u64 } } }

impl Middleware for ResponseTime {
    fn enter(&mut self, _req: &mut Request, _res: &mut Response) -> Status {
        self.entry_time = precise_time_ns();
        Continue
    }

    fn exit(&mut self, _req: &mut Request, _res: &mut Response) -> Status {
        let delta = precise_time_ns() - self.entry_time;
        println!("Request took: {} ms", (delta as f64) / 1000000.0);
        Continue
    }
}

// ...
server.chain.link(ResponseTime::new());
// ...
```

Iron is a high level web framework built in and for Rust.</br>
Iron does not come bundled with any middleware - instead, Iron is a robust and efficient framework for plugging in middleware.

**Iron focuses on providing a clean API for creating middleware and integrating
them in Iron servers.**

After spawning, handling a single request through Iron’s middleware stack
with a single no-op middleware takes only 0.9 _micro_ seconds - with ten middleware,
it's only 1.1 microseconds.

Iron averages [17,000+ requests per second for hello world](https://github.com/iron/iron/wiki/How-to-Benchmark-hello.rs-Example).

## [Overview](http://ironframework.io)

Iron aims to fill a void in the Rust web stack - a high level framework that is
*extensible* and makes organizing complex server code easy.

Middleware is painless to build, and the [core bundle](https://github.com/iron/core)
already includes:
- [Routing](https://github.com/iron/router)
- [Mounting](https://github.com/iron/mount)
- [Static File Serving](https://github.com/iron/static-file)
- [JSON Body Parsing](https://github.com/iron/body-parser)
- [URL Encoded Data Parsing](https://github.com/iron/urlencoded)
- [Logging](https://github.com/iron/logger)
- [Cookies](https://github.com/iron/cookie)
- [Sessions](https://github.com/iron/session)
- [Persistent Storage](https://github.com/iron/persistent)

This allows for insanely flexible and powerful setups and allows nearly all
of Iron’s features to be swappable - you can even change the middleware
resolution algorithm by swapping in your own `Chain`.

## Installation

If you're using `Cargo`, just add Iron to your `Cargo.toml`:

```toml
[dependencies.iron]

git = "https://github.com/iron/iron.git"
```

Otherwise, just clone this repo, `cargo build`, and the rlib will be in your `target` directory.

## [Documentation](http://docs.ironframework.io/)

Along with the [online documentation](http://docs.ironframework.io/),
you can build a local copy with `cargo doc`.

### Building Middleware

Implement the `Middleware` trait to create your own, or pass a function with the following signature to `FromFn::new`:

```rust
fn handler(req: &mut Request, res: &mut Response) -> Status;
```

## [More Examples](/examples)

Check out the [examples](/examples) directory!

You can compile all of the examples with `cargo test`. The binaries will be placed in `target/test/`.

## Get Help

One of us ([@reem](https://github.com/reem/), [@zzmp](https://github.com/zzmp/),
[@theptrk](https://github.com/theptrk/), [@mcreinhard](https://github.com/mcreinhard))
is usually on `#iron` on the mozilla irc. Come say hi and ask any questions you might have.
We are also usually on `#rust` and `#rust-webdev`.

