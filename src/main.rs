mod soap;
mod discovery;
mod onvif;

use soap::prelude::*;
use soap::GetCapabilitiesBuilder;

fn main() {
    const XADDR: &str = "http://192.168.1.88:2000/onvif/device_service";

    // let xml = GetCapabilitiesBuilder::new("admin".to_string(), "admin1234".to_string()).build().unwrap();

    // let mut req = reqwest::Client::new().post(XADDR)
    //     .body(xml)
    //     .send()
    //     .unwrap();

    // let res = req.text().unwrap();

    // println!("{}", res);
}
