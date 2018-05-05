use hyper::Uri;
use futures::future;
use futures::future::Future;
use futures::sync::oneshot;
use reqwest::Client;
use std::thread;
use std::io::Read;

use server::entry_point;
use server::requests_handler::RequestsHandler;

// TODO: wrap the constant into a mutex so that 2 thread wouldn't conflict because of it
const SERVER_ADDRESS: &str = "127.0.0.1:3000";

// Wrapper for server started by 'entry_point::start_server'.
// Starts server on background thread (so that tests would be blocked by the server), stops
// server in its destructor.
struct ServerWrapper {
    sender: Option<oneshot::Sender<()>>,
}

struct Echo {
    string: String,
}

impl RequestsHandler for Echo {
    fn handle(&self, _request: &Uri) -> Box<Future<Item=String, Error=()>> {
        Box::new(future::ok(self.string.clone()))
    }
}

impl ServerWrapper {
    fn new<RH>(requests_handler: RH) -> ServerWrapper where RH: RequestsHandler + 'static {
        let (sender, receiver) = oneshot::channel::<()>();
        let receiver = receiver.map_err(|_| ());

        thread::spawn(move || {
            let address = SERVER_ADDRESS.parse().unwrap();
            entry_point::start_server(&address, receiver, requests_handler);
        });

        ServerWrapper {
            sender: Some(sender)
        }
    }
}

impl Drop for ServerWrapper {
    fn drop(&mut self) {
        match self.sender.take() {
            Some(sender) => {
                sender.send(()).unwrap();
            },
            None => {
                panic!("Sender is None already somehow!");
            }
        }
    }
}

fn start_server<RH>(requests_handler: RH) -> ServerWrapper where RH: RequestsHandler + 'static {
    return ServerWrapper::new(requests_handler);
}

#[test]
fn test_server_responses() {
    let expected_response = "Hello, world";
    let _server = start_server(Echo{ string: expected_response.to_string() });;
    let client = Client::new().unwrap();

    let url = format!("http://{}", SERVER_ADDRESS);
    let mut response = client.get(&url).unwrap().send().unwrap();
    let mut response_str = String::new();
    response.read_to_string(&mut response_str).unwrap();

    assert_eq!(expected_response, response_str);
}

#[test]
fn test_register_client_cmd() {
//    panic!("not implemented yet");
}