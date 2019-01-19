use std::{default::Default, time::Duration};

#[derive(Clone)]
pub struct SocketConfig {
    /// The maximal amount of time to keep `VirtualConnection`s around before cleaning them up.
    pub idle_connection_timeout: Duration,
    /// This is the size of the buffer the underlying UDP socket reads data into.
    /// Default: Max MTU - 1500 bytes
    pub receive_buffer_size: usize,
    // This is the size of the event buffer we read socket events into.
    pub socket_event_buffer_size: usize,
    /// Optional duration specifying how long we should block polling for socket events.
    pub socket_polling_timeout: Option<Duration>,
}

impl Default for SocketConfig {
    fn default() -> Self {
        Self {
            idle_connection_timeout: Duration::from_secs(5),
            receive_buffer_size: 1500,
            socket_event_buffer_size: 1024,
            socket_polling_timeout: Some(Duration::from_millis(100)),
        }
    }
}
