Iron [![Build Status](https://secure.travis-ci.org/iron/iron.png?branch=master)](https://travis-ci.org/iron/iron)
====

> Express-inspired, rapid, scalable, concurrent and safe server development

## Simple ResponseTimer Middleware

```rust
#[deriving(Clone)]
pub struct ResponseTime { entry: u64 }

impl ResponseTime { fn new() -> ResponseTime { ResponseTime { entry: 0u64 } } }

// This Trait defines middleware.
impl MiddleWare for ResponseTime {
    fn enter(&mut self, _: &mut Request, _: &mut Response, _: &mut Alloy) -> Status {
        self.entry = precise_time_ns();
        Continue // Continue to other middleware in the stack
    }

    fn exit(&mut self, _: &mut Request, _: &mut Response, _: &mut Alloy) -> Status {
        let delta = precise_time_ns() - self.entry;
        println!("Request took {} ms.", (delta as f64) / 100000.0)
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

## [Overview](http://ironframework.io)

Iron aims to fill a void in the Rust web stack - a high level framework that is
*extensible* and makes organizing complex server code easy.

Middleware is painless to build, and the [core bundle](https://github.com/iron/core)
already includes:
- [Routing](https://github.com/iron/router)
- [Mounting](https://github.com/iron/mount)
- [Static file-serving](https://github.com/iron/static-file)
- [Body parsing](https://github.com/iron/body-parser)
- [Url encoding](https://github.com/iron/urlencoding)
- [Logging](https://github.com/iron/logger)
- [Cookies](https://github.com/iron/cookie)
- [Sessions](https://github.com/iron/session)
- [Persistent storage](https://github.com/iron/persistent)

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
you can build a local copy with `make doc`.

### Building Middleware

`impl Middleware` to create your own, or pass a function with the correct signature to `FromFn::new`.

## [More Examples](/examples)

[See more examples.](/examples)

## Get Help

One of us ([@reem](https://github.com/reem/), [@zzmp](https://github.com/zzmp/),
[@theptrk](https://github.com/theptrk/), [@mcreinhard](https://github.com/mcreinhard))
is usually on `#iron` on the mozilla irc. Come say hi and ask any questions you might have.
We are also usually on `#rust` and `#rust-webdev`.
