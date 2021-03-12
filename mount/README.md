mount [![Build Status](https://secure.travis-ci.org/iron/mount.png?branch=master)](https://travis-ci.org/iron/mount) [![Crates.io Status](https://meritbadge.herokuapp.com/mount)](https://crates.io/crates/mount)
====

> Mounting middleware for the [Iron](https://github.com/iron/iron) web framework.

## Example

```rust
fn send_hello(req: &mut Request) -> IronResult<Response> {
    println!("Running send_hello handler, URL path: {:?}", req.url.path());
    Ok(Response::with((StatusCode::OK, "Hello!")))
}

fn intercept(req: &mut Request) -> IronResult<Response> {
    println!("Running intercept handler, URL path: {:?}", req.url.path());
    Ok(Response::with((StatusCode::OK, "Blocked!")))
}

fn main() {
    let mut mount = Mount::new();
    mount.mount("/blocked/", intercept).mount("/", send_hello);

    Iron::new(mount).http("localhost:3000");
}
```

Running the code above, the following HTTP requests would write the following line to the server process's stdout:

```
$ curl http://localhost:3000/
Running send_hello handler, URL path: [""]

$ curl http://localhost:3000/blocked/
Running intercept handler, URL path: [""]

$ curl http://localhost:3000/foo
Running send_hello handler, URL path: ["foo"]

$ curl http://localhost:3000/blocked/foo
Running intercept handler, URL path: ["foo"]
```

## Overview

mount is a part of Iron's [core bundle](https://github.com/iron/common).

- Mount a handler on a sub-path, hiding the old path from that handler.

## Installation

If you're using `Cargo` to manage dependencies, just add mount to the toml:

```toml
[dependencies.mount]

git = "https://github.com/iron/mount.git"
```

Otherwise, `cargo build`, and the rlib will be in your `target` directory.

## [Documentation](https://docs.rs/mount)

Along with the [online documentation](https://docs.rs/mount),
you can build a local copy with `cargo doc`.

## [Examples](/examples)

## Get Help

One of us ([@reem](https://github.com/reem/), [@zzmp](https://github.com/zzmp/),
[@theptrk](https://github.com/theptrk/), [@mcreinhard](https://github.com/mcreinhard))
is usually on `#iron` on the mozilla irc. Come say hi and ask any questions you might have.
We are also usually on `#rust` and `#rust-webdev`.
