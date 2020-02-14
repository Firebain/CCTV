use actix_web::{get, web, Responder, dev::HttpServiceFactory};

use crate::onvif;

#[get("/discovery")]
async fn discovery() -> impl Responder {
    web::Json(onvif::discovery().await.unwrap())
}

pub fn service() -> impl HttpServiceFactory + 'static {
    web::scope("/onvif/")
        .service(discovery)
}