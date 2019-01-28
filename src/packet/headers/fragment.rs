use super::{calc_header_size, HeaderReader, HeaderWriter};
use byteorder::{ReadBytesExt, WriteBytesExt};
use lazy_static::lazy_static;
use std::io;

lazy_static! {
    static ref HEADER_SIZE: usize = calc_header_size::<FragmentHeader>();
}

/// This header represents a fragmented packet header.
#[derive(Copy, Clone, Debug)]
pub struct FragmentHeader {
    id: u8,
    num_fragments: u8,
}

impl FragmentHeader {
    /// Create new fragment with the given packet header
    pub fn new(id: u8, num_fragments: u8) -> Self {
        FragmentHeader { id, num_fragments }
    }

    /// Get the id of this fragment.
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Get the total number of fragments in the packet this fragment is part of.
    pub fn fragment_count(&self) -> u8 {
        self.num_fragments
    }
}

impl Default for FragmentHeader {
    fn default() -> Self {
        Self {
            id: 0,
            num_fragments: 0,
        }
    }
}

impl HeaderWriter for FragmentHeader {
    fn write(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        buffer.write_u8(self.id)?;
        buffer.write_u8(self.num_fragments)?;
        Ok(())
    }
}

impl HeaderReader for FragmentHeader {
    type Header = io::Result<Self>;

    fn read(rdr: &mut io::Cursor<&[u8]>) -> Self::Header {
        let id = rdr.read_u8()?;
        let num_fragments = rdr.read_u8()?;

        Ok(Self::new(id, num_fragments))
    }

    /// Get the size of this header.
    fn size(&self) -> usize {
        *HEADER_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::{FragmentHeader, HeaderReader, HeaderWriter};
    use crate::net::DeliveryMethod;
    use crate::packet::PacketType;
    use std::io::Cursor;

    #[test]
    pub fn serializes_deserialize_fragment_header_test() {
        // create default header
        //        let standard_header =
        //            StandardHeader::new(DeliveryMethod::UnreliableUnordered, PacketTypeId::Fragment);
        //
        //        let packet_header = AckedPacketHeader::new(standard_header.clone(), 1, 1, 5421);
        //
        //        // create fragment header with the default header and acked header.
        //        let fragment = FragmentHeader::new(1, 0, 1);
        //        let mut fragment_buffer = Vec::with_capacity(fragment.size() + 1);
        //        fragment.write(&mut fragment_buffer).unwrap();
        //
        //        let mut cursor: Cursor<&[u8]> = Cursor::new(fragment_buffer.as_slice());
        //        let fragment_deserialized = FragmentHeader::read(&mut cursor).unwrap();
        //
        //        assert_eq!(fragment_deserialized.id, 0);
        //        assert_eq!(fragment_deserialized.num_fragments, 1);
        //        assert_eq!(fragment_deserialized.sequence_num, 1);
        //
        //        assert!(fragment_deserialized.packet_header.is_some());
        //
        //        let fragment_packet_header = fragment_deserialized.packet_header.unwrap();
        //        assert_eq!(fragment_packet_header.sequence_num(), 1);
        //        assert_eq!(fragment_packet_header.last_acked(), 1);
        //        assert_eq!(fragment_packet_header.ack_field(), 5421);
    }

    #[test]
    pub fn header_size_test() {
        assert_eq!(FragmentHeader::default().size(), 2);
    }
}
