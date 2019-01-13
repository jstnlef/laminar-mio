mod acked;
mod fragment;
mod heart_beat;
mod standard;

pub use self::acked::AckedPacketHeader;
pub use self::fragment::FragmentHeader;
pub use self::heart_beat::HeartBeatHeader;
pub use self::standard::{StandardHeader, HEADER_SIZE as STANDARD_HEADER_SIZE};

use crate::error::NetworkResult;
use std::io::Cursor;

/// Trait for parsing a header
pub trait HeaderWriter {
    /// Write the header to the given buffer.
    fn write(&self, buffer: &mut Vec<u8>) -> NetworkResult<()>;
}

/// Trait that supports reading a Header from a packet
pub trait HeaderReader {
    /// Associated type for the HeaderReader, since it reads it from a Header
    type Header;

    /// Read the specified header from the given Cursor.
    fn read(rdr: &mut Cursor<&[u8]>) -> Self::Header;

    /// This will get the size of the header.
    fn size(&self) -> u8;
}

/// Small helper method to statically calculate the written size of a header struct
fn calc_header_size<T: Default + HeaderWriter>() -> u8 {
    let mut buffer: Vec<u8> = Vec::new();
    T::default().write(&mut buffer);
    buffer.len() as u8
}
