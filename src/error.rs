extern crate serde_json;
extern crate diesel;
extern crate hyper;
extern crate hyper_tls;
extern crate uuid;

use std;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        SerdeJson(serde_json::error::Error);
        UuidParseError(uuid::ParseError);
        InvalidUri(hyper::http::uri::InvalidUri);
        HyperError(hyper::Error);
        HyperTlsError(hyper_tls::Error);
    }
}