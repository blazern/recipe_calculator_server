extern crate reqwest;
extern crate serde_json;
extern crate url;

use std;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        SerdeJson(serde_json::error::Error);
        Reqwest(reqwest::Error);
        UrlParse(url::ParseError);
    }
}