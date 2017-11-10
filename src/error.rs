extern crate reqwest;
extern crate serde_json;
extern crate url;
extern crate diesel;
extern crate uuid;

use std;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        SerdeJson(serde_json::error::Error);
        Reqwest(reqwest::Error);
        UrlParse(url::ParseError);
        DieselError(diesel::result::Error);
        UuidParseError(uuid::ParseError);
    }
}