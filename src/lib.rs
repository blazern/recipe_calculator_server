#![recursion_limit="128"]
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate error_chain;
extern crate reqwest;
extern crate uuid;
pub mod schema;
pub mod db;
pub mod error;
pub mod command_line_args;
pub mod config;
pub mod vk;
