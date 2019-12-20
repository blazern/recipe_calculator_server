use super::error::Error;
use config::Config;
use db::core::connection::DBConnection;
use db::core::connection::DBConnectionImpl;
use db::core::connection::UnderlyingConnectionSource;
use db::core::error::Error as DBCoreError;

use std::sync::Arc;
use std::sync::Mutex;

// Thread-safe connection pool for connections reusing.

type WrappedPool = Arc<Mutex<Vec<DBConnectionImpl>>>;

pub enum ConnectionType {
    UserConnection,
    ServerConnection,
}

pub struct ConnectionPool {
    connection_type: ConnectionType,
    config: Config,
    connections: WrappedPool,
}

pub struct BorrowedDBConnection {
    connection: Option<DBConnectionImpl>,
    connections: WrappedPool,
}

impl ConnectionPool {
    pub fn new(connection_type: ConnectionType, config: Config) -> Self {
        ConnectionPool {
            connection_type,
            config,
            connections: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn for_client_user(config: Config) -> Self {
        ConnectionPool {
            connection_type: ConnectionType::UserConnection,
            config,
            connections: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn for_server_user(config: Config) -> Self {
        ConnectionPool {
            connection_type: ConnectionType::ServerConnection,
            config,
            connections: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn borrow(&mut self) -> Result<BorrowedDBConnection, Error> {
        let connection: Option<DBConnectionImpl>;
        {
            let mut connections = self.connections.lock().unwrap();
            connection = (&mut connections).pop();
        }
        match connection {
            Some(connection) => Ok(self.connection_to_borrowed(connection)),
            None => {
                let new_connection = self.new_connection()?;
                Ok(self.connection_to_borrowed(new_connection))
            }
        }
    }

    fn connection_to_borrowed(&mut self, connection: DBConnectionImpl) -> BorrowedDBConnection {
        BorrowedDBConnection {
            connection: Some(connection),
            connections: self.connections.clone(),
        }
    }

    fn new_connection(&self) -> Result<DBConnectionImpl, DBCoreError> {
        match self.connection_type {
            ConnectionType::UserConnection => DBConnectionImpl::for_client_user(&self.config),
            ConnectionType::ServerConnection => DBConnectionImpl::for_server_user(&self.config),
        }
    }

    pub fn pooled_connections_count(&self) -> usize {
        let connections = self.connections.lock().unwrap();
        (&connections).len()
    }
}

impl Drop for BorrowedDBConnection {
    fn drop(&mut self) {
        let mut connections = self.connections.lock().unwrap();
        (&mut connections).push(
            self.connection
                .take()
                .expect("Connection expected to be moved out only in drop"),
        );
    }
}

impl DBConnection for BorrowedDBConnection {
    fn underlying_connection_source(&self) -> &UnderlyingConnectionSource {
        &self
            .connection
            .as_ref()
            .expect("Connection expected to be moved out only in drop")
            .underlying_connection_source()
    }
}

#[cfg(test)]
#[path = "./connection_pool_test.rs"]
mod connection_pool_test;
