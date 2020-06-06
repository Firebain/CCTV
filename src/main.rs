mod onvif;
mod rtsp;
mod soap;
mod web;
mod xml;

use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use web::{config, State};

use tungstenite::server::accept;
use tungstenite::Message;
use tungstenite::WebSocket;

const XADDR: &str = "http://192.168.1.88:2000/onvif/device_service";

use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageFormat, Rgba};
use std::sync::mpsc;

// fn websocket_connections(users: Arc<Mutex<Vec<WebSocket<TcpStream>>>>) {
//     let server = TcpListener::bind("127.0.0.1:9001").unwrap();

//     for stream in server.incoming() {
//         println!("new user!");

//         let mut users = users.lock().unwrap();

//         users.push(accept(stream.unwrap()).unwrap());
//     }
// }

// fn websocket_sender(users: Arc<Mutex<Vec<WebSocket<TcpStream>>>>, rx: mpsc::Receiver<Vec<u8>>) {
//     println!("sending image started");
//     loop {
//         let image = match rx.recv() {
//             Ok(image) => image,
//             Err(err) => panic!(format!("{}", err)),
//         };
//         let mut users = users.lock().unwrap();

//         for user in (*users).iter_mut() {
//             user.write_message(Message::Binary(image.clone())).unwrap();
//         }
//     }
// }

// fn concat_streams(
//     stream1: &mut RtspStream,
//     stream2: &mut RtspStream,
//     stream3: &mut RtspStream,
//     stream4: &mut RtspStream,
// ) -> Vec<u8> {
//     let bytes1 = stream1.next();
//     let bytes2 = stream2.next();
//     let bytes3 = stream3.next();
//     let bytes4 = stream4.next();

//     let img1 = image::load_from_memory(&bytes1).unwrap();
//     let img2 = image::load_from_memory(&bytes2).unwrap();
//     let img3 = image::load_from_memory(&bytes3).unwrap();
//     let img4 = image::load_from_memory(&bytes4).unwrap();

//     let dimensions1 = img1.dimensions();
//     // let dimensions2 = img2.dimensions();
//     // let dimensions3 = img3.dimensions();
//     // let dimensions4 = img4.dimensions();

//     // TODO: Не очень быстрый и динамичный вариант
//     let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(dimensions1.0 * 2, dimensions1.1 * 2);

//     image.copy_from(&img1, 0, 0).unwrap();
//     image.copy_from(&img2, dimensions1.0, 0).unwrap();
//     image.copy_from(&img3, 0, dimensions1.1).unwrap();
//     image
//         .copy_from(&img4, dimensions1.0, dimensions1.1)
//         .unwrap();

//     let mut bytes = Vec::new();

//     DynamicImage::ImageRgba8(image.clone())
//         .write_to(&mut bytes, ImageFormat::Jpeg)
//         .unwrap();

//     bytes
// }

use actix_web::{web::Data, App, HttpServer};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let state = Data::new(State::default());

    HttpServer::new(move || App::new().app_data(state.clone()).configure(config))
        .bind("127.0.0.1:8080")?
        .run()
        .await

    // let (sender, receiver) = mpsc::channel();

    // let users: Arc<Mutex<Vec<WebSocket<TcpStream>>>> = Arc::new(Mutex::new(Vec::new()));
    // let users_1 = Arc::clone(&users);
    // let users_2 = Arc::clone(&users);

    // thread::spawn(move || websocket_connections(users_1));

    // thread::spawn(move || websocket_sender(users_2, receiver));

    // let mut stream1 = RtspStream::start(uri.clone());
    // let mut stream2 = RtspStream::start(uri.clone());
    // let mut stream3 = RtspStream::start(uri.clone());
    // let mut stream4 = RtspStream::start(uri.clone());
    // loop {
    //     let bytes = concat_streams(&mut stream1, &mut stream2, &mut stream3, &mut stream4);

    //     sender.send(bytes).unwrap();
    // }
}
