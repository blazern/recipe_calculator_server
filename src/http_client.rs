use std::str::FromStr;

use futures::Future;
use futures::Stream;

use hyper::Client;
use hyper::Uri;
use hyper_tls::HttpsConnector;

use tokio_core;

use error::Error;

// TODO: remove this function, all requests must be asynchronous
pub fn make_blocking_request(url_str: &str) -> Result<String, Error> {
    let https = HttpsConnector::new(1)?;
    let client = Client::builder().build::<_, hyper::Body>(https);
    let url = Uri::from_str(url_str)?;

    let mut tokio_core = tokio_core::reactor::Core::new()?;
    let response = tokio_core.run(client.get(url))?;

    let body = response.into_body();
    let result = body.concat2()
        .map(|body| {
            String::from_utf8_lossy(&body).to_string()
        }).wait()?;

    return Ok(result);
}