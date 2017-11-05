extern crate docopt;

use std;
use self::docopt::Docopt;

const USAGE: &'static str = "
Recipe calculator server.

Usage:
  recipe_calculator_server <config-path>
  recipe_calculator_server (-h | --help)

Options:
  -h --help     Show this screen.
";

#[derive(Debug, Deserialize)]
pub struct Args {
    arg_config_path: String,
}

impl Args {
    pub fn config_path(&self) -> &String {
        &self.arg_config_path
    }
}

pub fn get() -> Result<Args, docopt::Error> {
    return parse(std::env::args());
}

pub fn parse<I, S>(args: I) -> Result<Args, docopt::Error>
            where I: IntoIterator<Item=S>, S: AsRef<str>{
    let args: Result<Args, docopt::Error> =
        Docopt::new(USAGE)?
            .argv(args)
            .deserialize();
    return args;
}

#[cfg(test)]
#[path = "./command_line_args_test.rs"]
mod command_line_args_test;