// This requires running with:
//
// ```bash
// cargo run --example https --features ssl
// ```
//
// Generate a key and certificate like so:
//
// ```bash
// openssl genrsa -out localhost.key 4096
// openssl req -key localhost.key -x509 -new -days 3650 -out localhost.crt
// ```

extern crate iron;

#[cfg(feature = "ssl")]
fn main() {
    // Avoid unused errors due to conditional compilation ('ssl' feature is not default)
    use iron::status;
    use iron::{Iron, Request, Response};
    use std::path::{Path};
    use std::result::{Result};

    // openssl genrsa -out localhost.key 4096
    let key = Path::new("localhost.key").to_path_buf();
    // openssl req -key localhost.key -x509 -new -days 3650 -out localhost.crt
    let cert = Path::new("localhost.crt").to_path_buf();

    match Iron::new(|_: &mut Request| {
        Ok(Response::with((status::Ok, "Hello world!")))
    }).https("127.0.0.1:3000", cert, key) {
        Result::Ok(listening) => println!("{:?}", listening),
        Result::Err(err) => panic!("{:?}", err),
    }
    // curl -vvvv https://127.0.0.1:3000/ -k
}

#[cfg(not(feature = "ssl"))]
fn main() {
    // We need to do this to make sure `cargo test` passes.
}
