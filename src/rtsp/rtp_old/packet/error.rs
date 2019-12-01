use std::error;
use std::fmt;

#[derive(Debug)]
pub enum RTPPacketError {
    UnsupportedVersion,
    UnsupportedPayloadType,
    UnexpectedError(&'static str)
}

impl fmt::Display for RTPPacketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnsupportedVersion => write!(f, "Only 2 version is supported"),
            Self::UnsupportedPayloadType => write!(f, "Only jpeg payload is supported"),
            Self::UnexpectedError(cause) => write!(f, "Unexpected error: {}", cause)
        }
    }
}

impl error::Error for RTPPacketError {}