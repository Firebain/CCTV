mod jpeg;

use crate::rtsp::rtp_old::packet::payload_type::RTPPayloadType;

pub enum RTPPayloadParser {
    JpegParser
}

impl From<RTPPayloadType> for RTPPayloadParser {
    fn from(payload_type: RTPPayloadType) -> Self {
        match payload_type {
            RTPPayloadType::JPEG => Self::JpegParser
        }
    }
}

impl RTPPayloadParser {
    pub fn parse(&self, data: &[u8], header_seated: bool) -> (Vec<u8>, Vec<u8>) {
        match self {
            RTPPayloadParser::JpegParser => jpeg::parse(data, header_seated)
        }
    }
}