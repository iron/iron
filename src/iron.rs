//! Exposes the `Iron` type, the main entrance point of the
//! `Iron` library.

use taskpool::TaskPool;

use std::io::net::ip::IpAddr;
use std::io::net::tcp;
use std::io::{Listener, Acceptor};
use std::sync::Arc;

use {Request, Handler};
use status;

/// The primary entrance point to `Iron`, a `struct` to instantiate a new server.
///
/// `Iron` contains the `Handler` which takes a `Request` and produces a
/// `Response`.
pub struct Iron<H> {
    /// Iron contains a `Handler`, which it uses to create responses for client
    /// requests.
    pub handler: H,
}

// The struct which actually listens and serves requests.
struct IronListener<H> {
    handler: Arc<H>,
    ip: IpAddr,
    port: u16
}

impl<H: Send + Sync> Clone for IronListener<H> {
    fn clone(&self) -> IronListener<H> {
        IronListener {
            // Just increment the Arc's reference count.
            handler: self.handler.clone(),
            ip: self.ip.clone(),
            port: self.port.clone()
        }
    }
}

impl<H: Handler> Iron<H> {
    /// Kick off the server process.
    ///
    /// Call this once to begin listening for requests on the server.
    /// This consumes the Iron instance, but does the listening on
    /// another task, so is not blocking.
    ///
    /// Defaults to a threadpool of size 100.
    pub fn listen(self, ip: IpAddr, port: u16) {
        spawn(proc() {
            IronListener {
                handler: Arc::new(self.handler),
                ip: ip,
                port: port
            }.serve(100)
        });
    }

    /// Kick off the server process with X threads.
    pub fn listen_with(self, ip: IpAddr, port: u16, threads: uint) {
        spawn(proc() {
            IronListener {
                handler: Arc::new(self.handler),
                ip: ip,
                port: port
            }.serve(threads)
        });
    }

    /// Instantiate a new instance of `Iron`.
    ///
    /// This will create a new `Iron`, the base unit of the server, using the
    /// passed in `Handler`.
    pub fn new(handler: H) -> Iron<H> {
        Iron { handler: handler }
    }
}

impl<H: Handler> IronListener<H> {
    fn serve(self, threads: uint) {
        let mut acceptor = match tcp::TcpListener::bind((self.ip, self.port)).listen() {
            Err(err) => {
                error!("Bind or Listen failed: {}", err);
                return;
            },
            Ok(acceptor) => acceptor
        };

        let mut taskpool = TaskPool::new(threads);

        for stream in acceptor.incoming() {
            let stream = match stream {
                Err(_) => {
                    continue;
                },
                Ok(socket) => socket
            };

            let handler = self.handler.clone();

            taskpool.execute(proc() {
                let mut stream = ::http::buffer::BufferedStream::new(stream);
                let mut close = false;

                while !close {
                    let (http_req, err) = ::http::server::request::Request::load(&mut stream);
                    close = http_req.close_connection;
                    let mut http_res = ::http::server::response::ResponseWriter::new(&mut stream);

                    match err {
                        Ok(()) => {
                            // Create `Request` wrapper.
                            let mut req = match Request::from_http(http_req) {
                                Ok(req) => req,
                                Err(e) => {
                                    error!("Error getting request: {}", e);
                                    http_res.status = status::InternalServerError;
                                    let _ = http_res.write(b"Internal Server Error");
                                    return;
                                }
                            };

                            // Dispatch the request
                            let res = handler.call(&mut req).map_err(|e| {
                                handler.catch(&mut req, e)
                            });

                            match res {
                                // Write the response back to http_res
                                Ok(res) => res.write_back(&mut http_res),
                                Err(e) => {
                                    // There is no Response, so create one.
                                    error!("Error handling:\n{}\nError was: {}", req, e);
                                    http_res.status = status::InternalServerError;
                                    let _ = http_res.write(b"Internal Server Error");
                                }
                            }
                        },

                        Err(err) => {
                            http_res.status = err;
                            http_res.headers.content_length = Some(0);
                            match http_res.write_headers() {
                                Err(_) => return,
                                _ => {}
                            };
                        }
                    };

                    match http_res.finish_response() {
                        Err(_) => return,
                        _ => {}
                    };
                }
            });
        }
    }
}

