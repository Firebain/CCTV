use crate::onvif;
use crate::rtsp::RtspStream;
use actix_web::{get, post, web, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Mutex;

#[derive(Deserialize, Debug)]
struct NewCamera {
    name: String,
    xaddr: String,
    login: String,
    password: String,
    order: i8,
}

#[derive(Serialize)]
struct Camera {
    name: String,
    #[serde(skip)]
    stream: RtspStream,
}

#[derive(Serialize)]
struct CameraPool {
    cam1: Option<Camera>,
    cam2: Option<Camera>,
    cam3: Option<Camera>,
    cam4: Option<Camera>,
}

impl CameraPool {
    fn add(&mut self, order: i8, name: String, uri: String) {
        let camera = Some(Camera {
            name,
            stream: RtspStream::start(uri),
        });

        match order {
            1 => self.cam1 = camera,
            2 => self.cam2 = camera,
            3 => self.cam3 = camera,
            4 => self.cam4 = camera,
            _ => panic!("Order must be in 1-4"),
        };
    }
}

pub struct State {
    pool: Mutex<CameraPool>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            pool: Mutex::new(CameraPool {
                cam1: None,
                cam2: None,
                cam3: None,
                cam4: None,
            }),
        }
    }
}

#[get("/discovery")]
async fn discovery() -> impl Responder {
    let matches = onvif::discovery();

    web::Json(matches)
}

#[get("/cameras")]
async fn all(state: web::Data<State>) -> impl Responder {
    let pool = state.pool.lock().unwrap();

    let cameras = json!({
        "cam1": pool.cam1,
        "cam2": pool.cam2,
        "cam3": pool.cam3,
        "cam4": pool.cam4
    });

    web::Json(cameras)
}

#[post("/cameras/add")]
async fn add(json: web::Json<NewCamera>, state: web::Data<State>) -> impl Responder {
    let data = json.into_inner();

    if data.order <= 0 || data.order > 4 {
        return "Order must be in 1-4";
    }

    let uri = onvif::OnvifDevice::new(data.xaddr, data.login, data.password)
        .media()
        .await
        .get_profiles()
        .await[1] // change fixed profile later
        .get_stream_url()
        .await;

    let mut pool = state.pool.lock().unwrap();

    pool.add(data.order, data.name, uri);

    "ok"
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(discovery).service(all).service(add);
}
