use recipe_calculator_lib::config::Config;
use recipe_calculator_lib::db::pool::connection_pool::ConnectionPool;

fn main() {
    let config =
        Config::new(
            "VK_SERVER_TOKEN".to_owned(),
            "FCM_SERVER_TOKEN".to_owned(),
            "PSQL_URL".to_owned(),
            "PSQL_URL".to_owned(),
            123);
    let pool = ConnectionPool::for_client_user(config);
    the_accepting_send_trait_fn(pool);
}

fn the_accepting_send_trait_fn(_pool: impl Send + Sync) {
}