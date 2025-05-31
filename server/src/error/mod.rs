use std::{
    fmt,
    io,
};
use super::{
    ProtocolError,
    MAX_MESSAGE_SIZE,
};

impl ProtocolError {
    pub fn is_fatal(&self) -> bool {
        match self {
            ProtocolError::Io(e) => e.kind() != io::ErrorKind::WouldBlock,
            ProtocolError::MessageTooLarge(_) => false,
            _ => true,
        }
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::Io(err) => write!(f, "IO error: {}", err),
            ProtocolError::MessageTooLarge(size) => 
                write!(f, "Message too large ({} > {})", size, MAX_MESSAGE_SIZE),
            ProtocolError::ContentError(msg) => write!(f, "Content error: {}", msg),
        }
    }
}

impl std::error::Error for ProtocolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProtocolError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for ProtocolError {
    fn from(err: io::Error) -> Self {
        ProtocolError::Io(err)
    }
}

// Ensure thread safety
unsafe impl Send for ProtocolError {}
unsafe impl Sync for ProtocolError {}

// #[derive(Serialize)]
// pub struct ErrorResponse {
//     code: u16,
//     message: String,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     request_id: Option<u64>,
// }
