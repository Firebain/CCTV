use xml::writer::{EventWriter, EmitterConfig, Result};

use super::soap_builder::SoapBuilder;
use super::writer_owner::WriterOwner;

type Bytes = Vec<u8>;

pub struct ProbeBuilder<'a> {
    writer: EventWriter<Bytes>,
    device_type: &'a str, 
    uuid: &'a str
}

impl<'a> ProbeBuilder<'a> {
    pub fn new(device_type: &'a str, uuid: &'a str) -> Self {
        let writer = EmitterConfig::new()
            .perform_indent(true)
            .create_writer(Vec::new());

        Self {
            writer,
            device_type,
            uuid
        }
    }
}

impl<'a> WriterOwner<Bytes> for ProbeBuilder<'a> {
    fn owned_writer(self) -> EventWriter<Bytes> {
        self.writer
    }

    fn get_writer(&mut self) -> &mut EventWriter<Bytes> {
        &mut self.writer
    }
}

impl<'a> SoapBuilder for ProbeBuilder<'a> {
    fn header(&mut self) -> Result<()> {
        self.new_event("s:Header")
            .ns("a", "http://schemas.xmlsoap.org/ws/2004/08/addressing")
            .write()?;

        self.new_event("a:Action")
            .attr("s:mustUnderstand", "1")
            .content("http://schemas.xmlsoap.org/ws/2005/04/discovery/Probe")
            .end()
            .write()?;

        let message_id = format!("uuid:{}", self.uuid);

        self.new_event("a:MessageID")
            .content(&message_id)
            .end()
            .write()?;

        self.new_event("a:ReplyTo")
            .write()?;

        self.new_event("a:Address")
            .content("http://schemas.xmlsoap.org/ws/2004/08/addressing/role/anonymous")
            .end()
            .write()?;

        self.end_event()?; // ReplyTo

        self.new_event("a:To")
            .attr("s:mustUnderstand", "1")
            .content("urn:schemas-xmlsoap-org:ws:2005:04:discovery")
            .end()
            .write()?;

        self.end_event()?; // Header

        Ok(())
    }

    fn body(&mut self) -> Result<()> {
        self.new_event("s:Body")
            .write()?;

        self.new_event("d:Probe")
            .ns("d", "http://schemas.xmlsoap.org/ws/2005/04/discovery")
            .write()?;

        let types = format!("dp0:{}", self.device_type);

        self.new_event("d:Types")
            .ns("dp0", "http://www.onvif.org/ver10/network/wsdl")
            .content(&types)
            .end()
            .write()?;

        self.end_event()?; // Probe

        self.end_event()?; // Body

        Ok(())
    }
}