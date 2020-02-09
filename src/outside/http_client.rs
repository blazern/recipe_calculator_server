use std::collections::HashMap;
use std::sync::Arc;

use futures::{Future, StreamExt};

use hyper::client::HttpConnector;
use hyper::client::ResponseFuture;
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

#[derive(Debug)]
pub struct Response {
    pub body: String,
    pub status_code: u16,
}

pub struct HttpClient {
    hyper_client: Arc<Client<HttpsConnector<HttpConnector>>>,
}

impl HttpClient {
    pub fn new() -> Result<HttpClient, Error> {
        // TODO: control number of threads
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        Ok(HttpClient {
            hyper_client: Arc::new(client),
        })
    }

    pub fn req_get(&self, url: Uri) -> impl Future<Output = Result<Response, Error>> {
        self.req(url, RequestMethod::Get, HashMap::new(), None)
    }

    pub fn req(
        &self,
        url: Uri,
        method: RequestMethod,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> impl Future<Output = Result<Response, Error>> {
        let hyper_client = self.hyper_client.clone();
        async move {
            let request = Self::create_request_obj(url, method, headers, body)?;
            let response = hyper_client.request(request);
            Self::transform_response(response).await
        }
    }

    async fn transform_response(response: ResponseFuture) -> Result<Response, Error> {
        let response = response.await?;
        let status_code = response.status().as_u16();

        let mut bytes = Vec::new();
        let mut body_stream = response.into_body();
        while let Some(chunk) = body_stream.next().await {
            bytes.append(&mut chunk?.to_vec());
        }

        let body = String::from_utf8_lossy(&bytes).to_string();
        Ok(Response { body, status_code })
    }

    fn create_request_obj(
        url: Uri,
        method: RequestMethod,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> Result<Request<Body>, Error> {
        let mut req = Request::builder();
        req = req.uri(url).method::<Method>(method.into());

        for (key, val) in headers {
            let key: &str = &key;
            let val: &str = &val;
            req = req.header(key, val);
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
