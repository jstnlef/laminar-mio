use std::{
    collections::HashMap,
    fmt,
    net::SocketAddr,
    time::{Duration, Instant}
};

pub struct ActiveConnections {
    connections: HashMap<SocketAddr, VirtualConnection>
}

impl ActiveConnections {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new()
        }
    }
}


/// Contains the information about 'virtual connections' over UDP.
pub struct VirtualConnection {
    /// Last time we received a packet from this client
    last_packet_time: Instant,
    /// The address of the remote endpoint
    remote_address: SocketAddr,
}

impl VirtualConnection {
    pub fn new(remote_address: SocketAddr) -> Self {
        Self {
            last_packet_time: Instant::now(),
            remote_address
        }
    }

    /// Represents the duration since we last received a packet from this client
    pub fn time_since_last_packet(&self) -> Duration {
        let now = Instant::now();
        now.duration_since(self.last_packet_time)
    }

    /// The remote address of the client
    pub fn remote_address(&self) -> SocketAddr {
        self.remote_address
    }
}

impl fmt::Debug for VirtualConnection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}",
            self.remote_address.ip(),
            self.remote_address.port()
        )
    }
}
