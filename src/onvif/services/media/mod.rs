pub mod prelude;
mod methods;

use crate::onvif::services::service::Service;
use crate::onvif::soap::headers::UsernameToken;
use crate::onvif::soap::Client;

use methods::GetProfiles;
use methods::GetStreamUrl;

pub struct Media<'a> {
    xaddr: &'a String,
    wsse_client: &'a Client<UsernameToken>
}

impl<'a> Media<'a> {
    pub fn new(xaddr: &'a String, wsse_client: &'a Client<UsernameToken>) -> Self {
        Self {
            xaddr,
            wsse_client
        }
    }
}

impl<'a> Service for Media<'a> {
    fn xaddr(&self) -> &String {
        &self.xaddr
    }

    fn wsse_client(&self) -> &Client<UsernameToken> {
        &self.wsse_client
    }
}

impl<'a> GetProfiles for Media<'a> { }
impl<'a> GetStreamUrl for Media<'a> { }