use futures::Future;
use std::collections::HashMap;

pub trait RequestsHandler: Send + Sync {
    fn handle(
        &mut self,
        request: String,
        query: String,
        headers: HashMap<String, String>,
        body: String,
    ) -> Box<dyn Future<Item = String, Error = ()> + Send>;
}
