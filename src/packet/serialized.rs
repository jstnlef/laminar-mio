use crate::packet::headers::HeaderWriter;
use std::net::SocketAddr;

/// Wrapper struct to hold the fully serialized packet (includes header data)
pub struct ProcessedPacket {
    address: SocketAddr,
    payload: Vec<u8>,
}

impl ProcessedPacket {
    pub fn new(address: SocketAddr, payload: Vec<u8>) -> Self {
        Self { address, payload }
    }

    /// Get the endpoint from this packet.
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// Returns an iterator yielding payload fragments
    pub fn fragments(&self, fragment_size: u16) -> impl Iterator<Item = &[u8]> {
        // TODO: This needs to be implemented to yield out fragments based on size
        self.payload.chunks(fragment_size as usize)
    }
}
