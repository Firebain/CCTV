use uuid::Uuid;
use xml::writer::Result;

use super::HeaderBuilder;
use crate::xml::EventWriter;

pub struct Probe {
    uuid: Uuid,
}

impl Probe {
    pub fn new(uuid: Uuid) -> Self {
        Self { uuid }
    }
}

impl HeaderBuilder for Probe {
    fn build_header(&self, writer: &mut EventWriter) -> Result<()> {
        writer
            .new_event("s:Header")
            .ns("a", "http://schemas.xmlsoap.org/ws/2004/08/addressing")
            .write()?;

        writer
            .new_event("a:Action")
            .attr("s:mustUnderstand", "1")
            .content("http://schemas.xmlsoap.org/ws/2005/04/discovery/Probe")
            .end()?;

        let message_id = format!("uuid:{}", self.uuid);

        writer
            .new_event("a:MessageID")
            .content(&message_id)
            .end()?;

        writer.new_event("a:ReplyTo").write()?;

        writer
            .new_event("a:Address")
            .content("http://schemas.xmlsoap.org/ws/2004/08/addressing/role/anonymous")
            .end()?;

        writer.end_event()?; // ReplyTo

        writer
            .new_event("a:To")
            .attr("s:mustUnderstand", "1")
            .content("urn:schemas-xmlsoap-org:ws:2005:04:discovery")
            .end()?;

        writer.end_event()?; // Header

        Ok(())
    }
}
