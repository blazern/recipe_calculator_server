use futures::Future;
use futures::Stream;

use hyper::client::HttpConnector;
use hyper::Client;
use hyper::Uri;
use hyper_tls::HttpsConnector;

use error::Error;

pub struct HttpClient {
    hyper_client: Client<HttpsConnector<HttpConnector>>,
}

impl HttpClient {
    pub fn new() -> Result<HttpClient, Error> {
        // TODO: control number of threads
        let https = HttpsConnector::new(1)?;
        let client = Client::builder().build::<_, hyper::Body>(https);
        Ok(HttpClient {
            hyper_client: client,
        })
    }

    pub fn make_request(&self, url: Uri) -> impl Future<Item = String, Error = Error> {
        let response = self.hyper_client.get(url);
        response
            .map(|val| {
                val.into_body()
                    .concat2()
                    .map(|body| String::from_utf8_lossy(&body).to_string())
            })
            .flatten()
            .map_err(|err| err.into())
    }
}
