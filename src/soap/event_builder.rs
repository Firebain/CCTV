use xml::writer::{EventWriter, XmlEvent};
use std::io::Write;

pub struct EventBuilder<'a, W: Write> {
    writer: &'a mut EventWriter<W>,
    name: Option<&'a str>,
    ns: Option<(&'a str, &'a str)>,
    attr: Option<(&'a str, &'a str)>,
    content: Option<&'a str>,
    end: bool
}

impl<'a, W: Write> EventBuilder<'a, W> {
    pub fn new(writer: &'a mut EventWriter<W>) -> Self {
        Self {
            writer,
            name: None,
            ns: None,
            attr: None,
            content: None,
            end: false
        }
    }

    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);

        self
    }

    pub fn ns(mut self, prefix: &'a str, url: &'a str) -> Self {
        self.ns = Some((prefix, url));

        self
    }

    pub fn attr(mut self, name: &'a str, value: &'a str) -> Self {
        self.attr = Some((name, value));

        self
    }

    pub fn content(mut self, text: &'a str) -> Self {
        self.content = Some(text);

        self
    }

    pub fn end(mut self) -> Self {
        self.end = true;

        self
    }

    pub fn write(self) {
        if let Some(name) = self.name {
            let element = XmlEvent::start_element(name);

            let element = match self.ns {
                Some(ns) => element.ns(ns.0, ns.1),
                None => element
            };

            let element = match self.attr {
                Some(attr) => element.attr(attr.0, attr.1),
                None => element
            };

            self.writer.write(element).unwrap();

            if let Some(content) = self.content {
                self.writer.write(XmlEvent::characters(content)).unwrap();
            }
        }

        if self.end {
            self.writer.write(XmlEvent::end_element()).unwrap();
        }
    }
}