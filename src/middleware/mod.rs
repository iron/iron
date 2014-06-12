trait Response {}
trait Request {}

pub enum MiddlewareStatus {
    Continue, Unwind
}

pub trait Middleware<Rq: Request, Rs: Response>: Send + Clone {
    fn descend(&mut self, _request: &mut Rq, _response: &mut Rs) -> MiddlewareStatus {
        Continue
    }
    fn ascend(&mut self, _request: &mut Rq, _response: &mut Rs) -> MiddlewareStatus {
        Continue
    }
}

pub trait MiddlewareStack<Rq: Request, Rs: Response>: Send + Clone {
    fn handle(&self, _request: &mut Rq, _response: &mut Rs);
}

