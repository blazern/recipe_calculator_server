extern crate clap;
extern crate futures;
extern crate recipe_calculator_lib;
extern crate serde_json;

use clap::{Arg, App};

use recipe_calculator_lib::config;
use recipe_calculator_lib::server::entry_point;
use recipe_calculator_lib::server::requests_handler_impl::RequestsHandlerImpl;

const CONFIG_ARG: &'static str = "config";
const ADDRESS_ARG: &'static str = "address";

fn main() {
    // NOTE: we have a lot of unwraps bellow, but this is intentional - if something goes wrong
    // at app startup, we want to know that as soon as possible.

    let example_config = config::Config::new(
        "<large hex number>",
        "postgres://recipe_calculator_server:P@ssw0rd@localhost/recipe_calculator_main",
        "postgres://recipe_calculator_client:P@ssw0rd@localhost/recipe_calculator_main");
    let example_config_json = serde_json::to_string_pretty(&example_config).unwrap();

    let matches = App::new("Recipe calculator server")
        .arg(Arg::with_name(CONFIG_ARG)
            .long(CONFIG_ARG)
            .help(&format!("Path to config file, needed config format:\n{}", &example_config_json))
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name(ADDRESS_ARG)
            .long(ADDRESS_ARG)
            .help("Address of the server")
            .required(true)
            .takes_value(true))
        .get_matches();

    let config_path = matches.value_of(CONFIG_ARG).unwrap();
    let address = matches.value_of(ADDRESS_ARG).unwrap();

    let mut config_file = std::fs::OpenOptions::new().read(true).open(config_path).unwrap();
    let config = config::Config::from(&mut config_file).unwrap();

    let address = address.parse().unwrap();
    let shutdown_signal = futures::future::empty::<(),()>();
    entry_point::start_server(&address, shutdown_signal, RequestsHandlerImpl::new(config));
}
