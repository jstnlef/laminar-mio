use crate::{
    net::DeliveryMethod,
    packet::headers::{HeaderReader, HeaderWriter, StandardHeader},
    packet::PacketTypeId,
};
use std::net::SocketAddr;

/// Wrapper struct to hold the fully serialized packet (includes header data)
pub struct ProcessedPacket {
    address: SocketAddr,
    delivery_method: DeliveryMethod,
    payload: Vec<u8>,
}

impl ProcessedPacket {
    pub fn new(address: SocketAddr, delivery_method: DeliveryMethod, payload: &[u8]) -> Self {
        Self {
            address,
            delivery_method,
            payload: payload.to_owned(),
        }
    }

    /// Get the endpoint from this packet.
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// Returns an iterator yielding payload fragments
    pub fn fragments(&self, fragment_size: u16) -> impl Iterator<Item = &[u8]> {
        let standard_header = StandardHeader::new(self.delivery_method, PacketTypeId::Packet);
        let mut buffer = Vec::with_capacity(standard_header.size() + self.payload.len());
        standard_header.write(&mut buffer);
        buffer.extend(&self.payload);

        // TODO: This needs to be implemented to yield out fragments based on size
        self.payload.chunks(fragment_size as usize)
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
