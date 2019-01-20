use super::{calc_header_size, HeaderReader, HeaderWriter};
use crate::{packet::PacketTypeId, protocol_version::ProtocolVersion};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use lazy_static::lazy_static;
use std::io;

lazy_static! {
    static ref HEADER_SIZE: u8 = calc_header_size::<HeartBeatHeader>();
}

/// This header represents an heartbeat packet header.
/// A heart beat just keeps the client awake.
#[derive(Copy, Clone, Debug)]
pub struct HeartBeatHeader {
    packet_type_id: PacketTypeId,
}

impl HeartBeatHeader {
    /// Create new heartbeat header.
    pub fn new() -> Self {
        HeartBeatHeader {
            packet_type_id: PacketTypeId::HeartBeat,
        }
    }
}

impl Default for HeartBeatHeader {
    fn default() -> Self {
        HeartBeatHeader::new()
    }
}

impl HeaderWriter for HeartBeatHeader {
    fn write(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        buffer.write_u32::<BigEndian>(ProtocolVersion::get_crc32())?;
        buffer.write_u8(PacketTypeId::get_id(self.packet_type_id))?;

        Ok(())
    }
}

impl HeaderReader for HeartBeatHeader {
    type Header = io::Result<HeartBeatHeader>;

    fn read(rdr: &mut io::Cursor<&[u8]>) -> Self::Header {
        let _ = rdr.read_u32::<BigEndian>()?;
        let _ = rdr.read_u8();
        let header = Self {
            packet_type_id: PacketTypeId::HeartBeat,
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
    use super::{HeaderReader, HeartBeatHeader};

    #[test]
    pub fn header_size_test() {
        assert_eq!(HeartBeatHeader::default().size(), 5);
    }
}
