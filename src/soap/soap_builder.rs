use xml::writer::XmlEvent;

use super::event_builder::EventBuilder;
use super::writer_owner::WriterOwner;

pub trait SoapBuilder: WriterOwner + Sized {
    fn new_event<'a>(&'a mut self, name: &'a str) -> EventBuilder<'a> {
        EventBuilder::new(self.get_writer(), name)
    }

    fn end_event(&mut self) {
        self.get_writer().write(XmlEvent::end_element()).unwrap();
    }

    fn header(&mut self);

    fn body(&mut self);

    fn build(mut self) -> String {
        self.new_event("s:Envelope")
            .ns("s", "http://www.w3.org/2003/05/soap-envelope")
            .write();

        self.header();

        self.body();

        self.end_event(); // Envelope
        
        String::from_utf8(self.borrow_writer().into_inner()).unwrap()
    }
}