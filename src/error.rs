use std::io;
use std::result::Result;

pub type NetworkResult<T> = Result<T, NetworkError>;

pub struct NetworkError {
    kind: NetworkErrorKind,
}

enum NetworkErrorKind {
    /// Wrapper around a std io::Error
    IOError(io::Error),
}
