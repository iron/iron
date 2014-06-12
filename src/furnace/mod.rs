trait Request {}
trait Response {}

pub trait Furnace<Rq: Request, Rs: Response>: Send + Clone {
    fn forge(&self, _request: &mut Rq, _response: &mut Rs);
}

