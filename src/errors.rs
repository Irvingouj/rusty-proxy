use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub(crate) enum ProxyError {
    CrlfSequenceNotFoundError,
    // Other variants can be added here
}

impl fmt::Display for ProxyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProxyError::CrlfSequenceNotFoundError => write!(f, "CRLF sequence not found"),
            // Handle other variants here
        }
    }
}

impl Error for ProxyError {}
