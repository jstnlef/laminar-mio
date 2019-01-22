use super::{calc_header_size, HeaderReader, HeaderWriter};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use lazy_static::lazy_static;
use std::io;

lazy_static! {
    static ref HEADER_SIZE: usize = calc_header_size::<ReliableHeader>();
}

/// This header provides reliability information to the packet.
#[derive(Copy, Clone, Debug)]
pub struct ReliableHeader {
    /// This is the sequence number so that we can know where in the sequence of packets this packet belongs.
    sequence_num: u16,
    // This is the last acknowledged sequence number.
    last_acked: u16,
    // This is a bitfield of all last 32 acknowledged packets.
    ack_field: u32,
}

impl ReliableHeader {
    pub fn new(sequence_num: u16, last_acked: u16, ack_field: u32) -> Self {
        Self {
            sequence_num,
            last_acked,
            ack_field,
        }
    }

    /// Get the sequence number from this packet.
    #[inline]
    pub fn sequence_num(&self) -> u16 {
        self.sequence_num
    }

    /// Get last acknowledged sequence number.
    #[inline]
    pub fn last_acked(&self) -> u16 {
        self.last_acked
    }

    /// Get bit field of all last 32 acknowledged packets
    #[inline]
    pub fn ack_field(&self) -> u32 {
        self.ack_field
    }
}

impl Default for ReliableHeader {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

impl HeaderWriter for ReliableHeader {
    fn write(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        buffer.write_u16::<BigEndian>(self.sequence_num)?;
        buffer.write_u16::<BigEndian>(self.last_acked)?;
        buffer.write_u32::<BigEndian>(self.ack_field)?;
        Ok(())
    }
}

impl HeaderReader for ReliableHeader {
    type Header = io::Result<Self>;

    fn read(rdr: &mut io::Cursor<&[u8]>) -> Self::Header {
        let sequence_num = rdr.read_u16::<BigEndian>()?;
        let last_acked = rdr.read_u16::<BigEndian>()?;
        let ack_field = rdr.read_u32::<BigEndian>()?;

        Ok(Self::new(sequence_num, last_acked, ack_field))
    }

    fn size(&self) -> usize {
        *HEADER_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::{HeaderReader, HeaderWriter, ReliableHeader};
    use std::io::Cursor;

    #[test]
    pub fn serialize_deserialize_reliable_header_test() {
        let packet_header = ReliableHeader::new(1, 1, 5421);
        let mut buffer = Vec::with_capacity(packet_header.size() + 1);

        let _ = packet_header.write(&mut buffer);

        let mut cursor = Cursor::new(buffer.as_slice());

        match ReliableHeader::read(&mut cursor) {
            Ok(packet_deserialized) => {
                assert_eq!(packet_deserialized.sequence_num(), 1);
                assert_eq!(packet_deserialized.last_acked(), 1);
                assert_eq!(packet_deserialized.ack_field(), 5421);
            }
            Err(e) => println!("{:?}", e),
        }
    }

    #[test]
    pub fn header_size_test() {
        assert_eq!(ReliableHeader::default().size(), 8);
    }
}
