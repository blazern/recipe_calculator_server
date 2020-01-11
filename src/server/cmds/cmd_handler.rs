use std::collections::HashMap;

use futures::Future;
use serde_json::Value as JsonValue;
use std::sync::Arc;

use config::Config;
use db::pool::connection_pool::BorrowedDBConnection;
use http_client::HttpClient;

use server::request_error::RequestError;

pub trait CmdHandler {
    fn handle(
        &self,
        args: HashMap<String, String>,
        connection: BorrowedDBConnection,
        config: Config,
        http_client: Arc<HttpClient>,
    ) -> Box<dyn Future<Item = JsonValue, Error = RequestError> + Send>;
}
