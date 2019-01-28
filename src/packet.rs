/// Contains code dealing with Packet headers
pub mod headers;
mod packet_type;
mod processed;

pub use self::packet_type::PacketTypeId;
pub use self::processed::ProcessedPacket;

use crate::net::DeliveryMethod;
use std::net::SocketAddr;

#[derive(Clone, PartialEq, Eq, Debug)]
/// This is a user friendly packet containing the payload and the endpoint from
/// where it came or where to send it to.
pub struct Packet {
    /// the endpoint from where it came or where to send to.
    address: SocketAddr,
    /// the raw payload of the packet.
    payload: Box<[u8]>,
    /// defines on how the packet will be delivered.
    delivery_method: DeliveryMethod,
}

impl Packet {
    /// Unreliable. Packets can be dropped, duplicated or arrive without order.
    ///
    /// **Details**
    ///
    /// | Packet Drop     | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
    /// | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
    /// |       Yes       |        Yes         |      No          |      No              |       No        |
    ///
    /// Basically just bare UDP, free to be dropped, used for very unnecessary data, great for 'general' position updates.
    pub fn unreliable(address: SocketAddr, payload: Vec<u8>) -> Packet {
        Packet::new(
            address,
            payload.into_boxed_slice(),
            DeliveryMethod::UnreliableUnordered,
        )
    }

    /// Reliable. All packets will be sent and received, but without order.
    ///
    /// *Details*
    ///
    /// |   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
    /// | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
    /// |       No        |      No            |      No          |      Yes             |       Yes       |
    ///
    /// Basically this is almost TCP like without ordering of packets.
    /// Receive every packet and immediately give to application, order does not matter.
    pub fn reliable_unordered(address: SocketAddr, payload: Vec<u8>) -> Packet {
        Packet::new(
            address,
            payload.into_boxed_slice(),
            DeliveryMethod::ReliableUnordered,
        )
    }

    /// Create an new packet by passing the receiver, data and how this packet should be delivered.
    pub(crate) fn new(
        address: SocketAddr,
        payload: Box<[u8]>,
        delivery_method: DeliveryMethod,
    ) -> Self {
        Packet {
            address,
            payload,
            delivery_method,
        }
    }

    /// Get the payload (raw data) of this packet.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Get the endpoint from this packet.
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// Get the type representing on how this packet will be delivered.
    pub fn delivery_method(&self) -> DeliveryMethod {
        self.delivery_method
    }
}
