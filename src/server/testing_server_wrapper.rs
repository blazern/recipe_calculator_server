use futures::future::Future;
use futures::sync::oneshot;
use std::thread;

use server::entry_point;
use server::requests_handler::RequestsHandler;

// Wrapper for server started by 'entry_point::start_server'.
// Starts server on background thread (so that tests would be blocked by the server), stops
// server in its destructor.
#[cfg(test)]
pub struct ServerWrapper {
    sender: Option<oneshot::Sender<()>>,
}

impl ServerWrapper {
    fn new<RH>(address: &str, requests_handler: RH) -> ServerWrapper where RH: RequestsHandler + 'static {
        let (sender, receiver) = oneshot::channel::<()>();
        let receiver = receiver.map_err(|_| ());

        let address = address.parse().unwrap();
        thread::spawn(move || {
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

pub fn start_server<RH>(address: &str, requests_handler: RH) -> ServerWrapper where RH: RequestsHandler + 'static {
    return ServerWrapper::new(address, requests_handler);
}