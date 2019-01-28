use super::RttMeasurer;
use crate::{
    config::SocketConfig,
    errors::{LaminarError, PacketError},
    net::{DeliveryMethod, ExternalAcks, LocalAckRecord},
    packet::{
        headers::{HeaderReader, ReliableHeader, StandardHeader},
        PacketType, ProcessedPacket,
    },
    protocol_version,
    sequence_buffer::{CongestionData, SequenceBuffer},
    Packet,
};
use std::{
    fmt, io,
    io::Read,
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
    dropped_packets: Vec<Box<[u8]>>,

    // congestion control
    rtt_measurer: RttMeasurer,
    congestion_data: SequenceBuffer<CongestionData>,
    rtt: f32,
}

impl VirtualConnection {
    pub fn new(remote_address: SocketAddr, config: &SocketConfig) -> Self {
        Self {
            last_packet_time: Instant::now(),
            remote_address,
            max_packet_size_bytes: config.max_packet_size_bytes(),

            // reliability control
            sequence_num: 0,
            local_acks: LocalAckRecord::default(),
            external_acks: ExternalAcks::default(),
            dropped_packets: Vec::new(),

            // congestion control
            rtt_measurer: RttMeasurer::new(&config),
            congestion_data: SequenceBuffer::with_capacity(<u16>::max_value() as usize),
            rtt: 0.0,
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
        let standard_header = StandardHeader::read(&mut cursor)?;

        if !protocol_version::valid_version(standard_header.protocol_version()) {
            return Err(LaminarError::ProtocolVersionMismatch.into());
        }

        if standard_header.packet_type() == PacketType::Fragment {

        }

        match standard_header.delivery_method() {
            DeliveryMethod::ReliableUnordered => {
                let reliable_header = ReliableHeader::read(&mut cursor)?;
                self.external_acks.ack(standard_header.sequence_num());

                // Update congestion information.
                let congestion_data = self.congestion_data.get_mut(reliable_header.last_acked());
                self.rtt = self.rtt_measurer.get_rtt(congestion_data);

                // Update dropped packets if there are any.
                let dropped_packets = self
                    .local_acks
                    .ack(reliable_header.last_acked(), reliable_header.ack_field());

                self.dropped_packets = dropped_packets.into_iter().map(|(_, p)| p).collect();
            }
            _ => {}
        }

        // Read the rest of the bytes from the cursor to get the payload.
        let mut payload = Vec::with_capacity(payload.len());
        cursor.read_to_end(&mut payload)?;

        Ok(Some(Packet::new(
            self.remote_address,
            payload.into_boxed_slice(),
            standard_header.delivery_method(),
        )))
    }

    /// This pre-process the given Packet to be send over the network.
    /// It will perform some actions related to how the packet should be delivered and return
    /// a ProcessedPacket
    pub fn process_outgoing(&mut self, packet: Packet) -> io::Result<ProcessedPacket> {
        if packet.payload().len() > self.max_packet_size_bytes {
            return Err(PacketError::ExceededMaxPacketSize.into());
        }

        let reliability_header = match packet.delivery_method() {
            // TODO: Only implementing the reliable packets currently
            DeliveryMethod::ReliableUnordered => {
                // Queue congestion data.
                self.congestion_data.insert(
                    CongestionData::new(self.sequence_num, Instant::now()),
                    self.sequence_num,
                );

                // Queue packet for awaiting acknowledgement.
                self.local_acks.enqueue(self.sequence_num, packet.payload());

                let header = ReliableHeader::new(
                    self.external_acks.last_acked(),
                    self.external_acks.ack_field(),
                );

                Some(header)
            }
            _ => None,
        };

        let processed_packet = ProcessedPacket::new(self.sequence_num, packet, reliability_header);

        // Increase local sequence number.
        self.sequence_num = self.sequence_num.wrapping_add(1);

        Ok(processed_packet)
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

    /// Check if this channel has dropped packets.
    ///
    /// You could directly call `ReliableChannel::drain_dropped_packets()` and if it returns an empty vector you know there are no packets.
    /// But draining a vector will have its extra check logic even if it's empty.
    /// So that's why this function exists just a little shortcut to check if there are dropped packets which will be faster at the end.
    pub fn has_dropped_packets(&self) -> bool {
        !self.dropped_packets.is_empty()
    }

    /// Creates a draining iterator that removes dropped packets and yield the ones that are removed.
    ///
    /// So why drain?
    /// You have to think about the packet flow first.
    /// 1. Once we send a packet we place it in a queue until acknowledged.
    /// 2. If the packet doesn't get acknowledged in some time it will be dropped.
    /// 3. When we notice the packet drop we directly want to resend the dropped packet.
    /// 4. Once we notice that we start at '1' again.
    ///
    /// So keeping track of old dropped packets does not make sense, at least for now.
    /// We except when dropped packets are retrieved they will be sent out so we don't need to keep track of them internally the caller of this function will have ownership over them after the call.
    pub fn drain_dropped_packets(&mut self) -> Vec<Box<[u8]>> {
        self.dropped_packets.drain(..).collect()
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
