use futures::future::Future;

pub trait RequestsHandler : Send + Sync {
    fn handle(&mut self, request: &str, query: Option<&str>) -> Box<Future<Item=String, Error=()>>;
}