mod kinds;

pub use self::kinds::FragmentErrorKind;
pub use self::kinds::NetworkErrorKind;

use std::fmt::{self, Display, Formatter};
use std::io;
use std::result::Result;

pub type NetworkResult<T> = Result<T, NetworkError>;

#[derive(Debug)]
pub struct NetworkError {
    kind: NetworkErrorKind,
}

impl NetworkError {
    pub fn new(kind: NetworkErrorKind) -> Self {
        Self { kind }
    }
}

impl Display for NetworkError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.kind, f)
    }
}

impl From<NetworkErrorKind> for NetworkError {
    fn from(kind: NetworkErrorKind) -> NetworkError {
        NetworkError { kind }
    }
}

impl From<FragmentErrorKind> for NetworkError {
    fn from(inner: FragmentErrorKind) -> Self {
        NetworkErrorKind::FragmentError(inner).into()
    }
}

impl From<io::Error> for NetworkError {
    fn from(inner: io::Error) -> NetworkError {
        NetworkErrorKind::IOError(inner).into()
    }
}
