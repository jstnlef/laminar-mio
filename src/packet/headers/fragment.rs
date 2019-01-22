use super::{calc_header_size, AckedPacketHeader, HeaderReader, HeaderWriter, StandardHeader};
use crate::errors::FragmentError;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use lazy_static::lazy_static;
use std::io;

lazy_static! {
    static ref HEADER_SIZE: usize = calc_header_size::<FragmentHeader>();
}

/// This header represents a fragmented packet header.
#[derive(Copy, Clone, Debug)]
pub struct FragmentHeader {
    standard_header: StandardHeader,
    sequence_num: u16,
    id: u8,
    num_fragments: u8,
    packet_header: Option<AckedPacketHeader>,
}

impl FragmentHeader {
    /// Create new fragment with the given packet header
    pub fn new(
        standard_header: StandardHeader,
        id: u8,
        num_fragments: u8,
        packet_header: AckedPacketHeader,
    ) -> Self {
        FragmentHeader {
            standard_header,
            id,
            num_fragments,
            packet_header: Some(packet_header),
            sequence_num: packet_header.sequence_num(),
        }
    }

    /// Get the id of this fragment.
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Get the sequence number from this packet.
    pub fn sequence_num(&self) -> u16 {
        self.sequence_num
    }

    /// Get the total number of fragments in the packet this fragment is part of.
    pub fn fragment_count(&self) -> u8 {
        self.num_fragments
    }

    /// Get the packet header if attached to fragment.
    pub fn packet_header(&self) -> Option<AckedPacketHeader> {
        self.packet_header
    }
}

impl Default for FragmentHeader {
    fn default() -> Self {
        Self {
            standard_header: StandardHeader::default(),
            sequence_num: 0,
            id: 0,
            num_fragments: 0,
            packet_header: None,
        }
    }
}

impl HeaderWriter for FragmentHeader {
    fn write(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        self.standard_header.write(buffer)?;
        buffer.write_u16::<BigEndian>(self.sequence_num)?;
        buffer.write_u8(self.id)?;
        buffer.write_u8(self.num_fragments)?;

        // append acked header only first time
        if self.id == 0 {
            match self.packet_header {
                Some(header) => {
                    header.write(buffer)?;
                }
                None => {
                    return Err(FragmentError::PacketHeaderNotFound.into());
                }
            }
        }

        Ok(())
    }
}

impl HeaderReader for FragmentHeader {
    type Header = io::Result<Self>;

    fn read(rdr: &mut io::Cursor<&[u8]>) -> Self::Header {
        let standard_header = StandardHeader::read(rdr)?;
        let sequence_num = rdr.read_u16::<BigEndian>()?;
        let id = rdr.read_u8()?;
        let num_fragments = rdr.read_u8()?;

        let mut header = Self {
            standard_header,
            sequence_num,
            id,
            num_fragments,
            packet_header: None,
        };

        // append acked header is only appended to first packet.
        if id == 0 {
            header.packet_header = Some(AckedPacketHeader::read(rdr)?);
        }

        Ok(header)
    }

    /// Get the size of this header.
    fn size(&self) -> usize {
        if self.id == 0 {
            match self.packet_header {
                Some(header) => header.size() + *HEADER_SIZE,
                None => {
                    panic!("Attempting to retrieve size on a 0 ID packet with no packet header");
                }
            }
        } else {
            *HEADER_SIZE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AckedPacketHeader, FragmentHeader, HeaderReader, HeaderWriter, StandardHeader};
    use crate::net::DeliveryMethod;
    use crate::packet::PacketTypeId;
    use std::io::Cursor;

    #[test]
    pub fn serializes_deserialize_fragment_header_test() {
        // create default header
        let standard_header =
            StandardHeader::new(DeliveryMethod::UnreliableUnordered, PacketTypeId::Fragment);

        let packet_header = AckedPacketHeader::new(standard_header.clone(), 1, 1, 5421);

        // create fragment header with the default header and acked header.
        let fragment = FragmentHeader::new(standard_header.clone(), 0, 1, packet_header.clone());
        let mut fragment_buffer = Vec::with_capacity(fragment.size() + 1);
        fragment.write(&mut fragment_buffer).unwrap();

        let mut cursor: Cursor<&[u8]> = Cursor::new(fragment_buffer.as_slice());
        let fragment_deserialized = FragmentHeader::read(&mut cursor).unwrap();

        assert_eq!(fragment_deserialized.id, 0);
        assert_eq!(fragment_deserialized.num_fragments, 1);
        assert_eq!(fragment_deserialized.sequence_num, 1);

        assert!(fragment_deserialized.packet_header.is_some());

        let fragment_packet_header = fragment_deserialized.packet_header.unwrap();
        assert_eq!(fragment_packet_header.sequence_num(), 1);
        assert_eq!(fragment_packet_header.last_acked(), 1);
        assert_eq!(fragment_packet_header.ack_field(), 5421);
    }

    #[test]
    pub fn header_size_test() {
        // Test first fragment
        let fragment = FragmentHeader::new(
            StandardHeader::default(),
            0,
            0,
            AckedPacketHeader::default(),
        );
        assert_eq!(fragment.size(), 24);

        // Test subsequent fragment
        let fragment = FragmentHeader {
            standard_header: StandardHeader::default(),
            sequence_num: 0,
            id: 1,
            num_fragments: 0,
            packet_header: None,
        };
        assert_eq!(fragment.size(), 10);
    }
}
