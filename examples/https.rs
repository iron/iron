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

extern crate futures;
extern crate iron;
#[cfg(feature = "ssl")]
extern crate native_tls;
#[cfg(feature = "ssl")]
extern crate tokio_tls;

#[cfg(feature = "ssl")]
fn main() {
    // Avoid unused errors due to conditional compilation ('native-tls-example' feature is not default)
    use native_tls::{Pkcs12, TlsAcceptor};
    use iron::{Iron, Request, Response, BoxIronFuture};
    use iron::status;
    use std::io::prelude::*;
    use std::result::Result;
    use std::fs::File;
    use futures::future;

    let mut file = File::open("identity.p12").unwrap();
    let mut pkcs12 = vec![];
    file.read_to_end(&mut pkcs12).unwrap();
    let pkcs12 = Pkcs12::from_der(&pkcs12, "mypass").unwrap();

    let ssl = TlsAcceptor::builder(pkcs12).unwrap().build().unwrap();

    match Iron::new(|req: Request| {
       Box::new(future::ok((req, Response::with((status::Ok, "Hello world!"))))) as BoxIronFuture<(Request, Response)>
    }).https("127.0.0.1:3000", ssl) {
        Result::Ok(listening) => println!("{:?}", listening),
        Result::Err(err) => panic!("{:?}", err),
    }
    // curl -vvvv https://127.0.0.1:3000/ -k
}

#[cfg(not(feature = "ssl"))]
fn main() {
    // We need to do this to make sure `cargo test` passes.
}
