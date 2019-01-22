use crate::{
    net::DeliveryMethod,
    packet::headers::{HeaderReader, HeaderWriter, ReliableHeader, StandardHeader},
    packet::{Packet, PacketTypeId},
};
use std::net::SocketAddr;

/// Wrapper struct to hold the fully serialized packet (includes header data)
pub struct ProcessedPacket {
    packet: Packet,
    reliability: Option<ReliableHeader>,
}

impl ProcessedPacket {
    pub fn new(packet: Packet, reliability: Option<ReliableHeader>) -> Self {
        Self {
            packet,
            reliability,
        }
    }

    /// Get the endpoint from this packet.
    pub fn address(&self) -> SocketAddr {
        self.packet.address
    }

    /// Returns an iterator yielding payload fragments
    pub fn fragments(&self, fragment_size: u16) -> impl Iterator<Item = &[u8]> {
        //        let standard_header = StandardHeader::new(self.packet.delivery_method, PacketTypeId::Packet);
        //        let mut buffer = Vec::with_capacity(standard_header.size() + self.packet.payload.len());
        //        standard_header.write(&mut buffer);
        //        buffer.extend(&self.packet.payload);

        // TODO: This needs to be implemented to yield out fragments based on size
        self.packet.payload.chunks(fragment_size as usize)
    }
}

//struct FragmentIterator<'a> {
//
//}

//impl FragmentIterator {
//    pub fn new() -> Self {
//
//    }
//}

//impl<'a> Iterator for FragmentIterator<'a> {
//    type Item = &'a [u8];
//
//    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
//        unimplemented!()
//    }
//}
