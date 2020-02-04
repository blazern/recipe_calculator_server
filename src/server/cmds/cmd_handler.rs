use std::collections::HashMap;

use futures::Future;
use serde_json::Value as JsonValue;
use std::sync::Arc;

use crate::config::Config;
use crate::db::pool::connection_pool::BorrowedDBConnection;
use crate::outside::http_client::HttpClient;

use crate::server::request_error::RequestError;

pub type CmdHandleResult = Box<dyn Future<Item = JsonValue, Error = RequestError> + Send>;

pub trait CmdHandler {
    fn handle(
        &self,
        args: HashMap<String, String>,
        connection: BorrowedDBConnection,
        config: Config,
        http_client: Arc<HttpClient>,
    ) -> CmdHandleResult;
}
