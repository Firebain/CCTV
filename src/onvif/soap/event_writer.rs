use xml::writer::{EmitterConfig, EventWriter as DefaultEventWriter, Result};

use super::EventBuilder;

pub struct EventWriter(DefaultEventWriter<Vec<u8>>);

impl<'a> EventWriter {
    pub fn new() -> Self {
        let writer = EmitterConfig::new()
            .perform_indent(true)
            .create_writer(Vec::new());

        Self(writer)
    }

    pub fn new_event(&'a mut self, name: &'a str) -> EventBuilder<'a, Vec<u8>> {
        EventBuilder::new(&mut self.0).name(name)
    }

    pub fn end_event(&'a mut self) -> Result<()> {
        EventBuilder::new(&mut self.0).end().write()
    }

    pub fn to_string(self) -> String {
        let buffer = self.0.into_inner();

        String::from_utf8(buffer).expect("Xml contains non utf-8 characters")
    }
}
