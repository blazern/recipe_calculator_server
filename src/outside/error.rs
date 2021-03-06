error_chain! {
    foreign_links {
        SerdeJson(serde_json::error::Error);
        InvalidUri(hyper::http::uri::InvalidUri);
        HyperError(hyper::Error);
        HyperHttpError(hyper::http::Error);
    }

    errors {
        // TODO: ensure that panic with these errors shows parent errors correctly (with stacks)
        UnexpectedResponseFormat(msg: String) {
            description("Unexpected response format from outside"),
            display("{}", msg),
        }
    }
}
