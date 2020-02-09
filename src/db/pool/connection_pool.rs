use super::error::Error;
use crate::config::Config;
use crate::db::core::connection::DBConnection;
use crate::db::core::connection::DBConnectionImpl;
use crate::db::core::connection::UnderlyingConnectionSource;
use crate::db::core::error::Error as DBCoreError;

use std::sync::Arc;
use std::sync::Mutex;

// Thread-safe connection pool for connections reusing.

pub enum ConnectionType {
    UserConnection,
    ServerConnection,
}

/// Objects of the type are Cloneable, because they implement the PIMPL pattern -
/// a cloned pool defacto is the same pool as the one it was cloned from.
#[derive(Clone)]
pub struct ConnectionPool {
    pimpl: Arc<Mutex<ConnectionPoolImpl>>,
}

pub struct BorrowedDBConnection {
    connection: Option<DBConnectionImpl>,
    pimpl: Option<Arc<Mutex<ConnectionPoolImpl>>>,
}

struct ConnectionPoolImpl {
    connection_type: ConnectionType,
    config: Config,
    connections: Vec<DBConnectionImpl>,
}

impl ConnectionPool {
    pub fn new(connection_type: ConnectionType, config: Config) -> Self {
        let pimpl = ConnectionPoolImpl::new(connection_type, config);
        ConnectionPool {
            pimpl: Arc::new(Mutex::new(pimpl)),
        }
    }

    pub fn for_client_user(config: Config) -> Self {
        Self::new(ConnectionType::UserConnection, config)
    }

    pub fn for_server_user(config: Config) -> Self {
        Self::new(ConnectionType::ServerConnection, config)
    }

    pub fn borrow_connection(&mut self) -> Result<BorrowedDBConnection, Error> {
        let result = self
            .pimpl
            .lock()
            .expect("expecting ok mutex")
            .borrow_connection();
        if let Ok(mut result) = result {
            result.pimpl = Some(self.pimpl.clone());
            Ok(result)
        } else {
            result
        }
    }

    pub fn pooled_connections_count(&self) -> usize {
        self.pimpl
            .lock()
            .expect("expecting ok mutex")
            .pooled_connections_count()
    }
}

impl BorrowedDBConnection {
    pub fn try_clone(&self) -> Result<Self, Error> {
        let pimpl = self
            .pimpl
            .as_ref()
            .expect("Pimpl is expected to be always present");
        let mut pimpl = pimpl.lock().expect("Expecting ok mutex");
        let mut result = pimpl.borrow_connection()?;
        result.pimpl = self.pimpl.clone();
        Ok(result)
    }
}

impl ConnectionPoolImpl {
    fn new(connection_type: ConnectionType, config: Config) -> Self {
        ConnectionPoolImpl {
            connection_type,
            config,
            connections: Vec::new(),
        }
    }

    fn borrow_connection(&mut self) -> Result<BorrowedDBConnection, Error> {
        let connection: Option<DBConnectionImpl>;
        {
            connection = self.connections.pop();
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
            pimpl: None, // We'll init it later
        }
    }

    fn new_connection(&self) -> Result<DBConnectionImpl, DBCoreError> {
        match self.connection_type {
            ConnectionType::UserConnection => DBConnectionImpl::for_client_user(&self.config),
            ConnectionType::ServerConnection => DBConnectionImpl::for_server_user(&self.config),
        }
    }

    fn pooled_connections_count(&self) -> usize {
        self.connections.len()
    }

    fn return_borrowed_connection(&mut self, connection: DBConnectionImpl) {
        self.connections.push(connection)
    }
}

impl Drop for BorrowedDBConnection {
    fn drop(&mut self) {
        let connection = self
            .connection
            .take()
            .expect("Connection expected to be moved out only in drop");
        let pimpl = self
            .pimpl
            .as_ref()
            .expect("Pimpl is expected to be always present");
        let mut pimpl = pimpl.lock().expect("Expecting ok mutex");
        pimpl.return_borrowed_connection(connection);
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
