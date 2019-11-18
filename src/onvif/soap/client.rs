use xml::writer::Result;

use super::event_writer::EventWriter;
use super::headers::Header;

pub struct Client {
    pub header: Header,
}

impl Client {
    pub fn new() -> Self {
        Self {
            header: Header::None
        }
    }

    pub fn header(&mut self, header: Header) -> &mut Self {
        self.header = header;

        self
    }

    fn try_build<BF>(&self, body: BF) -> Result<String>
    where
        BF: Fn(&mut EventWriter) -> Result<()>,
    {
        let mut writer = EventWriter::new();

        writer
            .new_event("s:Envelope")
            .ns("s", "http://www.w3.org/2003/05/soap-envelope")
            .write()?;

        self.header.build(&mut writer)?;

        writer.new_event("s:Body").write()?;

        body(&mut writer)?;

        writer.end_event()?; // Body

        writer.end_event()?; // Envelope

        Ok(writer.into_string())
    }

    pub fn build<BF>(&self, body_builder: BF) -> String
    where
        BF: Fn(&mut EventWriter) -> Result<()>,
    {
        self.try_build(body_builder)
            .expect("Error while building xml")
    }
}
