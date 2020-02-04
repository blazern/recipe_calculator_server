#![recursion_limit = "128"]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_json;

pub mod config;
pub mod db;
pub mod error;
pub mod outside;
pub mod pairing;
pub mod server;
#[cfg(test)]
pub mod testing_utils;
pub mod utils;
