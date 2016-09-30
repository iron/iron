//! Iron's Middleware and Handler System
//!
//! Iron's Middleware system is best modeled with a diagram.
//!
//! ```plain
//! [b] = BeforeMiddleware
//! [a] = AfterMiddleware
//! [[h]] = AroundMiddleware
//! [h] = Handler
//! ```
//!
//! With no errors, the flow looks like:
//!
//! ```plain
//! [b] -> [b] -> [b] -> [[[[h]]]] -> [a] -> [a] -> [a] -> [a]
//! ```
//!
//! A request first travels through all BeforeMiddleware, then a Response is generated
//! by the Handler, which can be an arbitrary nesting of AroundMiddleware, then all
//! AfterMiddleware are called with both the Request and Response. After all AfterMiddleware
//! have been fired, the response is written back to the client.
//!
//! Iron's error handling system is pragmatic and focuses on tracking two pieces
//! of information for error receivers (other middleware):
//!
//! * The cause of the error
//! * The result (what to do about) the error.
//!
//! The cause of the error is represented simply by the error itself, and the result
//! of the error, representing the action to take in response to the error, is a complete
//! Response, which will be sent at the end of the error flow.
//!
//! When an error is thrown in Iron by any middleware or handler returning an `Err`
//! variant with an `IronError`, the flow of the Request switches to the error flow,
//! which proceeds to just call the `catch` method of middleware and sidesteps the
//! `Handler` entirely, since there is already a Response in the error.
//!
//! A Request can exit the error flow by returning an Ok from any of the catch methods.
//! This resumes the flow at the middleware immediately following the middleware which
//! handled the error. It is impossible to "go back" to an earlier middleware that was
//! skipped.
//!
//! Generally speaking, returning a 5XX error code means that the error flow should be
//! entered by raising an explicit error. Dealing with 4XX errors is trickier, since
//! the server may not want to recognize an error that is entirely the clients fault;
//! handling of 4XX error codes is up to to each application and middleware author.
//!
//! Middleware authors should be cognizant that their middleware may be skipped during
//! the error flow. Anything that *must* be done to each Request or Response should
//! be run during both the normal and error flow by implementing the `catch` method to
//! also do the necessary action.
//!

use std::sync::Arc;
use {Request, Response, IronResult, IronError};

/// `Handler`s are responsible for handling requests by creating Responses from Requests.
pub trait Handler: Send + Sync + 'static {
    /// Produce a `Response` from a Request, with the possibility of error.
    fn handle(&self, &mut Request) -> IronResult<Response>;
}

/// `BeforeMiddleware` are fired before a `Handler` is called inside of a Chain.
///
/// `BeforeMiddleware` are responsible for doing request pre-processing that requires
/// the ability to change control-flow, such as authorization middleware, or for editing
/// the request by modifying the headers.
///
/// `BeforeMiddleware` only have access to the Request, if you need to modify or read
/// a Response, you will need `AfterMiddleware`. Middleware which wishes to send an
/// early response that is not an error cannot be `BeforeMiddleware`, but should
/// instead be `AroundMiddleware`.
pub trait BeforeMiddleware: Send + Sync + 'static {
    /// Do whatever work this middleware should do with a `Request` object.
    fn before(&self, _: &mut Request) -> IronResult<()> { Ok(()) }

    /// Respond to an error thrown by a previous `BeforeMiddleware`.
    ///
    /// Returning a `Ok` will cause the request to resume the normal flow at the
    /// next `BeforeMiddleware`, or if this was the last `BeforeMiddleware`,
    /// at the `Handler`.
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<()> { Err(err) }
}

/// `AfterMiddleware` are fired after a `Handler` is called inside of a Chain.
///
/// `AfterMiddleware` receive both a `Request` and a `Response` and are responsible for doing
/// any response post-processing.
///
/// `AfterMiddleware` should *not* overwrite the contents of a Response. In
/// the common case, a complete response is generated by the Chain's `Handler` and
/// `AfterMiddleware` simply do post-processing of that Response, such as
/// adding headers or logging.
pub trait AfterMiddleware: Send + Sync + 'static {
    /// Do whatever post-processing this middleware should do.
    fn after(&self, _: &mut Request, res: Response) -> IronResult<Response> {
        Ok(res)
    }

    /// Respond to an error thrown by previous `AfterMiddleware`, the `Handler`,
    /// or a `BeforeMiddleware`.
    ///
    /// Returning `Ok` will cause the request to resume the normal flow at the
    /// next `AfterMiddleware`.
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<Response> {
        Err(err)
    }
}

/// AroundMiddleware are used to wrap and replace the `Handler` in a `Chain`.
///
/// AroundMiddleware produce `Handler`s through their `around` method, which is
/// called once on insertion into a Chain or can be called manually outside of a
/// `Chain`.
pub trait AroundMiddleware {
    /// Produce a `Handler` from this `AroundMiddleware` given another `Handler`.
    ///
    /// Usually this means wrapping the handler and editing the `Request` on the
    /// way in and the `Response` on the way out.
    ///
    /// This is called only once, when an `AroundMiddleware` is added to a `Chain`
    /// using `Chain::around`, it is passed the `Chain`'s current `Handler`.
    fn around(self, handler: Box<Handler>) -> Box<Handler>;
}

/// The middleware chain used in Iron.
///
/// This is a canonical implementation of Iron's middleware system,
/// but Iron's infrastructure is flexible enough to allow alternate
/// systems.
pub struct Chain {
    befores: Vec<Box<BeforeMiddleware>>,
    afters: Vec<Box<AfterMiddleware>>,

    // Internal invariant: this is always Some
    handler: Option<Box<Handler>>
}

impl Chain {
    /// Construct a new Chain from a `Handler`.
    pub fn new<H: Handler>(handler: H) -> Chain {
        Chain {
            befores: vec![],
            afters: vec![],
            handler: Some(Box::new(handler) as Box<Handler>)
        }
    }

    /// Link both a before and after middleware to the chain at once.
    ///
    /// Middleware that have a Before and After piece should have a constructor
    /// which returns both as a tuple, so it can be passed directly to link.
    pub fn link<B, A>(&mut self, link: (B, A)) -> &mut Chain
    where A: AfterMiddleware, B: BeforeMiddleware {
        let (before, after) = link;
        self.befores.push(Box::new(before) as Box<BeforeMiddleware>);
        self.afters.push(Box::new(after) as Box<AfterMiddleware>);
        self
    }

    /// Link a `BeforeMiddleware` to the `Chain`, after all previously linked
    /// `BeforeMiddleware`.
    pub fn link_before<B>(&mut self, before: B) -> &mut Chain
    where B: BeforeMiddleware {
        self.befores.push(Box::new(before) as Box<BeforeMiddleware>);
        self
    }

    /// Link a `AfterMiddleware` to the `Chain`, after all previously linked
    /// `AfterMiddleware`.
    pub fn link_after<A>(&mut self, after: A) -> &mut Chain
    where A: AfterMiddleware {
        self.afters.push(Box::new(after) as Box<AfterMiddleware>);
        self
    }

    /// Apply an `AroundMiddleware` to the `Handler` in this `Chain`.
    ///
    /// Note: This function is being renamed `link_around()`, and will
    /// eventually be removed.
    pub fn around<A>(&mut self, around: A) -> &mut Chain
    where A: AroundMiddleware {
        self.link_around(around)
    }

    /// Apply an `AroundMiddleware` to the `Handler` in this `Chain`.
    pub fn link_around<A>(&mut self, around: A) -> &mut Chain
    where A: AroundMiddleware {
        let mut handler = self.handler.take().unwrap();
        handler = around.around(handler);
        self.handler = Some(handler);
        self
    }
}

/// Builder struct for a `Chain`
///
pub struct ChainLink {
    chain: Chain
}


impl ChainLink {
    /// Construct a new `ChainLink` from a `Handler`.
    pub fn new<H: Handler>(handler: H) -> ChainLink {
        ChainLink {
            chain: Chain::new(handler)
        }
    }

    /// Link both a before and after middleware to the `ChainLink` at once.
    ///
    /// Middleware that have a Before and After piece should have a constructor
    /// which returns both as a tuple, so it can be passed directly to link.
    pub fn link<B, A>(mut self, link: (B, A)) -> ChainLink
    where A: AfterMiddleware, B: BeforeMiddleware {
        self.chain.link(link);
        self
    }

    /// Link a `BeforeMiddleware` to the `ChainLink`, after all previously linked
    /// `BeforeMiddleware`.
    pub fn link_before<B>(mut self, before: B) -> ChainLink
    where B: BeforeMiddleware {
        self.chain.link_before(before);
        self
    }

    /// Link a `AfterMiddleware` to the `ChainLink`, after all previously linked
    /// `AfterMiddleware`.
    pub fn link_after<A>(mut self, after: A) -> ChainLink
    where A: AfterMiddleware {
        self.chain.link_after(after);
        self
    }

    /// Apply an `AroundMiddleware` to the `Handler` in this `ChainLink`.
    pub fn around<A>(mut self, around: A) -> ChainLink
    where A: AroundMiddleware {
        self.chain.around(around);
        self
    }

    /// Return the built `Chain`
    pub fn lock_chain(self) -> Chain {
        self.chain
    }
}

impl Handler for Chain {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        // Kick off at befores, which will continue into handler
        // then afters.
        self.continue_from_before(req, 0)
    }
}

impl Chain {
    ///////////////// Implementation Helpers /////////////////

    // Enter the error flow from a before middleware, starting
    // at the passed index.
    //
    // If the index is out of bounds for the before middleware Vec,
    // this instead behaves the same as fail_from_handler.
    fn fail_from_before(&self, req: &mut Request, index: usize,
                        mut err: IronError) -> IronResult<Response> {
        // If this was the last before, yield to next phase.
        if index >= self.befores.len() {
            return self.fail_from_handler(req, err)
        }

        for (i, before) in self.befores[index..].iter().enumerate() {
            err = match before.catch(req, err) {
                Err(err) => err,
                Ok(()) => return self.continue_from_before(req, index + i + 1)
            };
        }

        // Next phase
        self.fail_from_handler(req, err)
    }

    // Enter the error flow from an errored handle, starting with the
    // first AfterMiddleware.
    fn fail_from_handler(&self, req: &mut Request,
                         err: IronError) -> IronResult<Response> {
        // Yield to next phase, nothing to do here.
        self.fail_from_after(req, 0, err)
    }

    // Enter the error flow from an errored after middleware, starting
    // with the passed index.
    //
    // If the index is out of bounds for the after middleware Vec,
    // this instead just returns the passed error.
    fn fail_from_after(&self, req: &mut Request, index: usize,
                       mut err: IronError) -> IronResult<Response> {
        // If this was the last after, we're done.
        if index == self.afters.len() { return Err(err) }

        for (i, after) in self.afters[index..].iter().enumerate() {
            err = match after.catch(req, err) {
                Err(err) => err,
                Ok(res) => return self.continue_from_after(req, index + i + 1, res)
            }
        }

        // Done
        Err(err)
    }

    // Enter the normal flow in the before middleware, starting with the passed
    // index.
    fn continue_from_before(&self, req: &mut Request,
                            index: usize) -> IronResult<Response> {
        // If this was the last beforemiddleware, start at the handler.
        if index >= self.befores.len() {
            return self.continue_from_handler(req)
        }

        for (i, before) in self.befores[index..].iter().enumerate() {
            match before.before(req) {
                Ok(()) => {},
                Err(err) => return self.fail_from_before(req, index + i + 1, err)
            }
        }

        // Yield to next phase.
        self.continue_from_handler(req)
    }

    // Enter the normal flow at the handler.
    fn continue_from_handler(&self, req: &mut Request) -> IronResult<Response> {
        // unwrap is safe because it's always Some
        match self.handler.as_ref().unwrap().handle(req) {
            Ok(res) => self.continue_from_after(req, 0, res),
            Err(err) => self.fail_from_handler(req, err)
        }
    }

    // Enter the normal flow in the after middleware, starting with the passed
    // index.
    fn continue_from_after(&self, req: &mut Request, index: usize,
                            mut res: Response) -> IronResult<Response> {
        // If this was the last after middleware, we're done.
        if index >= self.afters.len() {
            return Ok(res);
        }

        for (i, after) in self.afters[index..].iter().enumerate() {
            res = match after.after(req, res) {
                Ok(r) => r,
                Err(err) => return self.fail_from_after(req, index + i + 1, err)
            }
        }

        // We made it with no error!
        Ok(res)
    }
}

impl<F> Handler for F
where F: Send + Sync + 'static + Fn(&mut Request) -> IronResult<Response> {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        (*self)(req)
    }
}

impl Handler for Box<Handler> {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        (**self).handle(req)
    }
}

impl<F> BeforeMiddleware for F
where F: Send + Sync + 'static + Fn(&mut Request) -> IronResult<()> {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        (*self)(req)
    }
}

impl BeforeMiddleware for Box<BeforeMiddleware> {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        (**self).before(req)
    }

    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<()> {
        (**self).catch(req, err)
    }
}

impl<T> BeforeMiddleware for Arc<T> where T: BeforeMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        (**self).before(req)
    }

    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<()> {
        (**self).catch(req, err)
    }
}

impl<F> AfterMiddleware for F
where F: Send + Sync + 'static + Fn(&mut Request, Response) -> IronResult<Response> {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        (*self)(req, res)
    }
}

impl AfterMiddleware for Box<AfterMiddleware> {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        (**self).after(req, res)
    }

    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
        (**self).catch(req, err)
    }
}

impl<T> AfterMiddleware for Arc<T> where T: AfterMiddleware {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        (**self).after(req, res)
    }

    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
        (**self).catch(req, err)
    }
}

impl<F> AroundMiddleware for F
where F: FnOnce(Box<Handler>) -> Box<Handler> {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        self(handler)
    }
}

#[cfg(test)]
mod test;
