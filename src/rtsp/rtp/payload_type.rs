use std::convert::TryFrom;

use super::error::RTPPacketError;

pub enum RTPPayloadType {
    JPEG
}

impl TryFrom<u8> for RTPPayloadType {
    type Error = RTPPacketError;

    fn try_from(value: u8) -> Result<Self, RTPPacketError> {
        match value {
            26 => Ok(Self::JPEG),
            _ => Err(RTPPacketError::UnsupportedPayloadType)
        }
    }
}