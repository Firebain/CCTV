use super::services::Devicemgmt;
use super::services::Media;
use crate::soap::{headers::UsernameToken, Client};

pub struct OnvifDevice {
    xaddr: String,
    wsse_client: Client<UsernameToken>,
}

impl OnvifDevice {
    pub fn new(xaddr: String, username: String, password: String) -> Self {
        let wsse_client = Client {
            header: UsernameToken::new(username, password),
        };

        Self { xaddr, wsse_client }
    }

    pub fn devicemgmt(&self) -> Devicemgmt {
        Devicemgmt::new(&self.xaddr, &self.wsse_client)
    }

    pub fn media(&self) -> Media<'_> {
        let capabilities = self.devicemgmt().get_capabilities();
        let media_xaddr = capabilities.get("media").unwrap();

        Media::new(media_xaddr.clone(), &self.wsse_client)
    }
}
