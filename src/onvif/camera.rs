use reqwest::Result;

use super::services::Devicemgmt;
use super::services::prelude::*;
use super::soap::headers::UsernameToken;
use super::soap::Client;

pub struct Camera { 
    xaddr: String
}

impl Camera {
    pub fn new(xaddr: String, username: String, password: String) -> Result<Self> {
        let wsse_client = Client {
            header: UsernameToken::new(username, password),
        };
        
        let devicemgmt = Devicemgmt::new(&xaddr, &wsse_client);

        let capabilities = devicemgmt.get_capabilities()?;

        println!("{}", capabilities.media());

        Ok(Self {
            xaddr
        })
    }
}

// self.new_event("s:Body").write()?;

// self.new_event("ns0:GetCapabilities")
//     .ns("ns0", "http://www.onvif.org/ver10/device/wsdl")
//     .end()
//     .write()?;

// self.end_event()?; // Body

// Ok(())

// -----------------------------------------

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