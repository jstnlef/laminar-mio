use super::{calc_header_size, HeaderReader, HeaderWriter};
use crate::{net::DeliveryMethod, packet::PacketType, protocol_version};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use lazy_static::lazy_static;
use std::io;

lazy_static! {
    pub static ref HEADER_SIZE: usize = calc_header_size::<StandardHeader>();
}

/// This header will be included in each packet, and contains some basic information.
#[derive(Copy, Clone, Debug)]
pub struct StandardHeader {
    /// crc32 of the protocol version.
    protocol_version: u32,
    /// specifies the packet type.
    packet_type: PacketType,
    /// specifies how this packet should be processed.
    delivery_method: DeliveryMethod,
    /// This is the sequence number. This is used for both reliability and fragmentation
    sequence_num: u16,
}

impl StandardHeader {
    /// Create new standard header.
    pub fn new(
        delivery_method: DeliveryMethod,
        packet_type: PacketType,
        sequence_num: u16,
    ) -> Self {
        StandardHeader {
            protocol_version: protocol_version::get_crc32(),
            packet_type,
            delivery_method,
            sequence_num,
        }
    }

    #[inline]
    pub fn protocol_version(&self) -> u32 {
        self.protocol_version
    }

    #[inline]
    pub fn packet_type(&self) -> PacketType {
        self.packet_type
    }

    #[inline]
    pub fn delivery_method(&self) -> DeliveryMethod {
        self.delivery_method
    }

    #[inline]
    pub fn sequence_num(&self) -> u16 {
        self.sequence_num
    }
}

impl Default for StandardHeader {
    fn default() -> Self {
        StandardHeader::new(DeliveryMethod::UnreliableUnordered, PacketType::Packet, 0)
    }
}

impl HeaderWriter for StandardHeader {
    fn write(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        buffer.write_u32::<BigEndian>(self.protocol_version)?;
        buffer.write_u8(PacketType::get_id(self.packet_type))?;
        buffer.write_u8(DeliveryMethod::get_delivery_method_id(self.delivery_method))?;
        buffer.write_u16::<BigEndian>(self.sequence_num)?;
        Ok(())
    }
}

impl HeaderReader for StandardHeader {
    type Header = io::Result<Self>;

    fn read(rdr: &mut io::Cursor<&[u8]>) -> Self::Header {
        let protocol_version = rdr.read_u32::<BigEndian>()?;
        let packet_id = rdr.read_u8()?;
        let delivery_method_id = rdr.read_u8()?;
        let sequence_num = rdr.read_u16::<BigEndian>()?;

        let header = Self {
            protocol_version,
            packet_type: PacketType::get_packet_type(packet_id),
            delivery_method: DeliveryMethod::get_delivery_method_from_id(delivery_method_id),
            sequence_num,
        };

        Ok(header)
    }

    /// Get the size of this header.
    fn size(&self) -> usize {
        *HEADER_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::{HeaderReader, HeaderWriter, StandardHeader};
    use crate::net::DeliveryMethod;
    use crate::packet::PacketType;
    use crate::protocol_version;
    use std::io::Cursor;

    #[test]
    pub fn serializes_deserialize_packet_header_test() {
        let packet_header = StandardHeader::default();
        let mut buffer = Vec::with_capacity(packet_header.size() + 1);

        let _ = packet_header.write(&mut buffer);

        let mut cursor = Cursor::new(buffer.as_slice());
        let packet_header = StandardHeader::read(&mut cursor).unwrap();
        assert!(protocol_version::valid_version(
            packet_header.protocol_version
        ));
        assert_eq!(packet_header.packet_type, PacketType::Packet);
        assert_eq!(
            packet_header.delivery_method,
            DeliveryMethod::UnreliableUnordered
        );
    }

    #[test]
    pub fn header_size_test() {
        assert_eq!(StandardHeader::default().size(), 8);
    }
}
