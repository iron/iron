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
// openssl pkcs12 -export -out identity.p12 -inkey localhost.key -in localhost.crt -password pass:mypass
//
// ```

extern crate iron;
#[cfg(feature = "ssl")]
extern crate native_tls;

#[cfg(feature = "ssl")]
fn main() {
    // Avoid unused errors due to conditional compilation ('native-tls-example' feature is not default)
    use native_tls::{Pkcs12, TlsAcceptor};
    use iron::{Iron, Request, Response};
    use iron::status;
    use std::io::prelude::*;
    use std::fs::File;

    let mut file = File::open("identity.p12").unwrap();
    let mut pkcs12 = vec![];
    file.read_to_end(&mut pkcs12).unwrap();
    let pkcs12 = Pkcs12::from_der(&pkcs12, "mypass").unwrap();

    let ssl = TlsAcceptor::builder(pkcs12).unwrap().build().unwrap();

    Iron::new(|_: &mut Request| {
        Ok(Response::with((status::Ok, "Hello world!")))
    }).https("127.0.0.1:3000", ssl);
    // curl -vvvv https://127.0.0.1:3000/ -k
}

#[cfg(not(feature = "ssl"))]
fn main() {
    // We need to do this to make sure `cargo test` passes.
}
