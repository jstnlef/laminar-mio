use std::net::SocketAddr;

pub struct Packet {
    address: SocketAddr,
    payload: Box<[u8]>,
}

impl Packet {
    pub fn new(address: SocketAddr, payload: Box<[u8]>) -> Self {
        Self {
            address,
            payload
        }
    }

    /// This is the address where the packet should be sent to or was received from.
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// Get the payload (raw data) of this packet.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}
