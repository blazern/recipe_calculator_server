use futures::channel::oneshot;
use futures::future::Future;
use futures::future::FutureExt;
use std::sync::MutexGuard;
use std::thread;

use crate::server::entry_point;
use crate::server::requests_handler::RequestsHandler;

// Wrapper for server started by 'entry_point::start_server'.
// Starts server on background thread (so that tests would be blocked by the server), stops
// server in its destructor.
#[cfg(test)]
pub struct ServerWrapper {
    finish_cmd_sender: Option<oneshot::Sender<()>>,
    finish_event_receiver: Option<Box<dyn Future<Output = ()> + Unpin>>,
    address: MutexGuard<'static, String>,
}

impl ServerWrapper {
    fn new<RH>(requests_handler: RH, address: MutexGuard<'static, String>) -> ServerWrapper
    where
        RH: RequestsHandler + 'static,
    {
        // Channel for sending finish cmd - generates a Future to send a STOP cmd to Hyper.
        let (finish_cmd_sender, finish_cmd_receiver) = oneshot::channel::<()>();

        // Channel for receiving finish event - address'es mutex should be released only when
        // the server finishes its work, otherwise a new server would try to start on occupied port.
        let (finish_event_sender, finish_event_receiver) = oneshot::channel::<()>();

        // Channel for receiving start event -
        // this method should return only when server is started, otherwise testing code sometimes
        // will stumble upon Connection Refused (111) error.
        let (start_event_sender, start_event_receiver) = oneshot::channel::<()>();

        let sock_address = address.parse().unwrap();
        thread::spawn(move || {
            start_event_sender.send(()).unwrap();
            let finish_cmd_receiver = finish_cmd_receiver.map(|_| ());
            entry_point::start_server(&sock_address, finish_cmd_receiver, requests_handler);
            finish_event_sender.send(()).unwrap();
        });

        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(start_event_receiver)
            .unwrap();
        ServerWrapper {
            finish_cmd_sender: Some(finish_cmd_sender),
            finish_event_receiver: Some(Box::new(finish_event_receiver.map(|_| ()))),
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
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(finish_event_receiver);
            }
            (_, _) => {
                panic!("Sender or Receiver is None already somehow!");
            }
        }
    }
}

pub fn start_server<RH>(requests_handler: RH, address: MutexGuard<'static, String>) -> ServerWrapper
where
    RH: RequestsHandler + 'static,
{
    ServerWrapper::new(requests_handler, address)
}
