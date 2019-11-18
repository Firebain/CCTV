use super::soap::Client;

pub struct Camera { 
    xaddr: String,
    username: String,
    password: String
}

impl Camera {
    pub fn new(xaddr: String, username: String, password: String) -> Self {
        Self::get_capabilities(&xaddr);

        Self {
            xaddr,
            username,
            password
        }
    }

    fn get_capabilities(xaddr: &String) {
        let client = Client::new();

        let xml = client.build(|writer| {
            writer.new_event("ns0:GetCapabilities")
                .ns("ns0", "http://www.onvif.org/ver10/device/wsdl")
                .end()
                .write()?;

            Ok(())
        });

        // let mut req = reqwest::Client::new().post(XADDR)
        //     .body(xml)
        //     .send()
        //     .unwrap();
    
        // let res = req.text().unwrap();
    
        println!("{}", xml);
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