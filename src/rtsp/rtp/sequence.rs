use std::convert::TryFrom;

use std::error;
use std::fmt;

use super::jpeg_payload;
use super::package::{RTPPacket, RTPPacketError};

pub enum RTPSequenceStatus {
    Ok,
    LastPacket(Vec<u8>),
}

pub struct RTPSequence {
    buffer: Vec<u8>,
    header: Option<Vec<u8>>,
    last_package_number: Option<u16>,
}

impl RTPSequence {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            header: None,
            last_package_number: None,
        }
    }

    pub fn push(&mut self, buf: &[u8]) -> Result<RTPSequenceStatus, RTPSequenceError> {
        let rtp_packet = RTPPacket::try_from(buf)?;

        if let Some(number) = self.last_package_number {
            if number >= rtp_packet.sequence_number() {
                return Err(RTPSequenceError::PackageLost);
            }
        }

        self.last_package_number = Some(rtp_packet.sequence_number());

        let (header, body) = jpeg_payload::parse(rtp_packet.payload(), self.header.is_some());

        if self.header.is_none() {
            match header {
                Some(_) => self.header = header,
                None => return Err(RTPSequenceError::HeaderMissing)
            }
        }

        self.buffer.extend(body);

        if rtp_packet.marked() {
            match &self.header {
                Some(header) => {
                    let mut data = Vec::new();

                    data.extend(header);
                    data.extend(&self.buffer);

                    Ok(RTPSequenceStatus::LastPacket(data))
                }
                None => Err(RTPSequenceError::HeaderMissing),
            }
        } else {
            Ok(RTPSequenceStatus::Ok)
        }
    }

    pub fn clean(&mut self) {
        self.buffer = Vec::new();
        self.header = None;
    }
}

#[derive(Debug)]
pub enum RTPSequenceError {
    PackageLost,
    HeaderMissing,
    RTPPacketError(RTPPacketError),
}

impl From<RTPPacketError> for RTPSequenceError {
    fn from(err: RTPPacketError) -> RTPSequenceError {
        RTPSequenceError::RTPPacketError(err)
    }
}

impl fmt::Display for RTPSequenceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PackageLost => write!(f, "Package lost while buiilding sequence"),
            Self::HeaderMissing => write!(f, "Header missing in first package"),
            Self::RTPPacketError(error) => write!(f, "RTP packet parsing error: {}", error),
        }
    }
}

impl error::Error for RTPSequenceError {}
