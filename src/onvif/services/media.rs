use crate::onvif::soap::headers::UsernameToken;
use crate::onvif::soap::Client;
use crate::onvif::soap::Envelope;

use serde::Deserialize;

use std::{error, fmt};

#[derive(Deserialize)]
struct GetProfilesBody {
    #[serde(rename = "GetProfilesResponse")]
    get_profiles_response: GetProfilesResponse
}

#[derive(Deserialize)]
struct GetProfilesResponse {
    #[serde(rename = "Profiles")]
    profiles: Vec<Profile>
}

#[derive(Deserialize, Debug)]
pub struct Profile {
    pub token: String
}

#[derive(Deserialize)]
struct GetStreamBody {
    #[serde(rename = "GetStreamUriResponse")]
    get_stream_url_response: GetStreamUriResponse
}

#[derive(Deserialize)]
struct GetStreamUriResponse {
    #[serde(rename = "MediaUri")]
    media_uri: MediaUri
}

#[derive(Deserialize, Debug)]
pub struct MediaUri {
    #[serde(rename = "Uri")]
    pub uri: String
}

#[derive(Debug)]
pub enum Error {
    UnexpectedError(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedError(err) => write!(f, "Unexpected err: {}", err),
        }
    }
}

impl error::Error for Error {}

pub struct Media<'a> {
    xaddr: &'a String,
    wsse_client: &'a Client<UsernameToken>,
}

impl<'a> Media<'a> {
    pub fn new(xaddr: &'a String, wsse_client: &'a Client<UsernameToken>) -> Self {
        Self { xaddr, wsse_client }
    }

    pub async fn get_profiles(&self) -> Result<Vec<Profile>, Error> {
        let message = self
            .wsse_client
            .build(|writer| {
                writer
                    .new_event("ns0:GetProfiles")
                    .ns("ns0", "http://www.onvif.org/ver10/media/wsdl")
                    .end()?;
        
                Ok(())
            })
            .map_err(|_| Error::UnexpectedError("Xml builder error"))?;

        let response = reqwest::Client::new()
            .post(self.xaddr)
            .body(message)
            .send()
            .await
            .map_err(|_| Error::UnexpectedError("Send err"))?
            .text()
            .await
            .map_err(|_| Error::UnexpectedError("Response parsing err"))?;

        let data: Envelope<GetProfilesBody> = serde_xml_rs::from_str(&response)
            .map_err(|_| Error::UnexpectedError("Parsing err"))?;

        Ok(data.body.get_profiles_response.profiles)
    }

    pub async fn get_stream_url(&self, profile_token: &str) -> Result<String, Error> {
        let message = self
            .wsse_client
            .build(|writer| {
                writer
                    .new_event("ns0:GetStreamUri")
                    .ns("ns0", "http://www.onvif.org/ver10/media/wsdl")
                    .write()?;
        
                writer
                    .new_event("ns0:StreamSetup")
                    .ns("ns1", "http://www.onvif.org/ver10/schema")
                    .write()?;
        
                writer
                    .new_event("ns1:Stream")
                    .content("RTP-Unicast")
                    .end()?;
        
                writer.new_event("ns1:Transport").write()?;
        
                writer
                    .new_event("ns1:Protocol")
                    .content("RTSP")
                    .end()?;
        
                writer.end_event()?; // Transport
        
                writer.end_event()?; // StreamSetup
        
                writer
                    .new_event("ns0:ProfileToken")
                    .content(profile_token)
                    .end()?;
        
                writer.end_event()?; // GetStreamUri
        
                Ok(())
            })
            .map_err(|_| Error::UnexpectedError("Xml builder error"))?;

        let response = reqwest::Client::new()
            .post(self.xaddr)
            .body(message)
            .send()
            .await
            .map_err(|_| Error::UnexpectedError("Send err"))?
            .text()
            .await
            .map_err(|_| Error::UnexpectedError("Response parsing err"))?;

        let data: Envelope<GetStreamBody> = serde_xml_rs::from_str(&response)
            .map_err(|_| Error::UnexpectedError("Parsing err"))?;

        Ok(data.body.get_stream_url_response.media_uri.uri)
    }
}