use futures::Future;

pub trait RequestsHandler : Send + Sync {
    fn handle(&mut self, request: String, query: String)
        -> Box<Future<Item=String, Error=()> + Send>;
}