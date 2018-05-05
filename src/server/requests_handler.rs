use hyper::Uri;
use futures::future::Future;

pub trait RequestsHandler : Send + Sync {
    fn handle(&self, request: &Uri) -> Box<Future<Item=String, Error=()>>;
}