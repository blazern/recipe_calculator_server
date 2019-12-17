use futures::Future;

pub trait RequestsHandler: Send + Sync {
    fn handle(
        &mut self,
        request: String,
        query: String,
    ) -> Box<dyn Future<Item = String, Error = ()> + Send>;
}
