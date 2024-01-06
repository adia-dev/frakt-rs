use super::error::NetworkingError;

pub type NetworkingResult<T> = Result<T, NetworkingError>;
