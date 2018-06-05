#![recursion_limit="128"]
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate error_chain;
#[cfg(test)] #[macro_use] extern crate lazy_static;
extern crate hyper;
extern crate futures;
extern crate serde;
#[macro_use] extern crate serde_json;
extern crate reqwest;
extern crate uuid;
#[cfg(test)] pub mod testing_config;
pub mod schema;
pub mod db;
pub mod error;
pub mod config;
pub mod vk;
pub mod server;