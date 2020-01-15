use std::convert::TryFrom;

use super::error::RTPSequenceError;
use crate::rtsp::rtp_old::packet::{packet::RTPPacket, payload_type::RTPPayloadType};
use crate::rtsp::rtp::jpeg_payload;

pub enum RTPSequenceStatus {
    Ok,
    LastPacket,
}

pub struct RTPSequence {
    buffer: Vec<u8>,
    sequence_type: Option<RTPPayloadType>,
    header: Option<Vec<u8>>,
    last_package_number: Option<u16>,
}

impl RTPSequence {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            sequence_type: None,
            header: None,
            last_package_number: None,
        }
    }

    pub fn push(&mut self, buf: &[u8]) -> Result<RTPSequenceStatus, RTPSequenceError> {
        let rtp_packet = match RTPPacket::try_from(buf) {
            Ok(packet) => packet,
            Err(err) => return Err(RTPSequenceError::RTPPacketError(err)),
        };

        match self.sequence_type {
            Some(sequence_type) => {
                if !sequence_type.is_equals(rtp_packet.payload_type()) {
                    return Err(RTPSequenceError::PayloadTypeIsChanged);
                }
            }
            None => {
                self.sequence_type = Some(*rtp_packet.payload_type());
            }
        }

        match self.last_package_number {
            Some(number) => {
                if number < rtp_packet.sequence_number() {
                    self.last_package_number = Some(rtp_packet.sequence_number())
                } else {
                    return Err(RTPSequenceError::PackageLost);
                }
            }
            None => self.last_package_number = Some(rtp_packet.sequence_number()),
        }

        let (header, body) = jpeg_payload::parse(rtp_packet.payload(), self.header.is_some());

        if let None = self.header {
            self.header = Some(header.unwrap())
        }

        self.buffer.extend(body);

        if rtp_packet.marked() {
            return Ok(RTPSequenceStatus::LastPacket);
        }

        Ok(RTPSequenceStatus::Ok)
    }

    pub fn make(&mut self) -> Vec<u8> {
        let header = self.header.as_ref().expect("Unexpected error: header not found");
        
        let mut image = Vec::new();

        image.extend(header);
        image.extend(&self.buffer);

        image
    }

    pub fn clean(&mut self) {
        self.buffer = Vec::new();
        self.header = None;
    }
}
