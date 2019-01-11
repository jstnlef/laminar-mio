use std::default::Default;

#[derive(Clone)]
pub struct SocketConfig {
    /// This is the size of the buffer the underlying UDP socket reads data into.
    /// Default: Max MTU - 1500 bytes
    pub receive_buffer_size: usize,
}

impl Default for SocketConfig {
    fn default() -> Self {
        Self {
            receive_buffer_size: 1500,
        }
    }
}
