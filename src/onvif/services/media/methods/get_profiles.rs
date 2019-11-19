use reqwest::Result as RequestResult;
use xml::reader::{EventReader, Result as ParserResult, XmlEvent};

use crate::onvif::services::service::Service;
use crate::onvif::soap::headers::UsernameToken;
use crate::onvif::soap::Client;

pub struct Profile {
    token: String
}

impl Profile {
    pub fn token(&self) -> &String {
        &self.token
    }
}

pub trait GetProfiles: Service {
    fn get_profiles(&self) -> RequestResult<Vec<Profile>> {
        let message = create_message(self.wsse_client());

        let res = send_request(self.xaddr(), message)?;

        Ok(parse_response(res).expect("Unexpected error while parsing response"))
    }
}

fn create_message(wsse_client: &Client<UsernameToken>) -> String {
    wsse_client.build(|writer| {
        writer
            .new_event("ns0:GetProfiles")
            .ns("ns0", "http://www.onvif.org/ver10/media/wsdl")
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

fn parse_response(response: String) -> ParserResult<Vec<Profile>> {
    let mut parser = EventReader::from_str(&response);

    let mut profiles = Vec::new();

    loop {
        let event = parser.next()?;

        match event {
            XmlEvent::StartElement { name, attributes, .. } => {
                if let "Profiles" = name.local_name.as_str() {
                    for attribute in attributes {
                        if let "token" = attribute.name.local_name.as_str() {
                            profiles.push(Profile {
                                token: attribute.value
                            });
                        }
                    }
                }
            }
            XmlEvent::EndDocument => break,
            _ => {}
        }
    }

    Ok(profiles)
}