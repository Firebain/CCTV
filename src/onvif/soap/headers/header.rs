use xml::writer::Result;

use crate::onvif::soap::event_writer::EventWriter;
use super::probe::Probe;

pub enum Header {
    Probe(Probe),
    None
}

impl Header {
    pub fn build(&self, writer: &mut EventWriter) -> Result<()> {
        match self {
            Header::Probe(probe) => probe.build(writer),
            Header::None => Ok(())
        }
    }
}