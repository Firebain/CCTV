use std::sync::Mutex;
use std::collections::HashMap;
use serde::Deserialize;
use uuid::Uuid;
use actix_web::{get, post, web, Responder, HttpResponse};

use crate::onvif;

pub struct AuthorizedCameras {
    pub cameras: Mutex<HashMap<Uuid, onvif::Camera>>
}

#[derive(Deserialize, Debug)]
struct CameraInfo {
    xaddr: String,
    login: String,
    password: String
}

#[get("/cameras")]
async fn get_cameras(shared_data: web::Data<AuthorizedCameras>) -> impl Responder {
    let cameras = shared_data.cameras.lock().unwrap();

    let pretty_formated: HashMap<String, String> = (*cameras).iter()
        .map(|(key, value)| (key.to_string(), value.xaddr().to_string()))
        .collect();

    web::Json(pretty_formated)
}

#[post("/cameras")]
async fn create_camera(shared_data: web::Data<AuthorizedCameras>, camera_info: web::Json<CameraInfo>) -> impl Responder {
    let camera = onvif::Camera::new(
        camera_info.xaddr.clone(),
        camera_info.login.clone(),
        camera_info.password.clone()
    ).await;

    match camera {
        Ok(camera) => {
            let mut cameras = shared_data.cameras.lock().unwrap();

            cameras.insert(Uuid::new_v4(), camera);

            HttpResponse::Ok()
                .content_type("plain/text")
                .body("Ok")
        }
        Err(_) => {
            HttpResponse::Forbidden()
                .content_type("plain/text")
                .body("Something is incorrect")
        }
    }
}

#[get("/discovery")]
async fn discovery() -> impl Responder {
    web::Json(onvif::discovery().unwrap())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/onvif/")
            .service(get_cameras)
            .service(create_camera)
            .service(discovery)
    );
}