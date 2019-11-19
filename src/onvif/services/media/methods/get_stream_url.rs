use reqwest::Result as RequestResult;
use xml::reader::{EventReader, Result as ParserResult, XmlEvent};

use crate::onvif::services::service::Service;
use crate::onvif::soap::headers::UsernameToken;
use crate::onvif::soap::Client;

pub trait GetStreamUrl: Service {
    fn get_stream_url(&self, profile_token: &String) -> RequestResult<String> {
        let message = create_message(self.wsse_client(), profile_token);

        let res = send_request(self.xaddr(), message)?;

        Ok(parse_response(res).expect("Unexpected error while parsing response"))
    }
}

fn create_message(wsse_client: &Client<UsernameToken>, profile_token: &String) -> String {
    wsse_client.build(|writer| {
        writer
            .new_event("ns0:GetStreamUri")
            .ns("ns0", "http://www.onvif.org/ver10/media/wsdl")
            .write()?;

        writer
            .new_event("ns0:StreamSetup")
            .ns("ns1", "http://www.onvif.org/ver10/schema")
            .write()?;

        writer
            .new_event("ns1:Stream")
            .content("RTP-Unicast")
            .end()
            .write()?;

        writer.new_event("ns1:Transport").write()?;

        writer
            .new_event("ns1:Protocol")
            .content("RTSP")
            .end()
            .write()?;

        writer.end_event()?; // Transport

        writer.end_event()?; // StreamSetup

        writer
            .new_event("ns0:ProfileToken")
            .content(profile_token)
            .end()
            .write()?;

        writer.end_event()?; // GetStreamUri

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

fn parse_response(response: String) -> ParserResult<String> {
    let mut parser = EventReader::from_str(&response);

    loop {
        let event = parser.next()?;

        match event {
            XmlEvent::StartElement { name, .. } => {
                if let "Uri" = name.local_name.as_str() {
                    let event = parser.next()?;

                    if let XmlEvent::Characters(uri) = event {
                        return Ok(uri);
                    }
                }
            }
            XmlEvent::EndDocument => break,
            _ => {}
        }
    }

    panic!("Uri not found");
}
