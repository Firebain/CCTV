use std::convert::TryFrom;
use std::error;
use std::fmt;

pub struct RTPPacket {
    marker: bool,
    sequence_number: u16,
    payload: Vec<u8>,
}

impl RTPPacket {
    pub fn marked(&self) -> bool {
        self.marker
    }

    pub fn sequence_number(&self) -> u16 {
        self.sequence_number
    }

    pub fn payload(&self) -> &Vec<u8> {
        &self.payload
    }
}

impl TryFrom<&[u8]> for RTPPacket {
    type Error = RTPPacketError;

    fn try_from(buf: &[u8]) -> Result<Self, RTPPacketError> {
        let version = buf[0] >> 6;

        if version != 2 {
            return Err(RTPPacketError::UnsupportedVersion);
        }

        let payload_type = buf[1] & 0b0111_1111;
        if payload_type != 26 {
            return Err(RTPPacketError::UnsupportedPayloadType);
        }

        let marker = buf[1] >> 7 == 1;
        let sequence_number = u16::from_be_bytes([buf[2], buf[3]]);

        let payload = Vec::from(&buf[12..]);

        Ok(Self {
            marker,
            sequence_number,
            payload,
        })
    }
}

#[derive(Debug)]
pub enum RTPPacketError {
    UnsupportedVersion,
    UnsupportedPayloadType,
}

impl fmt::Display for RTPPacketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnsupportedVersion => write!(f, "Only 2 version is supported"),
            Self::UnsupportedPayloadType => write!(f, "Only jpeg payload is supported"),
        }
    }
}

impl error::Error for RTPPacketError {}
