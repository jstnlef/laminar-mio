use super::{calc_header_size, HeaderReader, HeaderWriter};
use crate::{net::DeliveryMethod, packet::PacketTypeId, protocol_version::ProtocolVersion};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use lazy_static::lazy_static;
use std::io;

lazy_static! {
    pub static ref HEADER_SIZE: u8 = calc_header_size::<StandardHeader>();
}

/// This header will be included in each packet, and contains some basic information.
#[derive(Copy, Clone, Debug)]
pub struct StandardHeader {
    /// crc32 of the protocol version.
    protocol_version: u32,
    /// specifies the packet type.
    packet_type: PacketTypeId,
    /// specifies how this packet should be processed.
    delivery_method: DeliveryMethod,
}

impl StandardHeader {
    /// Create new standard header.
    pub fn new(delivery_method: DeliveryMethod, packet_type: PacketTypeId) -> Self {
        StandardHeader {
            protocol_version: ProtocolVersion::get_crc32(),
            packet_type,
            delivery_method,
        }
    }

    pub fn protocol_version(&self) -> u32 {
        self.protocol_version
    }

    pub fn packet_type(&self) -> PacketTypeId {
        self.packet_type
    }
}

impl Default for StandardHeader {
    fn default() -> Self {
        StandardHeader::new(DeliveryMethod::UnreliableUnordered, PacketTypeId::Packet)
    }
}

impl HeaderWriter for StandardHeader {
    fn write(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        buffer.write_u32::<BigEndian>(self.protocol_version)?;
        buffer.write_u8(PacketTypeId::get_id(self.packet_type))?;
        buffer.write_u8(DeliveryMethod::get_delivery_method_id(self.delivery_method))?;

        Ok(())
    }
}

impl HeaderReader for StandardHeader {
    type Header = io::Result<StandardHeader>;

    fn read(rdr: &mut io::Cursor<&[u8]>) -> Self::Header {
        let protocol_version = rdr.read_u32::<BigEndian>()?;
        let packet_id = rdr.read_u8()?;
        let delivery_method_id = rdr.read_u8()?;

        let header = Self {
            protocol_version,
            packet_type: PacketTypeId::get_packet_type(packet_id),
            delivery_method: DeliveryMethod::get_delivery_method_from_id(delivery_method_id),
        };

        Ok(header)
    }

    /// Get the size of this header.
    fn size(&self) -> u8 {
        *HEADER_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::{HeaderReader, HeaderWriter, StandardHeader};
    use crate::net::DeliveryMethod;
    use crate::packet::PacketTypeId;
    use crate::protocol_version::ProtocolVersion;
    use std::io::Cursor;

    #[test]
    pub fn serializes_deserialize_packet_header_test() {
        let packet_header = StandardHeader::default();
        let mut buffer = Vec::with_capacity((packet_header.size() + 1) as usize);

        let _ = packet_header.write(&mut buffer);

        let mut cursor = Cursor::new(buffer.as_slice());
        let packet_header = StandardHeader::read(&mut cursor).unwrap();
        assert!(ProtocolVersion::valid_version(
            packet_header.protocol_version
        ));
        assert_eq!(packet_header.packet_type, PacketTypeId::Packet);
        assert_eq!(
            packet_header.delivery_method,
            DeliveryMethod::UnreliableUnordered
        );
    }

    #[test]
    pub fn header_size_test() {
        assert_eq!(StandardHeader::default().size(), 6);
    }
}
