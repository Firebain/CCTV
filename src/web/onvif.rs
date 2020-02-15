use std::sync::Mutex;
use std::collections::HashMap;
use serde::Deserialize;
use uuid::Uuid;
use actix_web::{get, post, delete, web, Responder, http::StatusCode};

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

// #[get("/test")]
// async fn test() -> impl Responder {
//     "asd"
// }

#[get("/cameras")]
async fn get_cameras(shared_data: web::Data<AuthorizedCameras>) -> impl Responder {
    let cameras = match shared_data.cameras.lock() {
        Ok(cameras) => cameras,
        Err(err) => return Response::err(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    };

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
            let mut cameras = match shared_data.cameras.lock() {
                Ok(cameras) => cameras,
                Err(err) => return Response::err(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            };

            let uuid = Uuid::new_v4();

            cameras.insert(uuid, camera);

            Response::ok(uuid.to_string())
        }
        Err(_) => Response::err(StatusCode::FORBIDDEN, "Some data is incorrect".to_string())
    }
}

#[delete("/cameras/{id}")]
async fn delete_camera(shared_data: web::Data<AuthorizedCameras>, id: web::Path<String>) -> impl Responder {
    let mut cameras = match shared_data.cameras.lock() {
        Ok(cameras) => cameras,
        Err(err) => return Response::err(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    };

    let result = match &Uuid::parse_str(&id.to_string()) {
        Ok(uuid) => cameras.remove(uuid),
        Err(err) => return Response::err(StatusCode::BAD_REQUEST, format!("id parsing error: {}", err))
    };
    
    match result {
        Some(_) => Response::ok("Camera was deleted"),
        None => Response::err(StatusCode::NOT_FOUND, "Camera not found".to_string())
    }
}

#[get("/discovery")]
async fn discovery() -> impl Responder {
    Response::decide(onvif::discovery())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/onvif/")
            // .service(test)
            .service(get_cameras)
            .service(create_camera)
            .service(delete_camera)
            .service(discovery)
    );
}