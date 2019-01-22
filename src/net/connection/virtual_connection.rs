use crate::{
    config::SocketConfig,
    errors::{LaminarError, PacketError},
    packet::{
        headers::{HeaderReader, HeaderWriter, EmptyHeader, StandardHeader, ReliableHeader},
        PacketTypeId, ProcessedPacket
    },
    net::{
        DeliveryMethod, LocalAckRecord, ExternalAcks
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
    /// Maximum size a packet can be.
    max_packet_size_bytes: usize,

    // TODO: These likely won't stay here
    // reliability control
    sequence_num: u16,
    local_acks: LocalAckRecord,
    external_acks: ExternalAcks,
}

impl VirtualConnection {
    pub fn new(remote_address: SocketAddr, config: &SocketConfig) -> Self {
        Self {
            last_packet_time: Instant::now(),
            remote_address,
            max_packet_size_bytes: config.max_packet_size_bytes(),
            sequence_num: 0,
            local_acks: LocalAckRecord::default(),
            external_acks: ExternalAcks::default()
        }
    }

    /// This processes incoming payload data and returns a packet if the data is complete.
    ///
    /// Returns `Ok(None)`:
    /// 1. In the case of fragmentation and not all fragments are received
    /// 2. In the case of the packet being queued for ordering and we are waiting on older packets
    ///    first.
    pub fn process_incoming(&mut self, payload: &[u8]) -> io::Result<Option<Packet>> {
        // TODO: Only implementing the reliable packets currently
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
        if packet.payload().len() > self.max_packet_size_bytes {
            return Err(PacketError::ExceededMaxPacketSize.into());
        }

        let typed_header = match packet.delivery_method() {
            // TODO: Only implementing the reliable packets currently
            DeliveryMethod::ReliableUnordered => {
                // Queue packet for awaiting acknowledgement.
                self.local_acks.enqueue(self.sequence_num, packet.payload());

                let header = ReliableHeader::new(
                    self.sequence_num,
                    self.external_acks.last_acked(),
                    self.external_acks.ack_field()
                );

                // Increase local sequence number.
                self.sequence_num = self.sequence_num.wrapping_add(1);
                header;
            }
            _ => {}
        };

        Ok(ProcessedPacket::new(
            packet.address(),
            packet.delivery_method(),
            packet.payload(),
        ))
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
