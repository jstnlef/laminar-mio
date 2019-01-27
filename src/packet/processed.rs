use crate::{
    net::DeliveryMethod,
    packet::headers::{
        HeaderReader, HeaderWriter, ReliableHeader, StandardHeader, RELIABLE_HEADER_SIZE,
        STANDARD_HEADER_SIZE,
    },
    packet::{Packet, PacketTypeId},
};
use std::net::SocketAddr;

/// Wrapper struct to hold the fully serialized packet (includes header data)
pub struct ProcessedPacket {
    packet: Packet,
    reliability: Option<ReliableHeader>,
    serialized: Vec<u8>,
}

impl ProcessedPacket {
    pub fn new(packet: Packet, reliability: Option<ReliableHeader>) -> Self {
        let size = *STANDARD_HEADER_SIZE + *RELIABLE_HEADER_SIZE + packet.payload.len();
        Self {
            packet,
            reliability,
            serialized: Vec::with_capacity(size),
        }
    }

    /// Get the endpoint from this packet.
    pub fn address(&self) -> SocketAddr {
        self.packet.address
    }

    /// Returns an iterator yielding payload fragments
    pub fn fragments(&mut self, fragment_size: u16) -> impl Iterator<Item = &[u8]> {
        let standard_header =
            StandardHeader::new(self.packet.delivery_method, PacketTypeId::Packet);
        standard_header.write(&mut self.serialized);

        if let Some(reliability_header) = self.reliability {
            reliability_header.write(&mut self.serialized);
        }

        self.serialized.extend(self.packet.payload.iter());

        self.serialized.chunks(self.serialized.len())
    }
}
