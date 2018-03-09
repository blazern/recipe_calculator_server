use futures;
use reqwest::Client;
use std::thread;
use std::io::Read;
use std::sync::{Arc, Mutex};

use server::entry_point;

const SERVER_ADDRESS: &str = "127.0.0.1:3000";

// Future for stopping server started in 'entry_point::start_server'.
struct TerminateServerFuture {
    should_finish: Arc<Mutex<bool>>,
    interested_tasks: Arc<Mutex<Vec<futures::task::Task>>>,
}

impl futures::Future for TerminateServerFuture {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        self.interested_tasks.lock().unwrap().push(futures::task::current());

        let should_finish = self.should_finish.lock().unwrap();
        if *should_finish == true {
            return Ok(futures::Async::Ready(()));
        } else {
            return Ok(futures::Async::NotReady);
        }
    }
}

// Wrapper for server started by 'entry_point::start_server'.
// Starts server on background thread (so that tests would be blocked by the server), stops
// server in its destructor.
struct ServerWrapper {
    should_finish: Arc<Mutex<bool>>,
    interested_tasks: Arc<Mutex<Vec<futures::task::Task>>>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl ServerWrapper {
    fn new() -> ServerWrapper {
        let mut result = ServerWrapper {
            should_finish: Arc::new(Mutex::new(false)),
            interested_tasks: Arc::new(Mutex::new(Vec::new())),
            join_handle: None,
        };

        let future = TerminateServerFuture {
            should_finish: result.should_finish.clone(),
            interested_tasks: result.interested_tasks.clone(),
        };

        let join_handle = thread::spawn(|| {
            let address = SERVER_ADDRESS.parse().unwrap();
            entry_point::start_server(&address, future);
        });
        result.join_handle = Some(join_handle);

        return result;
    }
}

impl Drop for ServerWrapper {
    fn drop(&mut self) {
        // Hiding mutexes usage to a scope so that ServerFuture::poll wouldn't deadlock.
        {
            let mut should_finish = self.should_finish.lock().unwrap();
            *should_finish = true;

            let interested_tasks = self.interested_tasks.lock().unwrap();
            for task in &*interested_tasks {
                task.notify();
            }
        }

        let join_handle = self.join_handle.take();
        join_handle.unwrap().join().unwrap();
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