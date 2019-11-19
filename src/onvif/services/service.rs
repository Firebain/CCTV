use crate::onvif::soap::headers::UsernameToken;
use crate::onvif::soap::Client;

pub trait Service {
    fn xaddr(&self) -> &String;

    fn wsse_client(&self) -> &Client<UsernameToken>;
}
