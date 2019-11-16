use xml::writer::Result;

use crate::soap::soap_builder::SoapBuilderCore;
use chrono::prelude::*;
use rand::prelude::*;
use base64::encode;

pub trait MethodBuilder: SoapBuilderCore {
    fn username(&self) -> &String;

    fn password(&self) -> &String;

    fn header(&mut self) -> Result<()> {
        let (password, nonce, date) = self.compute_wsse_fields();

        self.new_event("s:Header").write()?;

        self.new_event("wsse:Security")
            .ns(
                "wsse",
                "http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd",
            )
            .write()?;

        self.new_event("wsse:UsernameToken").write()?;

        self.new_event("wsse:Username")
            .content("admin")
            .end()
            .write()?;

        self.new_event("wsse:Password")
            .attr("Type", "http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordDigest")
            .content(&password)
            .end()
            .write()?;

        self.new_event("wsse:Nonce")
            .attr("EncodingType", "http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-soap-message-security-1.0#Base64Binary")
            .content(&nonce)
            .end()
            .write()?;

        self.new_event("wsu:Created")
            .ns("wsu", "http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd")
            .content(&date)
            .end()
            .write()?;

        self.end_event()?; // UsernameToken

        self.end_event()?; // Security

        self.end_event()?; // Header

        Ok(())
    }

    fn compute_wsse_fields(&self) -> (String, String, String) {
        let utc: DateTime<Utc> = Utc::now();

        let date = utc.to_rfc3339_opts(SecondsFormat::Secs, false);

        let mut rng = rand::thread_rng();
        let nonce: [u8; 16] = rng.gen();

        let mut buffer = Vec::new();
        buffer.append(&mut nonce.to_vec());
        buffer.append(&mut date.as_bytes().to_vec());
        buffer.append(&mut self.password().as_bytes().to_vec());

        let mut hashier = sha1::Sha1::new();
        hashier.update(&buffer[..]);

        let password_hash = hashier.digest().bytes();

        (encode(&password_hash), encode(&nonce), date)
    }
}
