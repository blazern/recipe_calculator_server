use std::net::ToSocketAddrs;

use clap::{App, Arg};
use log::info;

use recipe_calculator_lib::config;
use recipe_calculator_lib::db::core::migrator;
use recipe_calculator_lib::server::entry_point;
use recipe_calculator_lib::server::requests_handler_impl::RequestsHandlerImpl;

const CONFIG_ARG: &str = "config";
const LOG4RS_CONFIG_ARG: &str = "log4rs-config";
const ADDRESS_ARG: &str = "address";

fn main() {
    // NOTE: we have a lot of unwraps bellow, but this is intentional - if something goes wrong
    // at app startup, we want to know that as soon as possible.

    let example_config = config::Config::new(
        "<large hex number>".to_owned(),
        "<large string>".to_owned(),
        "postgres://recipe_calculator_server:P@ssw0rd@localhost/recipe_calculator_main".to_owned(),
        "postgres://recipe_calculator_client:P@ssw0rd@localhost/recipe_calculator_main".to_owned(),
        180,
    );
    let example_config_json = serde_json::to_string_pretty(&example_config).unwrap();

    let matches = App::new("Recipe calculator server")
        .arg(
            Arg::with_name(CONFIG_ARG)
                .long(CONFIG_ARG)
                .help(&format!(
                    "Path to config file, needed config format:\n{}",
                    &example_config_json
                ))
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(ADDRESS_ARG)
                .long(ADDRESS_ARG)
                .help("Address of the server")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(LOG4RS_CONFIG_ARG)
                .long(LOG4RS_CONFIG_ARG)
                .help("Path to a config file which regulates log4rs logging")
                .default_value("log4rs-default-config.yaml")
                .takes_value(true),
        )
        .get_matches();

    let config_path = matches.value_of(CONFIG_ARG).unwrap();
    let address = matches.value_of(ADDRESS_ARG).unwrap();
    let log4rs_config_path = matches.value_of(LOG4RS_CONFIG_ARG).unwrap();

    println!("log4rs config path: {}", log4rs_config_path);
    log4rs::init_file(log4rs_config_path, Default::default()).unwrap();

    let mut config_file = std::fs::OpenOptions::new()
        .read(true)
        .open(config_path)
        .unwrap();
    let config = config::Config::from(&mut config_file).unwrap();
    let config_json = serde_json::to_string_pretty(&config).unwrap();
    info!("Received config:\n{}", config_json);

    let mut address = address.to_socket_addrs().unwrap();
    let address = address.next().unwrap();
    let shutdown_signal = futures::future::pending();

    info!("Performing migrations");
    migrator::migrate_with_timeout(
        config.psql_diesel_url_server_user(),
        config.db_connection_attempts_timeout_seconds() as i64,
    )
    .unwrap();

    info!("Starting listening to address: {}", address);
    entry_point::start_server(
        &address,
        shutdown_signal,
        RequestsHandlerImpl::new(config).unwrap(),
    );
}
