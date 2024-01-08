use std::fmt;

use colored::Colorize;

#[derive(Debug)]
pub enum NetworkingError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    Error(Box<dyn std::error::Error>),
}

impl fmt::Display for NetworkingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkingError::IoError(err) => {
                write!(f, "[{}] {}", "IO Error".red(), err)
            }
            NetworkingError::JsonError(err) => {
                write!(f, "[{}] {}", "JSON Error".red(), err)
            }
            NetworkingError::Error(err) => {
                write!(f, "[{}] {}", "General Error".red(), err)
            }
        }
    }
}

impl From<std::io::Error> for NetworkingError {
    fn from(err: std::io::Error) -> NetworkingError {
        NetworkingError::IoError(err)
    }
}

impl From<serde_json::Error> for NetworkingError {
    fn from(err: serde_json::Error) -> NetworkingError {
        NetworkingError::JsonError(err)
    }
}

impl From<Box<dyn std::error::Error>> for NetworkingError {
    fn from(err: Box<dyn std::error::Error>) -> NetworkingError {
        NetworkingError::Error(err)
    }
}
