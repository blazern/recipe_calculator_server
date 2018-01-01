use config::Config;
use db::core::connection::DBConnection;
use db::core::error::Error as DBCoreError;
use super::error::Error;

// Primitive connection pool for connections reusing.

pub enum ConnectionType {
    UserConnection,
    ServerConnection
}

pub struct ConnectionPool {
    connection_type: ConnectionType,
    config: Config,
    connections: Vec<DBConnection>,
}

impl ConnectionPool {
    pub fn new(connection_type: ConnectionType, config: Config) -> Self {
        ConnectionPool { connection_type, config, connections: Vec::new() }
    }

    pub fn for_client_user(config: Config) -> Self {
        return ConnectionPool {
            connection_type: ConnectionType::UserConnection,
            config,
            connections: Vec::new()
        };
    }

    pub fn for_server_user(config: Config) -> Self {
        return ConnectionPool {
            connection_type: ConnectionType::ServerConnection,
            config,
            connections: Vec::new()
        };
    }

    pub fn borrow(&mut self) -> Result<DBConnection, Error> {
        match self.connections.pop() {
            Some(connection) => {
                return Ok(connection);
            }
            None => {
                return Ok(self.new_connection()?);
            }
        }
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

    pub fn put_back(&mut self, connection: DBConnection) {
        self.connections.push(connection);
    }

    pub fn pooled_connections_count(&self) -> usize {
        return self.connections.len();
    }
}

#[cfg(test)]
#[path = "./connection_pool_test.rs"]
mod connection_pool_test;