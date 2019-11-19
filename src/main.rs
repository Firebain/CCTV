mod onvif;

use onvif::Camera;
use onvif::prelude::*;

fn main() {
    const XADDR: &str = "http://192.168.1.88:2000/onvif/device_service";

    let camera = Camera::new(XADDR.to_string(), "admin".to_string(), "admin1234".to_string()).unwrap();

    let media = camera.media();

    let profiles = media.get_profiles().unwrap();

    let uri = media.get_stream_url(profiles.first().unwrap().token()).unwrap();

    println!("{}", uri);
}
