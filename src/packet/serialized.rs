use std::net::SocketAddr;

/// Wrapper struct to hold the fully serialized packet (includes header data)
pub struct SerializedPacket {
    address: SocketAddr,
    payload: Vec<u8>,
}

impl SerializedPacket {
    pub fn new(address: SocketAddr) -> Self {
        Self {
            address,
            payload: vec![65, 66, 67],
        }
    }

    /// Get the endpoint from this packet.
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// Returns an iterator yielding payload fragments
    pub fn fragments(&self, fragment_size: u16) -> impl Iterator<Item = &[u8]> {
        /// TODO: This needs to be implemented to yield out fragments based on size
        self.payload.chunks(fragment_size as usize)
    }
}
