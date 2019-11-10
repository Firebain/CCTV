use xml::writer::{EventWriter, EmitterConfig, Result};

use crate::soap::soap_builder::{Bytes, SoapBuilderCore, SoapBuilder};

pub struct MethodBuilder {
    writer: EventWriter<Bytes>
}

impl MethodBuilder {
    pub fn new() -> Self {
        let writer = EmitterConfig::new()
            .perform_indent(true)
            .create_writer(Vec::new());

        Self {
            writer
        }
    }
}

impl SoapBuilder for MethodBuilder {}

impl SoapBuilderCore for MethodBuilder {
    fn owned_writer(self) -> EventWriter<Bytes> {
        self.writer
    }

    fn get_writer(&mut self) -> &mut EventWriter<Bytes> {
        &mut self.writer
    }

    fn header(&mut self) -> Result<()> {
        self.new_event("s:Header")
            .write()?;

        self.new_event("wsse:Security")
            .ns("wsse", "http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd")
            .write()?;

        self.new_event("wsse:UsernameToken")
            .write()?;

        self.new_event("wsse:Username")
            .content("admin")
            .end()
            .write()?;

        self.new_event("wsse:Password")
            .attr("Type", "http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordDigest")
            .content("50GvEYUNaseUQd1n/iQXP6U1DpU=")
            .end()
            .write()?;

        self.new_event("wsse:Nonce")
            .attr("EncodingType", "http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-soap-message-security-1.0#Base64Binary")
            .content("0XGmlYMjM8ciJRHWykTUcA==")
            .end()
            .write()?;

        self.new_event("wsu:Created")
            .ns("wsu", "http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd")
            .content("2019-11-09T07:42:41+00:00")
            .end()
            .write()?;

        self.end_event()?; // UsernameToken

        self.end_event()?; // Security

        self.end_event()?; // Header

        Ok(())
    }

    fn body(&mut self) -> Result<()> {
        self.new_event("s:Body")
            .write()?;

        self.new_event("ns0:GetStreamUrl")
            .ns("ns0", "http://www.onvif.org/ver10/media/wsdl")
            .write()?;

        self.new_event("ns0:StreamSetup")
            .ns("ns1", "http://www.onvif.org/ver10/schema")
            .write()?;

        self.new_event("ns1:Stream")
            .content("RTP-Unicast")
            .end()
            .write()?;

        self.new_event("ns1:Transport")
            .write()?;

        self.new_event("ns1:Protocol")
            .content("UDP")
            .end()
            .write()?;

        self.end_event()?; // Transport

        self.end_event()?; // StreamSetup

        self.new_event("ns0:ProfileToken")
            .content("profile0")
            .end()
            .write()?;

        self.end_event()?; // GetStreamUrl

        self.end_event()?; // Body

        Ok(())
    }
}