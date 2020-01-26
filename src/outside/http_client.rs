use std::collections::HashMap;

use futures::future::err;
use futures::Future;
use futures::Stream;

use hyper::client::HttpConnector;
use hyper::Body;
use hyper::Client;
use hyper::Method;
use hyper::Request;
use hyper::Uri;
use hyper_tls::HttpsConnector;

use super::error::Error;

pub enum RequestMethod {
    Post,
    Get,
}

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

    pub fn req_get(&self, url: Uri) -> impl Future<Item = String, Error = Error> {
        self.req(url, RequestMethod::Get, HashMap::new(), None)
    }

    pub fn req(
        &self,
        url: Uri,
        method: RequestMethod,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> Box<dyn Future<Item = String, Error = Error> + Send> {
        let request = Self::create_request_obj(url, method, headers, body);
        let request = match request {
            Err(error) => return Box::new(err(error)),
            Ok(request) => request,
        };
        let response = self.hyper_client.request(request);
        let response = response
            .map(|val| {
                val.into_body()
                    .concat2()
                    .map(|body| String::from_utf8_lossy(&body).to_string())
            })
            .flatten()
            .map_err(|err| err.into());
        Box::new(response)
    }

    fn create_request_obj(
        url: Uri,
        method: RequestMethod,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> Result<Request<Body>, Error> {
        let mut req = Request::builder();
        req.uri(url).method::<Method>(method.into());

        for (key, val) in headers {
            let key: &str = &key;
            let val: &str = &val;
            req.header(key, val);
        }

        let res = match body {
            Some(body) => req.body(Body::from(body)),
            None => req.body(Body::from("")),
        };
        res.map_err(|e| e.into())
    }
}

impl Into<Method> for RequestMethod {
    fn into(self) -> Method {
        match self {
            RequestMethod::Get => Method::GET,
            RequestMethod::Post => Method::POST,
        }
    }
}
