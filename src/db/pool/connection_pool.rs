use config::Config;
use db::core::connection::DBConnection;
use db::core::error::Error as DBCoreError;
use super::error::Error;

use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex;

// Thread-safe connection pool for connections reusing.

type WrappedPool = Arc<Mutex<Vec<DBConnection>>>;

pub enum ConnectionType {
    UserConnection,
    ServerConnection
}

pub struct ConnectionPool {
    connection_type: ConnectionType,
    config: Config,
    connections: WrappedPool,
}

pub struct BorrowedDBConnection {
    connection: Option<DBConnection>,
    connections: WrappedPool,
}

impl ConnectionPool {
    pub fn new(connection_type: ConnectionType, config: Config) -> Self {
        ConnectionPool {
            connection_type,
            config,
            connections: Arc::new(Mutex::new(Vec::new()))
        }
    }

    pub fn for_client_user(config: Config) -> Self {
        return ConnectionPool {
            connection_type: ConnectionType::UserConnection,
            config,
            connections: Arc::new(Mutex::new(Vec::new()))
        };
    }

    pub fn for_server_user(config: Config) -> Self {
        return ConnectionPool {
            connection_type: ConnectionType::ServerConnection,
            config,
            connections: Arc::new(Mutex::new(Vec::new()))
        };
    }

    pub fn borrow(&mut self) -> Result<BorrowedDBConnection, Error> {
        let connection: Option<DBConnection>;
        {
            let mut connections = self.connections.lock().unwrap();
            connection = (&mut connections).pop();
        }
        match connection {
            Some(connection) => {
                return Ok(self.connection_to_borrowed(connection));
            }
            None => {
                let new_connection = self.new_connection()?;
                return Ok(self.connection_to_borrowed(new_connection));
            }
        }
    }

    fn connection_to_borrowed(&mut self, connection: DBConnection) -> BorrowedDBConnection {
        let result = BorrowedDBConnection {
            connection: Some(connection),
            connections: self.connections.clone()
        };
        return result;
    }

    fn new_connection(&self) -> Result<DBConnection, DBCoreError> {
        match self.connection_type {
            ConnectionType::UserConnection => {
                return DBConnection::for_client_user(&self.config);
            }
            ConnectionType::ServerConnection => {
                return DBConnection::for_server_user(&self.config);
            }
        }
    }

    pub fn pooled_connections_count(&self) -> usize {
        let connections = self.connections.lock().unwrap();
        return (&connections).len();
    }
}

impl Drop for BorrowedDBConnection {
    fn drop(&mut self) {
        let mut connections = self.connections.lock().unwrap();
        (&mut connections).push(self.connection.take().expect(
            "Connection expected to be moved out only in drop"));
    }
}

impl Deref for BorrowedDBConnection {
    type Target = DBConnection;

    fn deref(&self) -> &DBConnection {
        &self.connection.as_ref().expect("Connection expected to be moved out only in drop")
    }
}

#[cfg(test)]
#[path = "./connection_pool_test.rs"]
mod connection_pool_test;