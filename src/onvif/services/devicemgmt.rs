use crate::soap::{headers::UsernameToken, Envelope, SoapClient};

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct GetCapabilitiesBody {
    #[serde(rename = "GetCapabilitiesResponse")]
    get_capabilities_response: GetCapabilitiesResponse,
}

#[derive(Deserialize)]
pub struct GetCapabilitiesResponse {
    #[serde(rename = "Capabilities")]
    capabilities: Capabilities,
}

#[derive(Deserialize)]
pub struct Capabilities {
    #[serde(rename = "Media")]
    media: Service,
}

#[derive(Deserialize)]
pub struct Service {
    #[serde(rename = "XAddr")]
    xaddr: String,
}

pub struct Devicemgmt<'a> {
    xaddr: &'a str,
    wsse_client: &'a SoapClient<UsernameToken>,
}

impl<'a> Devicemgmt<'a> {
    pub fn new(xaddr: &'a str, wsse_client: &'a SoapClient<UsernameToken>) -> Self {
        Self { xaddr, wsse_client }
    }

    pub fn get_capabilities(&self) -> HashMap<String, String> {
        let message = self.wsse_client.build(|writer| {
            writer
                .new_event("ns0:GetCapabilities")
                .ns("ns0", "http://www.onvif.org/ver10/device/wsdl")
                .end();
        });

        let response = reqwest::blocking::Client::new()
            .post(self.xaddr)
            .body(message)
            .send()
            .unwrap()
            .text()
            .unwrap();

        let data: Envelope<GetCapabilitiesBody> = serde_xml_rs::from_str(&response).unwrap();

        let mut capabilities = HashMap::new();

        capabilities.insert(
            "media".to_string(),
            data.body.get_capabilities_response.capabilities.media.xaddr,
        );

        capabilities
    }
}
