Iron [![Build Status](https://secure.travis-ci.org/iron/iron.png?branch=master)](https://travis-ci.org/iron/iron)
====

> Middleware-Oriented, Concurrency Focused Web Development in Rust.

## Response Timer Example

```rust
struct ResponseTime;

impl Assoc<u64> for ResponseTime {}

impl BeforeMiddleware for ResponseTime {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<ResponseTime, u64>(precise_time_ns());
        Ok(())
    }
}

impl AfterMiddleware for ResponseTime {
    fn after(&self, req: &mut Request, _: &mut Response) -> IronResult<()> {
        let delta = precise_time_ns() - *req.extensions.find::<ResponseTime, u64>().unwrap();
        println!("Request took: {} ms", (delta as f64) / 1000000.0);
        Ok(())
    }
}
```

Iron is a high level web framework built in and for Rust

Iron does not come bundled with any middleware - instead, Iron is a robust and efficient framework for plugging in middleware.

**Iron focuses on providing a clean API for creating middleware and integrating
them in Iron servers.**

Iron averages [52,000+ requests per second for hello world](https://github.com/iron/iron/wiki/How-to-Benchmark-hello.rs-Example).

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

## [Documentation](http://ironframework.io/doc/iron)

Along with the [online documentation](http://ironframework.io/doc/iron),
you can build a local copy with `cargo doc`.

## [More Examples](/examples)

Check out the [examples](/examples) directory!

You can compile all of the examples with `cargo test`. The binaries will be placed in `target/test/`.

## Get Help

One of us ([@reem](https://github.com/reem/), [@zzmp](https://github.com/zzmp/),
[@theptrk](https://github.com/theptrk/), [@mcreinhard](https://github.com/mcreinhard))
is usually on `#iron` on the mozilla irc. Come say hi and ask any questions you might have.
We are also usually on `#rust` and `#rust-webdev`.

