mod quality;
mod virtual_connection;

pub use self::quality::{NetworkQuality, RttMeasurer};
pub use self::virtual_connection::VirtualConnection;

use crate::config::SocketConfig;
use std::{collections::HashMap, net::SocketAddr, time::Duration};

/// Maintains a registry of active "connections". Essentially, when we receive a packet on the
/// socket from a particular `SocketAddr`, we will track information about it here.
pub struct ActiveConnections {
    connections: HashMap<SocketAddr, VirtualConnection>,
}

impl ActiveConnections {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    /// Try to get a VirtualConnection by address. If the connection does not exist, it will be
    /// inserted and returned.
    pub fn get_or_insert_connection(
        &mut self,
        address: &SocketAddr,
        config: &SocketConfig,
    ) -> &mut VirtualConnection {
        if !self.connections.contains_key(address) {
            self.connections
                .insert(*address, VirtualConnection::new(*address, config));
        }
        self.connections
            .get_mut(address)
            .expect("We just added this key. It should definitely exist.")
    }

    /// Removes the connection from ActiveConnections by socket address.
    pub fn remove_connection(
        &mut self,
        address: &SocketAddr,
    ) -> Option<(SocketAddr, VirtualConnection)> {
        self.connections.remove_entry(address)
    }

    /// Check for and return VirtualConnections which have been idling longer than `max_idle_time`.
    pub fn idle_connections(&mut self, max_idle_time: Duration) -> Vec<SocketAddr> {
        self.connections
            .iter()
            .filter(|(_, connection)| connection.time_since_last_packet() >= max_idle_time)
            .map(|(address, _)| address.clone())
            .collect()
    }

    /// Get the number of connected clients.
    pub fn count(&self) -> usize {
        self.connections.len()
    }
}

#[cfg(test)]
mod tests {
    use super::{ActiveConnections, SocketConfig};
    use std::{thread, time::Duration};

    const ADDRESS: &str = "127.0.0.1:12345";

    #[test]
    fn connection_timed_out() {
        let mut connections = ActiveConnections::new();
        let config = SocketConfig::default();

        // add 10 clients
        for i in 0..10 {
            connections.get_or_insert_connection(
                &(format!("127.0.0.1:123{}", i).parse().unwrap()),
                &config,
            );
        }

        assert_eq!(connections.count(), 10);

        // Sleep a little longer than the polling interval.
        thread::sleep(Duration::from_millis(400));

        let timed_out_connections = connections.idle_connections(Duration::from_millis(200));

        assert_eq!(timed_out_connections.len(), 10);
    }

    #[test]
    fn insert_connection() {
        let mut connections = ActiveConnections::new();
        let config = SocketConfig::default();

        let address = &("127.0.0.1:12345".parse().unwrap());
        connections.get_or_insert_connection(address, &config);
        assert!(connections.connections.contains_key(address));
    }

    #[test]
    fn insert_existing_connection() {
        let mut connections = ActiveConnections::new();
        let config = SocketConfig::default();

        let address = &("127.0.0.1:12345".parse().unwrap());
        connections.get_or_insert_connection(address, &config);
        assert!(connections.connections.contains_key(address));
        connections.get_or_insert_connection(address, &config);
        assert!(connections.connections.contains_key(address));
    }

    #[test]
    fn remove_connection() {
        let mut connections = ActiveConnections::new();
        let config = SocketConfig::default();

        let address = &("127.0.0.1:12345".parse().unwrap());
        connections.get_or_insert_connection(address, &config);
        assert!(connections.connections.contains_key(address));
        connections.remove_connection(address);
        assert!(!connections.connections.contains_key(address));
    }

    #[test]
    fn remove_non_existing_connection() {
        let mut connections = ActiveConnections::new();

        let address = &("127.0.0.1:12345".parse().unwrap());
        connections.remove_connection(address);
        assert!(!connections.connections.contains_key(address));
    }
}
