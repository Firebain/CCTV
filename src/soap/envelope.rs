use serde::Deserialize;

#[derive(Deserialize)]
pub struct Envelope<T> {
    #[serde(rename = "Body", bound(deserialize = "T: Deserialize<'de>"))]
    pub body: T,
}
