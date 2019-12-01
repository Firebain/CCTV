use std::convert::TryFrom;

use super::error::RTPPacketError;

#[derive(Clone, Copy)]
pub enum RTPPayloadType {
    JPEG
}

impl RTPPayloadType {
    pub fn key(&self) -> &'static str {
        match self {
            Self::JPEG => "jpeg"
        }
    }

    pub fn is_equals(&self, payload_type: &RTPPayloadType) -> bool {
        self.key() == payload_type.key()
    }
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