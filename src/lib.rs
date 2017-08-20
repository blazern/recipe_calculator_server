#[macro_use] extern crate serde_derive;
#[macro_use] extern crate error_chain;
extern crate reqwest;
pub mod error;
pub mod command_line_args;
pub mod config;
pub mod vk;