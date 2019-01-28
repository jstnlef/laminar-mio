
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
/// Id to identify an certain packet type.
pub enum PacketType {
    /// Full packet that is not fragmented
    Packet = 0,
    /// Fragment of a full packet
    Fragment = 1,
    /// Special packet that serves as a heartbeat
    HeartBeat = 2,
    /// Special packet that disconnects
    Disconnect = 3,
    /// Unknown packet type
    Unknown = 255,
}

impl PacketType {
    /// Get integer value from `PacketTypeId` enum.
    pub fn get_id(packet_type: PacketType) -> u8 {
        packet_type as u8
    }

    /// Get `PacketTypeid` enum instance from integer value.
    pub fn get_packet_type(packet_type_id: u8) -> PacketType {
        match packet_type_id {
            0 => PacketType::Packet,
            1 => PacketType::Fragment,
            2 => PacketType::HeartBeat,
            3 => PacketType::Disconnect,
            _ => PacketType::Unknown,
        }
    }
}
