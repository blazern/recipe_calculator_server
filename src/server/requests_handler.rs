use futures::Future;
use std::collections::HashMap;
use std::pin::Pin;

pub trait RequestsHandler: Send + Sync {
    fn handle(
        &mut self,
        request: String,
        query: String,
        headers: HashMap<String, String>,
        body: String,
    ) -> Pin<Box<dyn Future<Output = String> + Send>>;
}
