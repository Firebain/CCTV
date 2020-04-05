use super::services::Media;

pub struct Profile<'a> {
    token: String,
    media: &'a Media<'a>,
}

impl<'a> Profile<'a> {
    pub fn new(token: String, media: &'a Media<'a>) -> Self {
        Self { token, media }
    }

    pub async fn get_stream_url(&self) -> String {
        self.media.get_stream_url(&self.token).await
    }
}
