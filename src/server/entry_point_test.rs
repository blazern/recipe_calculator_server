use futures::Future;
use futures::sync::oneshot;
use reqwest::Client;
use std::thread;
use std::io::Read;

use server::entry_point;

const SERVER_ADDRESS: &str = "127.0.0.1:3000";

// Wrapper for server started by 'entry_point::start_server'.
// Starts server on background thread (so that tests would be blocked by the server), stops
// server in its destructor.
struct ServerWrapper {
    sender: Option<oneshot::Sender<()>>,
}

impl ServerWrapper {
    fn new() -> ServerWrapper {
        let (sender, receiver) = oneshot::channel::<()>();
        let receiver = receiver.map_err(|_| ());

        thread::spawn(|| {
            let address = SERVER_ADDRESS.parse().unwrap();
            entry_point::start_server(&address, receiver);
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

fn start_server() -> ServerWrapper {
    return ServerWrapper::new();
}

#[test]
fn test() {
    let _server = start_server();
    let client = Client::new().unwrap();

    let url = format!("http://{}", SERVER_ADDRESS);
    let mut response = client.get(&url).unwrap().send().unwrap();
    let mut response_str = String::new();
    response.read_to_string(&mut response_str).unwrap();

    assert_eq!(entry_point::RESPONSE, response_str);
}