pub mod prelude;
mod methods;

use crate::onvif::services::service::Service;
use crate::onvif::soap::headers::UsernameToken;
use crate::onvif::soap::Client;

use methods::GetCapabilities;

pub struct Devicemgmt<'a> {
    xaddr: &'a String,
    wsse_client: &'a Client<UsernameToken>
}

impl<'a> Devicemgmt<'a> {
    pub fn new(xaddr: &'a String, wsse_client: &'a Client<UsernameToken>) -> Self {
        Self {
            xaddr,
            wsse_client
        }
    }
}

impl<'a> Service for Devicemgmt<'a> {
    fn xaddr(&self) -> &String {
        &self.xaddr
    }

    fn wsse_client(&self) -> &Client<UsernameToken> {
        &self.wsse_client
    }
}

impl<'a> GetCapabilities for Devicemgmt<'a> { }