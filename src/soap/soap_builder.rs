use xml::writer::Result as WriterResult;

use super::SoapBuilderError;
use super::event_builder::EventBuilder;
use super::writer_owner::WriterOwner;

type Bytes = Vec<u8>;

pub trait SoapBuilder: WriterOwner<Bytes> + Sized {
    fn new_event<'a>(&'a mut self, name: &'a str) -> EventBuilder<'a, Bytes> {
        EventBuilder::new(self.get_writer()).name(name)
    }

    fn end_event(&mut self) -> WriterResult<()> {
        EventBuilder::new(self.get_writer()).end().write()
    }

    fn header(&mut self) -> WriterResult<()>;

    fn body(&mut self) -> WriterResult<()>;

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