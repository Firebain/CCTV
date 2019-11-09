use xml::writer::{EventWriter, XmlEvent};

pub struct EventBuilder<'a> {
    writer: &'a mut EventWriter<Vec<u8>>,
    name: &'a str,
    ns: Option<(&'a str, &'a str)>,
    attr: Option<(&'a str, &'a str)>,
    content: Option<&'a str>,
    end: bool
}

impl<'a> EventBuilder<'a> {
    pub fn new(writer: &'a mut EventWriter<Vec<u8>>, name: &'a str) -> Self {
        Self {
            writer,
            name,
            ns: None,
            attr: None,
            content: None,
            end: false
        }
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
        let element = XmlEvent::start_element(self.name);

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

        if self.end {
            self.writer.write(XmlEvent::end_element()).unwrap();
        }
    }
}