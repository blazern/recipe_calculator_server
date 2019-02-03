pub trait RequestsHandler : Send + Sync {
    fn handle(&mut self, request: &str, query: Option<&str>) -> String;
}