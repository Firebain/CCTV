use reqwest::Result as RequestResult;
use xml::reader::{EventReader, Result as ParserResult, XmlEvent};

use crate::onvif::services::service::Service;
use crate::onvif::soap::headers::UsernameToken;
use crate::onvif::soap::Client;

pub struct Capabilities {
    pub media: String,
}

struct CapabilitiesBuilder {
    media: Option<String>,
}

impl CapabilitiesBuilder {
    fn new() -> Self {
        Self { media: None }
    }

    fn media(&mut self, xaddr: String) -> &mut Self {
        self.media = Some(xaddr);

        self
    }

    fn build(self) -> Capabilities {
        Capabilities {
            media: self
                .media
                .expect("Get capabilities response doesn't contains media xaddr"),
        }
    }
}

pub trait GetCapabilities: Service {
    fn get_capabilities(&self) -> RequestResult<Capabilities> {
        let message = create_message(self.username(), self.password());

        let res = send_request(self.xaddr(), message)?;
        Ok(parse_response(res).expect("Unexpected error while parsing response"))
    }
}

fn create_message(username: &String, password: &String) -> String {
    let client = Client {
        header: UsernameToken::new(username, password),
    };

    client.build(|writer| {
        writer
            .new_event("ns0:GetCapabilities")
            .ns("ns0", "http://www.onvif.org/ver10/device/wsdl")
            .end()
            .write()?;

        Ok(())
    })
}

fn send_request(xaddr: &String, message: String) -> RequestResult<String> {
    let response = reqwest::Client::new()
        .post(xaddr)
        .body(message)
        .send()?
        .text()?;

    Ok(response)
}

fn parse_response(response: String) -> ParserResult<Capabilities> {
    let mut parser = EventReader::from_str(&response);
    let mut capabilities_builder = CapabilitiesBuilder::new();

    loop {
        let event = parser.next()?;

        match event {
            XmlEvent::StartElement { name, .. } => {
                if let "Media" = name.local_name.as_str() {
                    loop {
                        let event = parser.next()?;

                        match event {
                            XmlEvent::StartElement { name, .. } => {
                                if let "XAddr" = name.local_name.as_str() {
                                    let event = parser.next()?;

                                    if let XmlEvent::Characters(xaddr) = event {
                                        capabilities_builder.media(xaddr);
                                    }
                                }
                            }
                            XmlEvent::EndElement { name } => {
                                if let "Media" = name.local_name.as_str() {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            XmlEvent::EndDocument => break,
            _ => {}
        }
    }

    Ok(capabilities_builder.build())
}
