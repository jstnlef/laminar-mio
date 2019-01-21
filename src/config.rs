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
    pub fn fragment_size_bytes(&self) -> u16 {
        self.fragment_size_bytes
    }

    #[inline]
    pub fn idle_connection_timeout(&self) -> Duration {
        self.idle_connection_timeout
    }

    #[inline]
    pub fn receive_buffer_size_bytes(&self) -> usize {
        self.receive_buffer_size_bytes
    }

    #[inline]
    pub fn socket_event_buffer_size(&self) -> usize {
        self.socket_event_buffer_size
    }

    #[inline]
    pub fn socket_polling_timeout(&self) -> Option<Duration> {
        self.socket_polling_timeout
    }
}

impl Default for SocketConfig {
    fn default() -> Self {
        Self {
            fragment_size_bytes: 1450,
            idle_connection_timeout: Duration::from_secs(5),
            receive_buffer_size_bytes: 1500,
            socket_event_buffer_size: 1024,
            socket_polling_timeout: Some(Duration::from_millis(100)),
        }
    }
}
