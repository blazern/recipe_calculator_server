use futures::future::Future;
use futures::sync::oneshot;
use std::sync::MutexGuard;
use std::thread;

use server::entry_point;
use server::requests_handler::RequestsHandler;
use server::testing_hostname;

// Wrapper for server started by 'entry_point::start_server'.
// Starts server on background thread (so that tests would be blocked by the server), stops
// server in its destructor.
#[cfg(test)]
pub struct ServerWrapper {
    finish_cmd_sender: Option<oneshot::Sender<()>>,
    finish_event_receiver: Option<Box<dyn Future<Item = (), Error = ()>>>,
    address: MutexGuard<'static, String>,
}

impl ServerWrapper {
    fn new<RH>(requests_handler: RH) -> ServerWrapper
    where
        RH: RequestsHandler + 'static,
    {
        // Channel for sending finish cmd - generates a Future to send a STOP cmd to Hyper.
        let (finish_cmd_sender, finish_cmd_receiver) = oneshot::channel::<()>();
        let finish_cmd_receiver = finish_cmd_receiver.map_err(|_| ());

        // Channel for receiving finish event - address'es mutex should be released only when
        // the server finishes its work, otherwise a new server would try to start on occupied port.
        let (finish_event_sender, finish_event_receiver) = oneshot::channel::<()>();
        let finish_event_receiver = finish_event_receiver.map_err(|_| ());

        // Channel for receiving start event -
        // this method should return only when server is started, otherwise testing code sometimes
        // will stumble upon Connection Refused (111) error.
        let (start_event_sender, start_event_receiver) = oneshot::channel::<()>();
        let start_event_receiver = start_event_receiver.map_err(|_| ());

        let address = testing_hostname::get_hostname();
        let sock_address = address.parse().unwrap();
        thread::spawn(move || {
            start_event_sender.send(()).unwrap();
            entry_point::start_server(&sock_address, finish_cmd_receiver, requests_handler);
            finish_event_sender.send(()).unwrap();
        });

        start_event_receiver.wait().unwrap();
        ServerWrapper {
            finish_cmd_sender: Some(finish_cmd_sender),
            finish_event_receiver: Some(Box::new(finish_event_receiver)),
            address,
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }
}

impl Drop for ServerWrapper {
    fn drop(&mut self) {
        match (
            self.finish_cmd_sender.take(),
            self.finish_event_receiver.take(),
        ) {
            (Some(finish_cmd_sender), Some(finish_event_receiver)) => {
                finish_cmd_sender.send(()).unwrap();
                finish_event_receiver.wait().unwrap();
            }
            (_, _) => {
                panic!("Sender or Receiver is None already somehow!");
            }
        }
    }
}

pub fn start_server<RH>(requests_handler: RH) -> ServerWrapper
where
    RH: RequestsHandler + 'static,
{
    return ServerWrapper::new(requests_handler);
}
