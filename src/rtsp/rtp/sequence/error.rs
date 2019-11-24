use std::error;
use std::fmt;

use crate::rtsp::rtp::packet::{error::RTPPacketError};

#[derive(Debug)]
pub enum RTPSequenceError {
    PayloadTypeIsChanged,
    PackageLost,
    RTPPacketError(RTPPacketError),
}

impl fmt::Display for RTPSequenceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PayloadTypeIsChanged => write!(f, "While building sequence of frame payload type is changed"),
            Self::PackageLost => write!(f, "Package lost while buiilding sequence"),
            Self::RTPPacketError(error) => write!(f, "RTP packet parsing error: {}", error)
        }
    }
}

impl error::Error for RTPSequenceError {}