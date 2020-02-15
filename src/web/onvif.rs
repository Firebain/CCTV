use std::sync::Mutex;
use std::collections::HashMap;
use serde::Deserialize;
use uuid::Uuid;
use actix_web::{get, post, web, Responder, http::StatusCode};

use crate::onvif;
use super::response::Response;

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

    Response::ok(pretty_formated)
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

            let uuid = Uuid::new_v4();

            cameras.insert(uuid, camera);

            Response::ok(uuid.to_string())
        }
        Err(_) => Response::err(StatusCode::FORBIDDEN, "Some data is incorrect".to_string())
    }
}

#[get("/discovery")]
async fn discovery() -> impl Responder {
    Response::decide(onvif::discovery())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/onvif/")
            .service(get_cameras)
            .service(create_camera)
            .service(discovery)
    );
}