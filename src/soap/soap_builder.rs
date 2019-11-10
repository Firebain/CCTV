use xml::writer::{EventWriter, Result as WriterResult};

use crate::soap::SoapBuilderError;
use crate::soap::event_builder::EventBuilder;

pub type Bytes = Vec<u8>;

pub trait SoapBuilderCore: Sized {
    fn owned_writer(self) -> EventWriter<Bytes>;
    
    fn get_writer(&mut self) -> &mut EventWriter<Bytes>;

    fn new_event<'a>(&'a mut self, name: &'a str) -> EventBuilder<'a, Bytes> {
        EventBuilder::new(self.get_writer()).name(name)
    }

    fn end_event(&mut self) -> WriterResult<()> {
        EventBuilder::new(self.get_writer()).end().write()
    }

    fn header(&mut self) -> WriterResult<()>;

    fn body(&mut self) -> WriterResult<()>;
}

pub trait SoapBuilder: SoapBuilderCore {
    fn build(mut self) -> Result<String, SoapBuilderError> {
        self.new_event("s:Envelope")
            .ns("s", "http://www.w3.org/2003/05/soap-envelope")
            .write()?;

        self.header()?;

        self.body()?;

        self.end_event()?; // Envelope
        
        Ok(String::from_utf8(self.owned_writer().into_inner())?)
    }
}