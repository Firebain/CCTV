use reqwest::Result;

use super::services::devicemgmt::Capabilities;
use super::services::prelude::*;
use super::services::Devicemgmt;
use super::services::Media;
use super::soap::headers::UsernameToken;
use super::soap::Client;

pub struct Camera {
    xaddr: String,
    wsse_client: Client<UsernameToken>,
    capabilities: Capabilities,
}

impl Camera {
    pub async fn new(xaddr: String, username: String, password: String) -> Result<Self> {
        let wsse_client = Client {
            header: UsernameToken::new(username, password),
        };

        let devicemgmt = Devicemgmt::new(&xaddr, &wsse_client);

        let capabilities = devicemgmt.get_capabilities().await?;

        Ok(Self {
            xaddr,
            wsse_client,
            capabilities,
        })
    }

    pub fn xaddr(&self) -> &str {
        &self.xaddr
    }

    #[allow(dead_code)]
    pub fn devicemgmt(&self) -> Devicemgmt {
        Devicemgmt::new(&self.xaddr, &self.wsse_client)
    }

    pub fn media(&self) -> Media {
        Media::new(&self.capabilities.media(), &self.wsse_client)
    }
}
