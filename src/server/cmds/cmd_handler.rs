use std::collections::HashMap;
use std::pin::Pin;

use futures::Future;
use serde_json::Value as JsonValue;
use std::sync::Arc;

use crate::config::Config;
use crate::db::pool::connection_pool::ConnectionPool;
use crate::outside::http_client::HttpClient;
use crate::server::request_error::RequestError;

pub type CmdHandleResult = Result<JsonValue, RequestError>;
pub type CmdHandleResultFuture = Pin<Box<dyn Future<Output = CmdHandleResult> + Send>>;

pub trait CmdHandler {
    fn handle(
        &self,
        args: HashMap<String, String>,
        connections_pool: ConnectionPool,
        config: Config,
        http_client: Arc<HttpClient>,
    ) -> CmdHandleResultFuture;
}
