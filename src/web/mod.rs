use crate::onvif;
use crate::rtsp::RtspStream;
use actix_web::{delete, get, post, web, Responder};
use image::{DynamicImage, GenericImage, ImageBuffer, ImageFormat, Rgba};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

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
pub struct CameraPool {
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

    fn delete(&mut self, order: i8) {
        match order {
            1 => self.cam1 = None,
            2 => self.cam2 = None,
            3 => self.cam3 = None,
            4 => self.cam4 = None,
            _ => panic!("Order must be in 1-4"),
        };
    }
}

pub struct State {
    pool: Arc<Mutex<CameraPool>>,
}

impl State {
    pub fn new(channel: mpsc::Sender<Vec<u8>>) -> Self {
        let pool = Arc::new(Mutex::new(CameraPool {
            cam1: None,
            cam2: None,
            cam3: None,
            cam4: None,
        }));

        let handler_pool = pool.clone();

        thread::spawn(move || Self::main_loop(channel, handler_pool));

        Self { pool }
    }

    pub fn main_loop(channel: mpsc::Sender<Vec<u8>>, pool: Arc<Mutex<CameraPool>>) {
        loop {
            let (bytes1, bytes2, bytes3, bytes4) = {
                let mut pool_lock = pool.lock().unwrap();

                (
                    pool_lock.cam1.as_mut().map(|cam| cam.stream.next()),
                    pool_lock.cam2.as_mut().map(|cam| cam.stream.next()),
                    pool_lock.cam3.as_mut().map(|cam| cam.stream.next()),
                    pool_lock.cam4.as_mut().map(|cam| cam.stream.next()),
                )
            };

            let img1 = bytes1.map(|bytes| image::load_from_memory(&bytes).unwrap());
            let img2 = bytes2.map(|bytes| image::load_from_memory(&bytes).unwrap());
            let img3 = bytes3.map(|bytes| image::load_from_memory(&bytes).unwrap());
            let img4 = bytes4.map(|bytes| image::load_from_memory(&bytes).unwrap());

            let dimensions = (1280, 720);
            // let dimensions1 = img1.dimensions();
            // let dimensions2 = img2.dimensions();
            // let dimensions3 = img3.dimensions();
            // let dimensions4 = img4.dimensions();

            // TODO: Не очень быстрый и динамичный вариант
            let mut image =
                ImageBuffer::<Rgba<u8>, Vec<u8>>::new(dimensions.0 * 2, dimensions.1 * 2);

            if let Some(img) = img1 {
                image.copy_from(&img, 0, 0).unwrap();
            }
            if let Some(img) = img2 {
                image.copy_from(&img, dimensions.0, 0).unwrap();
            }
            if let Some(img) = img3 {
                image.copy_from(&img, 0, dimensions.1).unwrap();
            }
            if let Some(img) = img4 {
                image.copy_from(&img, dimensions.0, dimensions.1).unwrap();
            }

            let mut bytes = Vec::new();

            DynamicImage::ImageRgba8(image.clone())
                .write_to(&mut bytes, ImageFormat::Jpeg)
                .unwrap();

            channel.send(bytes).unwrap();
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

#[post("/cameras")]
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

#[delete("/cameras/{id}")]
async fn delete(path: web::Path<(i8,)>, state: web::Data<State>) -> impl Responder {
    if path.0 <= 0 || path.0 > 4 {
        return "Order must be in 1-4";
    }

    let mut pool = state.pool.lock().unwrap();

    pool.delete(path.0);

    "ok"
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(discovery)
        .service(all)
        .service(add)
        .service(delete);
}
