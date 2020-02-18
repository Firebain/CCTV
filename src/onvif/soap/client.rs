use xml::writer::Result;

use super::headers::HeaderBuilder;
use crate::xml::EventWriter;

pub struct Client<HB: HeaderBuilder> {
    pub header: HB,
}

impl<HB: HeaderBuilder> Client<HB> {
    pub fn build<BF>(&self, body_builder: BF) -> Result<String>
    where
        BF: Fn(&mut EventWriter) -> Result<()>,
    {
        let mut writer = EventWriter::new();

        writer
            .new_event("s:Envelope")
            .ns("s", "http://www.w3.org/2003/05/soap-envelope")
            .write()?;

        self.header.build_header(&mut writer)?;

        writer.new_event("s:Body").write()?;

        body_builder(&mut writer)?;

        writer.end_event()?; // Body

        writer.end_event()?; // Envelope

        Ok(writer.into_string())
    }
}
