use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    io,
};

#[derive(Debug)]
pub enum LaminarError {
    /// Error relating to receiving or parsing a fragment
    FragmentError(FragmentError),
    /// Error returned if you try to sent data to the socket if the polling loop hasn't yet
    /// been started
    PollingNotStarted,
    /// Protocol versions did not match
    ProtocolVersionMismatch,
    /// Did not receive enough data
    ReceivedDataTooShort,
}

impl Display for LaminarError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            LaminarError::FragmentError(kind) => write!(
                f,
                "Something went wrong with receiving/parsing fragments. Reason: {:?}.",
                kind
            ),
            LaminarError::PollingNotStarted => write!(
                f,
                "Trying to send a packet without first starting the event loop."
            ),
            LaminarError::ProtocolVersionMismatch => {
                write!(f, "The protocol versions do not match.")
            }
            LaminarError::ReceivedDataTooShort => {
                write!(f, "The received data did not have any length.")
            }
        }
    }
}

impl Error for LaminarError {}

impl Into<io::Error> for LaminarError {
    fn into(self) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, self)
    }
}

/// Errors which could occur with constructing/parsing fragment contents
#[derive(Debug)]
pub enum FragmentError {
    /// A packet header was not found in the packet
    PacketHeaderNotFound,
}

impl Display for FragmentError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            FragmentError::PacketHeaderNotFound => write!(f, "Packet header not found."),
        }
    }
}

impl Error for FragmentError {}

impl Into<io::Error> for FragmentError {
    fn into(self) -> io::Error {
        LaminarError::FragmentError(self).into()
    }
}
