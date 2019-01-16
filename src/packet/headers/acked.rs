use super::{calc_header_size, HeaderReader, HeaderWriter, StandardHeader};
use crate::error::NetworkResult;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use lazy_static::lazy_static;
use std::io::Cursor;

lazy_static! {
    static ref HEADER_SIZE: u8 = calc_header_size::<AckedPacketHeader>();
}

/// This header providing reliability information.
#[derive(Copy, Clone, Debug)]
pub struct AckedPacketHeader {
    /// StandardHeader for the Acked Packet
    pub standard_header: StandardHeader,
    /// this is the sequence number so that we can know where in the sequence of packages this packet belongs.
    sequence_num: u16,
    // this is the last acknowledged sequence number.
    last_acked: u16,
    // this is a bitfield of all last 32 acknowledged packages
    ack_field: u32,
}

impl AckedPacketHeader {
    /// When we compose packet headers, the local sequence becomes the sequence number of the packet, and the remote sequence becomes the ack.
    /// The ack bitfield is calculated by looking into a queue of up to 33 packets, containing sequence numbers in the range [remote sequence - 32, remote sequence].
    /// We set bit n (in [1,32]) in ack bits to 1 if the sequence number remote sequence - n is in the received queue.
    pub fn new(
        standard_header: StandardHeader,
        sequence_num: u16,
        last_acked: u16,
        ack_field: u32,
    ) -> AckedPacketHeader {
        AckedPacketHeader {
            standard_header,
            sequence_num,
            last_acked,
            ack_field,
        }
    }

    /// Get the sequence number from this packet.
    pub fn sequence_num(&self) -> u16 {
        self.sequence_num
    }

    /// Get last acknowledged sequence number.
    pub fn last_acked(&self) -> u16 {
        self.last_acked
    }

    /// Get bit field of all last 32 acknowledged packages
    pub fn ack_field(&self) -> u32 {
        self.ack_field
    }
}

impl Default for AckedPacketHeader {
    fn default() -> Self {
        AckedPacketHeader::new(StandardHeader::default(), 0, 0, 0)
    }
}

impl HeaderWriter for AckedPacketHeader {
    fn write(&self, buffer: &mut Vec<u8>) -> NetworkResult<()> {
        self.standard_header.write(buffer)?;
        buffer.write_u16::<BigEndian>(self.sequence_num)?;
        buffer.write_u16::<BigEndian>(self.last_acked)?;
        buffer.write_u32::<BigEndian>(self.ack_field)?;
        Ok(())
    }
}

impl HeaderReader for AckedPacketHeader {
    type Header = NetworkResult<AckedPacketHeader>;

    fn read(rdr: &mut Cursor<&[u8]>) -> Self::Header {
        let standard_header = StandardHeader::read(rdr)?;
        let sequence_num = rdr.read_u16::<BigEndian>()?;
        let last_acked = rdr.read_u16::<BigEndian>()?;
        let ack_field = rdr.read_u32::<BigEndian>()?;

        Ok(Self {
            standard_header,
            sequence_num,
            last_acked,
            ack_field,
        })
    }

    fn size(&self) -> u8 {
        *HEADER_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::{AckedPacketHeader, HeaderReader, HeaderWriter, StandardHeader};
    use std::io::Cursor;

    #[test]
    pub fn serializes_deserialize_acked_header_test() {
        let packet_header = AckedPacketHeader::new(StandardHeader::default(), 1, 1, 5421);
        let mut buffer = Vec::with_capacity((packet_header.size() + 1) as usize);

        let _ = packet_header.write(&mut buffer);

        let mut cursor = Cursor::new(buffer.as_slice());

        match AckedPacketHeader::read(&mut cursor) {
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
        assert_eq!(AckedPacketHeader::default().size(), 14);
    }
}
