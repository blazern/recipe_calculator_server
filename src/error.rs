use std;
use std::fmt;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        SerdeJson(serde_json::error::Error);
        UuidParseError(uuid::parser::ParseError);
    }
}

#[derive(Debug)]
pub enum Never {}
impl fmt::Display for Never {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        match *self {}
    }
}
impl std::error::Error for Never {
    fn description(&self) -> &str {
        match *self {}
    }
}
