// This requires running with:
//
// ```bash
// cargo run --example https --features native-tls-example
// ```
//
// Generate an identity like so:
//
// ```bash
// openssl req -x509 -newkey rsa:4096 -nodes -keyout localhost.key -out localhost.crt -days 3650
// openssl pkcs12 -export -out identity.p12 -inkey localhost.key -in localhost.crt --password mypass
//
// ```

extern crate iron;
#[cfg(feature = "native-tls-example")]
extern crate hyper_native_tls;

#[cfg(feature = "native-tls-example")]
fn main() {
    // Avoid unused errors due to conditional compilation ('native-tls-example' feature is not default)
    use hyper_native_tls::NativeTlsServer;
    use iron::{Iron, Request, Response};
    use iron::status;
    use std::result::Result;

    let ssl = NativeTlsServer::new("identity.p12", "mypass").unwrap();

    match Iron::new(|_: &mut Request| {
        Ok(Response::with((status::Ok, "Hello world!")))
    }).https("127.0.0.1:3000", ssl) {
        Result::Ok(listening) => println!("{:?}", listening),
        Result::Err(err) => panic!("{:?}", err),
    }
    // curl -vvvv https://127.0.0.1:3000/ -k
}

#[cfg(not(feature = "native-tls-example"))]
fn main() {
    // We need to do this to make sure `cargo test` passes.
}
