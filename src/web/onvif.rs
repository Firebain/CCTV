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

#[get("/cameras")]
async fn get_cameras(shared_data: web::Data<AuthorizedCameras>) -> Result<impl Responder, Response<String>> {
    let cameras = shared_data.cameras.lock()
        .map_err(|err| Response::err(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let pretty_formated: HashMap<String, String> = (*cameras).iter()
        .map(|(key, value)| (key.to_string(), value.xaddr().to_string()))
        .collect();

    Ok(Response::ok(pretty_formated))
}

#[post("/cameras")]
async fn create_camera(shared_data: web::Data<AuthorizedCameras>, camera_info: web::Json<CameraInfo>) -> Result<impl Responder, Response<String>> {
    let camera = onvif::Camera::new(
        camera_info.xaddr.clone(),
        camera_info.login.clone(),
        camera_info.password.clone()
    )
    .await
    .map_err(|_| Response::err(StatusCode::FORBIDDEN, "Some data is incorrect".to_string()))?;

    let mut cameras = shared_data.cameras.lock()
        .map_err(|err| Response::err(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?; 

    let uuid = Uuid::new_v4();

    cameras.insert(uuid, camera);

    Ok(Response::ok(uuid.to_string()))
}

#[delete("/cameras/{id}")]
async fn delete_camera(shared_data: web::Data<AuthorizedCameras>, id: web::Path<String>) -> Result<impl Responder, Response<String>> {
    let mut cameras = shared_data.cameras.lock()
        .map_err(|err| Response::err(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let uuid = &Uuid::parse_str(&id.to_string())
        .map_err(|err| Response::err(StatusCode::BAD_REQUEST, format!("id parsing error: {}", err)))?;

    cameras.remove(uuid)
        .ok_or(Response::err(StatusCode::NOT_FOUND, "Camera not found".to_string()))?;
    
    Ok(Response::ok("Camera was deleted"))
}

#[get("/discovery")]
async fn discovery() -> Result<impl Responder, Response<String>> {
    let probe_matches = onvif::discovery()
        .map_err(|err| Response::err(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(Response::ok(probe_matches))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/onvif/")
            .service(get_cameras)
            .service(create_camera)
            .service(delete_camera)
            .service(discovery)
    );
}