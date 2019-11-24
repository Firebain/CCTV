use std::convert::TryFrom;

use super::error::RTPPacketError;
use super::payload_type::RTPPayloadType;

fn get_n_bit(byte: u8, n: usize) -> bool {
    let mask = 1 << n;
    let masked_byte = byte & mask;
    let bit = masked_byte >> n;

    bit == 1
}

pub struct RTPPacket {
    marker: bool,
    payload_type: RTPPayloadType,
    sequence_number: u16,
    timestamp: u32,
    ssrc: u32,
    payload: Vec<u8>,
}

impl RTPPacket {
    pub fn marked(&self) -> bool {
        self.marker
    }
    
    pub fn sequence_number(&self) -> u16 {
        self.sequence_number
    }

    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }

    pub fn ssrc(&self) -> u32 {
        self.ssrc
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

        let padding = get_n_bit(buf[0], 5);
        if padding {
            return Err(RTPPacketError::UnexpectedError("Unexpected padding header"));
        }

        let extension = get_n_bit(buf[0], 4);
        if extension {
            return Err(RTPPacketError::UnexpectedError("Unexpected extension header"));
        }

        let csrc_count = buf[0] & 0b0000_1111;
        if csrc_count > 0 {
            return Err(RTPPacketError::UnexpectedError("Unexpected csrc header"));
        }

        let marker = buf[1] >> 7 == 1;
        let payload_type = RTPPayloadType::try_from(buf[1] & 0b0111_1111)?;
        let sequence_number = u16::from_be_bytes([buf[2], buf[3]]);
        let timestamp = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let ssrc = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);

        let payload = Vec::from(&buf[12..]);

        Ok(Self {
            marker,
            payload_type,
            sequence_number,
            timestamp,
            ssrc,
            payload,
        })
    }
}