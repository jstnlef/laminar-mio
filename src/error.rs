mod kinds;

pub use self::kinds::FragmentError;
pub use self::kinds::LaminarError;

use std::fmt::{self, Display, Formatter};
use std::result::Result;

pub type NetworkResult<T> = Result<T, NetworkError>;

#[derive(Debug)]
pub struct NetworkError {
    kind: LaminarError,
}

impl NetworkError {
    pub fn new(kind: LaminarError) -> Self {
        Self { kind }
    }

    pub fn kind(&self) -> &LaminarError {
        &self.kind
    }
}

impl Display for NetworkError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.kind, f)
    }
}

impl From<LaminarError> for NetworkError {
    fn from(kind: LaminarError) -> NetworkError {
        NetworkError { kind }
    }
}

impl From<FragmentError> for NetworkError {
    fn from(inner: FragmentError) -> Self {
        LaminarError::FragmentError(inner).into()
    }
}
