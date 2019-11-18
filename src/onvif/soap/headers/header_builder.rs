use xml::writer::Result;

use crate::onvif::soap::event_writer::EventWriter;

pub trait HeaderBuilder {
    fn build_header(&self, writer: &mut EventWriter) -> Result<()>;
}