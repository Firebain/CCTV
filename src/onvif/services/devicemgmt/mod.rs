pub mod prelude;
mod methods;

use crate::onvif::services::service::Service;

use methods::GetCapabilities;

pub struct Devicemgmt<'a> {
    xaddr: &'a String,
    username: &'a String,
    password: &'a String
}

impl<'a> Devicemgmt<'a> {
    pub fn new(xaddr: &'a String, username: &'a String, password: &'a String) -> Self {
        Self {
            xaddr,
            username,
            password
        }
    }
}

impl<'a> Service for Devicemgmt<'a> {
    fn xaddr(&self) -> &String {
        &self.xaddr
    }

    fn username(&self) -> &String {
        &self.username
    }

    fn password(&self) -> &String {
        &self.password
    }
}

impl<'a> GetCapabilities for Devicemgmt<'a> { }