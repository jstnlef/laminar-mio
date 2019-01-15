use std::fmt::{self, Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum NetworkErrorKind {
    /// Error relating to receiving or parsing a fragment
    FragmentError(FragmentErrorKind),
    /// Wrapper around a std io::Error
    IOError(io::Error),
    /// Did not receive enough data
    ReceivedDataToShort,
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
            NetworkErrorKind::ReceivedDataToShort => {
                write!(f, "The received data did not have any length.")
            },
        }
    }
}

/// Errors which could occur with constructing/parsing fragment contents
#[derive(Debug)]
pub enum FragmentErrorKind {
    PacketHeaderNotFound,
}
