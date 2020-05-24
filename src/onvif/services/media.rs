use crate::soap::headers::UsernameToken;
use crate::soap::Client;
use crate::soap::Envelope;

use crate::onvif::Profile;

use serde::Deserialize;

#[derive(Deserialize)]
struct GetProfilesBody {
    #[serde(rename = "GetProfilesResponse")]
    get_profiles_response: GetProfilesResponse,
}

#[derive(Deserialize)]
struct GetProfilesResponse {
    #[serde(rename = "Profiles")]
    profiles: Vec<ProfileModel>,
}

#[derive(Deserialize, Debug)]
pub struct ProfileModel {
    token: String,
}

#[derive(Deserialize)]
struct GetStreamBody {
    #[serde(rename = "GetStreamUriResponse")]
    get_stream_url_response: GetStreamUriResponse,
}

#[derive(Deserialize)]
struct GetStreamUriResponse {
    #[serde(rename = "MediaUri")]
    media_uri: MediaUri,
}

#[derive(Deserialize, Debug)]
pub struct MediaUri {
    #[serde(rename = "Uri")]
    pub uri: String,
}

pub struct Media<'a> {
    xaddr: String,
    wsse_client: &'a Client<UsernameToken>,
}

impl<'a> Media<'a> {
    pub fn new(xaddr: String, wsse_client: &'a Client<UsernameToken>) -> Self {
        Self { xaddr, wsse_client }
    }

    pub fn get_profiles(&self) -> Vec<Profile<'_>> {
        let message = self.wsse_client.build(|writer| {
            writer
                .new_event("ns0:GetProfiles")
                .ns("ns0", "http://www.onvif.org/ver10/media/wsdl")
                .end();
        });

        let response = reqwest::blocking::Client::new()
            .post(&self.xaddr)
            .body(message)
            .send()
            .unwrap()
            .text()
            .unwrap();

        let data: Envelope<GetProfilesBody> = serde_xml_rs::from_str(&response).unwrap();

        data.body
            .get_profiles_response
            .profiles
            .into_iter()
            .map(|profile_model| Profile::new(profile_model.token, &self))
            .collect()
    }

    pub fn get_stream_url(&self, profile_token: &str) -> String {
        let message = self.wsse_client.build(|writer| {
            writer
                .new_event("ns0:GetStreamUri")
                .ns("ns0", "http://www.onvif.org/ver10/media/wsdl")
                .write();
            writer
                .new_event("ns0:StreamSetup")
                .ns("ns1", "http://www.onvif.org/ver10/schema")
                .write();
            writer.new_event("ns1:Stream").content("RTP-Unicast").end();
            writer.new_event("ns1:Transport").write();

            writer.new_event("ns1:Protocol").content("RTSP").end();
            writer.end_event(); // Transport
            writer.end_event(); // StreamSetup
            writer
                .new_event("ns0:ProfileToken")
                .content(profile_token)
                .end();
            writer.end_event(); // GetStreamUri
        });

        let response = reqwest::blocking::Client::new()
            .post(&self.xaddr)
            .body(message)
            .send()
            .unwrap()
            .text()
            .unwrap();

        let data: Envelope<GetStreamBody> = serde_xml_rs::from_str(&response).unwrap();

        data.body.get_stream_url_response.media_uri.uri
    }
}
