use std::{
    fmt::{self, Display, Formatter},
    io,
    sync::mpsc
};

#[derive(Debug)]
pub enum NetworkErrorKind {
    /// Error relating to receiving or parsing a fragment
    FragmentError(FragmentErrorKind),
    /// Wrapper around a std io::Error
    IOError(io::Error),
    /// Error returned if you try to sent data to the socket if the polling loop hasn't yet
    /// been started
    PollingNotStarted,
    /// Did not receive enough data
    ReceivedDataToShort
}

impl Display for NetworkErrorKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            NetworkErrorKind::FragmentError(kind) => write!(
                f,
                "Something went wrong with receiving/parsing fragments. Reason: {:?}.",
                kind
            ),
            NetworkErrorKind::IOError(e) => write!(f, "An IO Error occurred. Reason: {:?}.", e),
            NetworkErrorKind::PollingNotStarted => {
                write!(f, "Trying to send a packet without first starting the event loop")
            }
            NetworkErrorKind::ReceivedDataToShort => {
                write!(f, "The received data did not have any length.")
            }
        }
    }
}

/// Errors which could occur with constructing/parsing fragment contents
#[derive(Debug)]
pub enum FragmentErrorKind {
    PacketHeaderNotFound,
}
