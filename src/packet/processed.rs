use crate::{
    errors::FragmentError,
    packet::headers::{
        FragmentHeader, HeaderReader, HeaderWriter, ReliableHeader, StandardHeader
    },
    packet::{Packet, PacketType},
};
use std::{
    io::{self, Write},
    net::SocketAddr,
};

/// Wrapper struct to hold the fully serialized packet (includes header data)
pub struct ProcessedPacket {
    sequence_num: u16,
    packet: Packet,
    reliability: Option<ReliableHeader>,
    // This will be used by the fragments function. There is likely a more efficient way to handle
    // fragments.
    serialized_fragments: Vec<Vec<u8>>,
}

impl ProcessedPacket {
    pub fn new(sequence_num: u16, packet: Packet, reliability: Option<ReliableHeader>) -> Self {
        Self {
            sequence_num,
            packet,
            reliability,
            serialized_fragments: Vec::new(),
        }
    }

    /// Get the endpoint from this packet.
    pub fn address(&self) -> SocketAddr {
        self.packet.address
    }

    /// Returns an iterator yielding payload fragments
    pub fn fragments(
        &mut self,
        fragment_size: u16,
        max_fragments: u8,
    ) -> io::Result<impl Iterator<Item = &[u8]>> {
        let payload_length = self.packet.payload.len();
        let num_fragments = total_fragments_needed(payload_length, fragment_size) as u8; /* safe cast max_fragments is u8 */

        if num_fragments > max_fragments {
            return Err(FragmentError::ExceededMaxFragments.into());
        }

        if num_fragments <= 1 {
            self.serialize_unfragmented()?;
        } else {
            self.serialize_fragmented(num_fragments, fragment_size)?;
        }

        Ok(self
            .serialized_fragments
            .iter()
            .map(|fragment| fragment.as_slice()))
    }

    fn serialize_unfragmented(&mut self) -> io::Result<()> {
        // Calculate the buffer size
        let standard_header = StandardHeader::new(
            self.packet.delivery_method,
            PacketType::Packet,
            self.sequence_num,
        );

        let mut buffer_size = standard_header.size();
        buffer_size += if let Some(reliability_header) = self.reliability {
            reliability_header.size()
        } else {
            0
        };
        buffer_size += self.packet.payload.len();

        // Create the buffer and write out the header info plus the payload
        let mut buffer = Vec::with_capacity(buffer_size);
        standard_header.write(&mut buffer)?;
        if let Some(reliability_header) = self.reliability {
            reliability_header.write(&mut buffer)?;
        }
        buffer.extend(self.packet.payload.iter());

        self.serialized_fragments.push(buffer);
        Ok(())
    }

    fn serialize_fragmented(&mut self, num_fragments: u8, fragment_size: u16) -> io::Result<()> {
        let standard_header = StandardHeader::new(
            self.packet.delivery_method,
            PacketType::Fragment,
            self.sequence_num,
        );

        for fragment_id in 0..num_fragments {
            let fragment_header = FragmentHeader::new(fragment_id, num_fragments);
            // Calculate the buffer size
            let mut buffer_size = standard_header.size();
            buffer_size += fragment_header.size();
            buffer_size += if let Some(reliability_header) = self.reliability {
                reliability_header.size()
            } else {
                0
            };
            buffer_size += fragment_size as usize;

            // Create the buffer and write out the header info plus the payload
            let mut buffer = Vec::with_capacity(buffer_size);
            standard_header.write(&mut buffer)?;
            fragment_header.write(&mut buffer)?;
            if let Some(reliability_header) = self.reliability {
                reliability_header.write(&mut buffer)?;
            }
            // get start end pos in buffer
            let start_fragment_pos = (u16::from(fragment_id) * fragment_size) as usize;
            let mut end_fragment_pos = ((u16::from(fragment_id) + 1) * fragment_size) as usize;
            // If remaining buffer fits int one packet just set the end position to the length of the packet payload.
            let payload_length = self.packet.payload.len();
            if end_fragment_pos > payload_length {
                end_fragment_pos = payload_length;
            }
            let fragment_data = &self.packet.payload[start_fragment_pos..end_fragment_pos];
            buffer.write_all(fragment_data)?;
            self.serialized_fragments.push(buffer);
        }

        Ok(())
    }
}

/// This functions checks how many times a number fits into another number and will round up.
///
/// For example we have two numbers:
/// - number 1 = 4000;
/// - number 2 = 1024;
/// If you do it the easy way the answer will be 4000/1024 = 3.90625.
/// But since we care about how how many whole times the number fits in we need the result 4.
///
/// Note that when rust is rounding it is always rounding to zero (3.456 as u32 = 3)
/// 1. calculate with modulo if `number 1` fits exactly in the `number 2`.
/// 2. Divide `number 1` with `number 2` (this wil be rounded to zero by rust)
/// 3. So in all cases we need to add 1 to get the right amount of fragments.
///
/// lets take an example
///
/// Calculate modules:
/// - number 1 % number 2 = 928
/// - this is bigger than 0 so remainder = 1
///
/// Calculate how many times the `number 1` fits in `number 2`:
/// - number 1 / number 2 = 3,90625 (this will be rounded to 3)
/// - add remainder from above to 3 = 4.
///
/// The above described method will figure out for all number how many times it fits into another number rounded up.
///
/// So an example of dividing an packet of bytes we get these fragments:
///
/// So for 4000 bytes we need 4 fragments
/// [fragment: 1024] [fragment: 1024] [fragment: 1024] [fragment: 928]
fn total_fragments_needed(payload_length: usize, fragment_size: u16) -> u16 {
    let payload_length = payload_length as u16;
    let remainder = if payload_length % fragment_size > 0 {
        1
    } else {
        0
    };
    ((payload_length / fragment_size) + remainder)
}

#[cfg(test)]
mod tests {
    use super::{
        total_fragments_needed, FragmentHeader, HeaderReader, ProcessedPacket, ReliableHeader,
        StandardHeader,
    };
    use crate::Packet;
    use std::io::{Cursor, Read};
    use std::net::SocketAddr;

    fn create_processed(payload: Vec<u8>, reliability: Option<ReliableHeader>) -> ProcessedPacket {
        let address: SocketAddr = "127.0.0.1:9000".parse().unwrap();
        let packet = Packet::unreliable(address, payload);
        let sequence_num = 0;
        ProcessedPacket::new(sequence_num, packet, reliability)
    }

    #[test]
    pub fn test_processed_no_fragmentation_no_reliability() {
        let payload = "hello!".as_bytes().to_owned();
        let mut processed = create_processed(payload.clone(), None);

        let serialized: Vec<&[u8]> = processed.fragments(1024, 10).unwrap().collect();

        assert_eq!(serialized.len(), 1);

        let message = serialized[0];

        let mut cursor = Cursor::new(message);

        // message must have standard header
        let standard_header = StandardHeader::read(&mut cursor).unwrap();
        assert_eq!(standard_header.sequence_num(), 0);

        // the next bytes must be payload
        let mut deserialized_message = Vec::new();
        cursor.read_to_end(&mut deserialized_message).unwrap();
        assert_eq!(payload, deserialized_message);
    }

    #[test]
    pub fn test_processed_no_fragmentation_with_reliability() {
        let payload = "hello!".as_bytes().to_owned();
        let reliable = ReliableHeader::new(1, 5421);
        let mut processed = create_processed(payload.clone(), Some(reliable));

        let serialized: Vec<&[u8]> = processed.fragments(1024, 10).unwrap().collect();

        assert_eq!(serialized.len(), 1);

        let message = serialized[0];

        let mut cursor = Cursor::new(message);

        // message must have standard header
        let standard_header = StandardHeader::read(&mut cursor).unwrap();
        assert_eq!(standard_header.sequence_num(), 0);

        // message must have a reliability header
        let reliable_header = ReliableHeader::read(&mut cursor).unwrap();
        assert_eq!(reliable_header.last_acked(), 1);
        assert_eq!(reliable_header.ack_field(), 5421);

        // the next bytes must be payload
        let mut deserialized_message = Vec::new();
        cursor.read_to_end(&mut deserialized_message).unwrap();
        assert_eq!(payload, deserialized_message);
    }

    #[test]
    pub fn test_processed_fragmentation_no_reliability() {
        let payload = "hello world!".as_bytes().to_owned();
        let mut processed = create_processed(payload.clone(), None);

        let serialized: Vec<&[u8]> = processed.fragments(5, 10).unwrap().collect();

        assert_eq!(serialized.len(), 3);

        for (index, packet) in serialized.iter().enumerate() {
            let mut cursor = Cursor::new(*packet);
            // message must have standard header
            let standard_header = StandardHeader::read(&mut cursor).unwrap();
            assert_eq!(standard_header.sequence_num(), 0);

            // message must have a fragment header
            let fragment_header = FragmentHeader::read(&mut cursor).unwrap();
            assert_eq!(fragment_header.id(), index as u8);
            assert_eq!(fragment_header.fragment_count(), 3);

            // the next bytes must be payload
            let mut deserialized_message = Vec::new();
            cursor.read_to_end(&mut deserialized_message).unwrap();
            assert!(deserialized_message.len() <= 5);
        }
    }

    #[test]
    pub fn test_processed_fragmentation_and_reliability() {
        let payload = "hello world!".as_bytes().to_owned();
        let reliable = ReliableHeader::new(1, 5421);
        let mut processed = create_processed(payload.clone(), Some(reliable));

        let serialized: Vec<&[u8]> = processed.fragments(5, 10).unwrap().collect();

        assert_eq!(serialized.len(), 3);

        for (index, packet) in serialized.iter().enumerate() {
            let mut cursor = Cursor::new(*packet);
            // message must have standard header
            let standard_header = StandardHeader::read(&mut cursor).unwrap();
            assert_eq!(standard_header.sequence_num(), 0);

            // message must have a fragment header
            let fragment_header = FragmentHeader::read(&mut cursor).unwrap();
            assert_eq!(fragment_header.id(), index as u8);
            assert_eq!(fragment_header.fragment_count(), 3);

            // message must have a reliability header
            let reliable_header = ReliableHeader::read(&mut cursor).unwrap();
            assert_eq!(reliable_header.last_acked(), 1);
            assert_eq!(reliable_header.ack_field(), 5421);

            // the next bytes must be payload
            let mut deserialized_message = Vec::new();
            cursor.read_to_end(&mut deserialized_message).unwrap();
            assert!(deserialized_message.len() <= 5);
        }
    }

    #[test]
    pub fn total_fragments_needed_test() {
        let fragment_number = total_fragments_needed(4000, 1024);
        let fragment_number1 = total_fragments_needed(500, 1024);

        assert_eq!(fragment_number, 4);
        assert_eq!(fragment_number1, 1);
    }
}
