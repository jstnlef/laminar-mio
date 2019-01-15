use std::{
    default::Default,
    time::Duration
};

#[derive(Clone)]
pub struct SocketConfig {
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
            receive_buffer_size: 1500,
            socket_event_buffer_size: 1024,
            socket_polling_timeout: Some(Duration::from_millis(100)),
        }
    }
}
