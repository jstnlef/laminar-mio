use crc::crc32;
use lazy_static::lazy_static;

lazy_static! {
    // Generated protocol version based on the version of the library
    static ref PROTOCOL_VERSION: String = format!("{}-{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    // The CRC32 of the current protocol version.
    static ref VERSION_CRC32: u32 = crc32::checksum_ieee(PROTOCOL_VERSION.as_bytes());
}

/// Get the current protocol version.
#[inline]
pub fn get_version() -> &'static str {
    &PROTOCOL_VERSION
}

/// This will return the crc32 from the current protocol version.
#[inline]
pub fn get_crc32() -> u32 {
    *VERSION_CRC32
}

/// Validate a crc32 with the current protocol version and return the results.
#[inline]
pub fn valid_version(protocol_version_crc32: u32) -> bool {
    protocol_version_crc32 == get_crc32()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_version() {
        let protocol_id = crc32::checksum_ieee(&PROTOCOL_VERSION.as_bytes());
        assert!(valid_version(protocol_id));
    }

    #[test]
    fn test_not_valid_version() {
        let protocol_id = crc32::checksum_ieee("not-laminar".as_bytes());
        assert!(!valid_version(protocol_id));
    }

    #[test]
    fn test_get_crc32() {
        assert_eq!(get_crc32(), *VERSION_CRC32);
    }

    #[test]
    fn test_get_version() {
        assert_eq!(get_version(), *PROTOCOL_VERSION);
    }
}
