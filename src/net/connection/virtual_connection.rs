use crate::{
    errors::LaminarError,
    packet::{
        headers::{HeaderReader, HeaderWriter, StandardHeader},
        PacketTypeId, ProcessedPacket,
    },
    protocol_version, Packet,
};
use std::{
    fmt, io,
    net::SocketAddr,
    time::{Duration, Instant},
};

/// Contains the information about 'virtual connections' over UDP.
pub struct VirtualConnection {
    /// Last time we received a packet from this client
    last_packet_time: Instant,
    /// The address of the remote endpoint
    remote_address: SocketAddr,
}

impl VirtualConnection {
    pub fn new(remote_address: SocketAddr) -> Self {
        Self {
            last_packet_time: Instant::now(),
            remote_address,
        }
    }

    /// This processes incoming payload data and returns a packet if the data is complete.
    ///
    /// Returns `Ok(None)`:
    /// 1. In the case of fragmentation and not all fragments are received
    /// 2. In the case of the packet being queued for ordering and we are waiting on older packets
    ///    first.
    pub fn process_incoming(&mut self, payload: &[u8]) -> io::Result<Option<Packet>> {
        self.last_packet_time = Instant::now();

        let mut cursor = io::Cursor::new(payload);
        let header = StandardHeader::read(&mut cursor)?;

        if !protocol_version::valid_version(header.protocol_version()) {
            return Err(LaminarError::ProtocolVersionMismatch.into());
        }

        Ok(Some(Packet::reliable_unordered(
            self.remote_address,
            payload[header.size()..].to_owned(),
        )))
    }

    /// This pre-process the given buffer to be send over the network.
    /// 1. It will append the right header.
    /// 2. It will perform some actions related to how the packet should be delivered.
    pub fn process_outgoing(&mut self, packet: Packet) -> io::Result<ProcessedPacket> {
        let header = StandardHeader::new(packet.delivery_method(), PacketTypeId::Packet);
        let mut buffer = Vec::with_capacity(header.size() + packet.payload().len());
        header.write(&mut buffer)?;
        buffer.extend(packet.payload());

        Ok(ProcessedPacket::new(packet.address(), buffer))
    }

    /// Represents the duration since we last received a packet from this client
    pub fn time_since_last_packet(&self) -> Duration {
        let now = Instant::now();
        now.duration_since(self.last_packet_time)
    }

    /// The remote address of the client
    pub fn remote_address(&self) -> SocketAddr {
        self.remote_address
    }
}

impl fmt::Debug for VirtualConnection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}",
            self.remote_address.ip(),
            self.remote_address.port()
        )
    }
}
