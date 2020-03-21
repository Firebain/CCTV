use crate::onvif::soap::headers::UsernameToken;
use crate::onvif::soap::Client;
use crate::onvif::soap::Envelope;

use std::collections::HashMap;
use serde::Deserialize;

use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    UnexpectedError(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedError(err) => write!(f, "Unexpected err: {}", err),
        }
    }
}

impl error::Error for Error {}


#[derive(Deserialize)]
pub struct GetCapabilitiesBody {
    #[serde(rename = "GetCapabilitiesResponse")]
    get_capabilities_response: GetCapabilitiesResponse
}

#[derive(Deserialize)]
pub struct GetCapabilitiesResponse {
    #[serde(rename = "Capabilities")]
    capabilities: Capabilities
}

#[derive(Deserialize)]
pub struct Capabilities {
    #[serde(rename = "Media")]
    media: Service
}

#[derive(Deserialize)]
pub struct Service {
    #[serde(rename = "XAddr")]
    xaddr: String
}


pub struct Devicemgmt<'a> {
    xaddr: &'a String,
    wsse_client: &'a Client<UsernameToken>,
}

impl<'a> Devicemgmt<'a> {
    pub fn new(xaddr: &'a String, wsse_client: &'a Client<UsernameToken>) -> Self {
        Self { xaddr, wsse_client }
    }

    pub async fn get_capabilities(&self) -> Result<HashMap<String, String>, Error> {
        let message = self
            .wsse_client
            .build(|writer| {
                writer
                    .new_event("ns0:GetCapabilities")
                    .ns("ns0", "http://www.onvif.org/ver10/device/wsdl")
                    .end()?;
        
                Ok(())
            })
            .map_err(|_| Error::UnexpectedError("Xml builder error"))?;

        let response = reqwest::Client::new()
            .post(self.xaddr)
            .body(message)
            .send()
            .await
            .map_err(|_| Error::UnexpectedError("Send err"))?
            .text()
            .await
            .map_err(|_| Error::UnexpectedError("Response parsing err"))?;

        let data: Envelope<GetCapabilitiesBody> = serde_xml_rs::from_str(&response)
            .map_err(|_| Error::UnexpectedError("Parsing err"))?;

        let mut capabilities = HashMap::new();

        capabilities.insert("media".to_string(), data.body.get_capabilities_response.capabilities.media.xaddr);

        Ok(capabilities)
    }
}