mod fragment;
mod heart_beat;
mod reliable;
mod standard;

pub use self::fragment::FragmentHeader;
pub use self::heart_beat::HeartBeatHeader;
pub use self::reliable::{ReliableHeader, HEADER_SIZE as RELIABLE_HEADER_SIZE};
pub use self::standard::{StandardHeader, HEADER_SIZE as STANDARD_HEADER_SIZE};

use std::io;

/// Trait for parsing a header
pub trait HeaderWriter {
    /// Write the header to the given buffer.
    fn write(&self, buffer: &mut Vec<u8>) -> io::Result<()>;
}

/// Trait that supports reading a Header from a packet
pub trait HeaderReader {
    /// Associated type for the HeaderReader, since it reads it from a Header
    type Header;

    /// Read the specified header from the given Cursor.
    fn read(rdr: &mut io::Cursor<&[u8]>) -> Self::Header;

    /// This will get the size of the header.
    fn size(&self) -> usize;
}

/// Small helper method to statically calculate the written size of a header struct
fn calc_header_size<T: Default + HeaderWriter>() -> usize {
    let mut buffer: Vec<u8> = Vec::new();
    let _ = T::default().write(&mut buffer);
    buffer.len()
}
