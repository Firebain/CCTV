use xml::writer::{EventWriter, Result};

use crate::soap::method_builder::MethodBuilder;
use crate::soap::soap_builder::{Bytes, SoapBuilder, SoapBuilderCore};

pub struct GetCapabilitiesBuilder {
    username: String,
    password: String,
    writer: EventWriter<Bytes>,
}

impl GetCapabilitiesBuilder {
    pub fn new(username: String, password: String) -> Self {
        Self {
            username,
            password,
            writer: Self::create_writer(),
        }
    }
}

impl SoapBuilder for GetCapabilitiesBuilder {}

impl MethodBuilder for GetCapabilitiesBuilder {
    fn username(&self) -> &String {
        &self.username
    }

    fn password(&self) -> &String {
        &self.password
    }
}

impl SoapBuilderCore for GetCapabilitiesBuilder {
    fn owned_writer(self) -> EventWriter<Bytes> {
        self.writer
    }

    fn get_writer(&mut self) -> &mut EventWriter<Bytes> {
        &mut self.writer
    }

    fn header(&mut self) -> Result<()> {
        MethodBuilder::header(self)
    }

    fn body(&mut self) -> Result<()> {
        self.new_event("s:Body").write()?;

        self.new_event("ns0:GetCapabilities")
            .ns("ns0", "http://www.onvif.org/ver10/device/wsdl")
            .end()
            .write()?;

        self.end_event()?; // Body

        Ok(())
        // self.new_event("s:Body").write()?;

        // self.new_event("ns0:GetStreamUrl")
        //     .ns("ns0", "http://www.onvif.org/ver10/media/wsdl")
        //     .write()?;

        // self.new_event("ns0:StreamSetup")
        //     .ns("ns1", "http://www.onvif.org/ver10/schema")
        //     .write()?;

        // self.new_event("ns1:Stream")
        //     .content("RTP-Unicast")
        //     .end()
        //     .write()?;

        // self.new_event("ns1:Transport").write()?;

        // self.new_event("ns1:Protocol")
        //     .content("UDP")
        //     .end()
        //     .write()?;

        // self.end_event()?; // Transport

        // self.end_event()?; // StreamSetup

        // self.new_event("ns0:ProfileToken")
        //     .content("profile0")
        //     .end()
        //     .write()?;

        // self.end_event()?; // GetStreamUrl

        // self.end_event()?; // Body

        // Ok(())
    }
}
