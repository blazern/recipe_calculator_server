#![recursion_limit = "128"]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;
#[cfg(test)]
#[macro_use]
extern crate lazy_static;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate percent_encoding;
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate tokio_core;
extern crate uuid;
pub mod config;
pub mod db;
pub mod error;
pub mod http_client;
pub mod server;
#[cfg(test)]
pub mod testing_utils;
pub mod vk;
