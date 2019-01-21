use std::{default::Default, time::Duration};

#[derive(Clone)]
pub struct SocketConfig {
    /// This is the size of a fragment.
    /// If a packet is too large it needs to be split in fragments.
    ///
    /// Recommended value: +- 1450 (1500 is the default MTU)
    fragment_size_bytes: u16,
    /// The maximal amount of time to keep `VirtualConnection`s around before cleaning them up.
    idle_connection_timeout: Duration,
    /// These are the maximal fragments a packet could be divided into.
    ///
    /// Why can't I have more than 255 (u8)?
    /// This is because you don't want to send more then 256 fragments over UDP, with high amounts
    /// of fragments the chance for an invalid packet is very high.
    /// Use TCP instead (later we will probably support larger ranges but every fragment packet
    /// then needs to be present before an acknowledgement is sent).
    ///
    /// Recommended value: 16 but keep in mind that lower is better.
    max_fragments: u8,
    /// This is the size of the buffer the underlying UDP socket reads data into.
    /// Default: Max MTU - 1500 bytes
    receive_buffer_size_bytes: usize,
    // This is the size of the event buffer we read socket events (from `mio::Poll`) into.
    socket_event_buffer_size: usize,
    /// Optional duration specifying how long we should block polling for socket events.
    socket_polling_timeout: Option<Duration>,
}

impl SocketConfig {
    #[inline]
    pub const fn fragment_size_bytes(&self) -> u16 {
        self.fragment_size_bytes
    }

    #[inline]
    pub const fn idle_connection_timeout(&self) -> Duration {
        self.idle_connection_timeout
    }

    #[inline]
    pub const fn max_fragments(&self) -> u8 {
        self.max_fragments
    }

    /// Calculated value based on the maximum number of fragments and the fragment size.
    #[inline]
    pub const fn max_packet_size_bytes(&self) -> usize {
        self.max_fragments as usize + self.fragment_size_bytes as usize
    }

    #[inline]
    pub const fn receive_buffer_size_bytes(&self) -> usize {
        self.receive_buffer_size_bytes
    }

    #[inline]
    pub const fn socket_event_buffer_size(&self) -> usize {
        self.socket_event_buffer_size
    }

    #[inline]
    pub const fn socket_polling_timeout(&self) -> Option<Duration> {
        self.socket_polling_timeout
    }
}

impl Default for SocketConfig {
    fn default() -> Self {
        Self {
            fragment_size_bytes: 1450,
            idle_connection_timeout: Duration::from_secs(5),
            max_fragments: 16,
            receive_buffer_size_bytes: 1500,
            socket_event_buffer_size: 1024,
            socket_polling_timeout: Some(Duration::from_millis(100)),
        }
    }
}
