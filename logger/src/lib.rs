#![deny(missing_docs, warnings)]

//! Request logging middleware for Iron

extern crate iron;
#[macro_use] extern crate log;
extern crate time;

use iron::{AfterMiddleware, BeforeMiddleware, IronResult, IronError, Request, Response};
use iron::typemap::Key;

use format::FormatText::{Str, Method, URI, Status, ResponseTime, RemoteAddr, RequestTime};
use format::{ContextDisplay, FormatText};

use std::fmt::{Display, Formatter};

mod format;
pub use format::Format;

/// `Middleware` for logging request and response info to the terminal.
pub struct Logger {
    format: Format,
}

impl Logger {
    /// Create a pair of `Logger` middlewares with the specified `format`. If a `None` is passed in, uses the default format:
    ///
    /// ```ignore
    /// {method} {uri} -> {status} ({response-time} ms)
    /// ```
    ///
    /// While the returned value can be passed straight to `Chain::link`, consider making the logger `BeforeMiddleware`
    /// the first in your chain and the logger `AfterMiddleware` the last by doing something like this:
    ///
    /// ```ignore
    /// let mut chain = Chain::new(handler);
    /// let (logger_before, logger_after) = Logger::new(None);
    /// chain.link_before(logger_before);
    /// // link other middlewares here...
    /// chain.link_after(logger_after);
    /// ```
    pub fn new(format: Option<Format>) -> (Logger, Logger) {
        let format = format.unwrap_or_default();
        (Logger { format: format.clone() }, Logger { format: format })
    }
}

struct StartTime;
impl Key for StartTime { type Value = time::Tm; }

impl Logger {
    fn initialise(&self, req: &mut Request) {
        req.extensions.insert::<StartTime>(time::now());
    }

    fn log(&self, req: &mut Request, res: &Response) -> IronResult<()> {
        let entry_time = *req.extensions.get::<StartTime>().unwrap();

        let response_time = time::now() - entry_time;
        let response_time_ms = (response_time.num_nanoseconds().unwrap_or(0) as f64) / 1000000.0;

        {
            let render = |fmt: &mut Formatter, text: &FormatText| {
                match *text {
                    Str(ref string) => fmt.write_str(string),
                    Method => req.method.fmt(fmt),
                    URI => req.url.fmt(fmt),
                    Status => {
                        match res.status {
                            Some(status) => status.fmt(fmt),
                            None => fmt.write_str("<missing status code>"),
                        }
                    }
                    ResponseTime => fmt.write_fmt(format_args!("{} ms", response_time_ms)),
                    RemoteAddr => req.remote_addr.fmt(fmt),
                    RequestTime => {
                        entry_time.strftime("%Y-%m-%dT%H:%M:%S.%fZ%z")
                            .unwrap()
                            .fmt(fmt)
                    }
                }
            };

            info!("{}", self.format.display_with(&render));
        }

        Ok(())
    }
}

impl BeforeMiddleware for Logger {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        self.initialise(req);
        Ok(())
    }

    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<()> {
        self.initialise(req);
        Err(err)
    }
}

impl AfterMiddleware for Logger {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        self.log(req, &res)?;
        Ok(res)
    }

    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
        self.log(req, &err.response)?;
        Err(err)
    }
}
