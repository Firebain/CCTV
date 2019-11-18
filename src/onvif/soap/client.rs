use xml::writer::Result;

use super::event_writer::EventWriter;
use super::headers::HeaderBuilder;

pub struct Client<HB: HeaderBuilder> {
    pub header: Option<HB>,
}

impl<HB: HeaderBuilder> Client<HB> {
    pub fn new() -> Self {
        Self {
            header: None
        }
    }

    pub fn header(&mut self, header: HB) -> &mut Self {
        self.header = Some(header);

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

        if let Some(header) = &self.header {
            header.build_header(&mut writer)?;
        }

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
