use base64::encode;
use chrono::prelude::*;
use rand::prelude::*;

use super::HeaderBuilder;
use crate::xml::EventWriter;

pub struct UsernameToken {
    username: String,
    password: String,
}

impl UsernameToken {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    fn compute_wsse_fields(&self) -> (String, String, String) {
        let utc: DateTime<Utc> = Utc::now();

        let date = utc.to_rfc3339_opts(SecondsFormat::Secs, false);

        let mut rng = rand::thread_rng();
        let nonce: [u8; 16] = rng.gen();

        let mut buffer = Vec::new();
        buffer.append(&mut nonce.to_vec());
        buffer.append(&mut date.as_bytes().to_vec());
        buffer.append(&mut self.password.as_bytes().to_vec());

        let mut hashier = sha1::Sha1::new();
        hashier.update(&buffer[..]);

        let password_hash = hashier.digest().bytes();

        (encode(&password_hash), encode(&nonce), date)
    }
}

impl HeaderBuilder for UsernameToken {
    fn build_header(&self, writer: &mut EventWriter) {
        let (password, nonce, date) = self.compute_wsse_fields();

        writer.new_event("s:Header").write();

        writer
            .new_event("wsse:Security")
            .ns(
                "wsse",
                "http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd",
            )
            .write();

        writer.new_event("wsse:UsernameToken").write();

        writer
            .new_event("wsse:Username")
            .content(&self.username)
            .end();

        writer.new_event("wsse:Password")
            .attr("Type", "http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordDigest")
            .content(&password)
            .end();

        writer.new_event("wsse:Nonce")
            .attr("EncodingType", "http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-soap-message-security-1.0#Base64Binary")
            .content(&nonce)
            .end();

        writer.new_event("wsu:Created")
            .ns("wsu", "http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd")
            .content(&date)
            .end();

        writer.end_event(); // UsernameToken

        writer.end_event(); // Security

        writer.end_event(); // Header
    }
}
