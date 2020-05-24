// mod onvif;
// mod rtsp;
mod onvif;
mod rtsp;
mod soap;
mod xml;

use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use tungstenite::server::accept;
use tungstenite::Message;
use tungstenite::WebSocket;

use futures::stream::StreamExt;

use onvif::OnvifDevice;
use rtsp::Stream;

const XADDR: &str = "http://192.168.1.88:2000/onvif/device_service";

// use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageFormat, Rgb, Rgba};
use std::sync::mpsc;

// fn process_image(sender: mpsc::Sender<Vec<u8>>, receiver: mpsc::Receiver<(usize, Vec<u8>)>) {
//     let mut cache: Option<ImageBuffer<Rgba<u8>, Vec<u8>>> = None;

//     loop {
//         let (number, image) = receiver.recv().unwrap();

//         let img = image::load_from_memory(&image).unwrap();

//         let dimensions = img.dimensions();

//         let mut image = cache.unwrap_or(ImageBuffer::<Rgba<u8>, Vec<u8>>::new(
//             dimensions.0 * 2,
//             dimensions.1 * 2,
//         ));

//         let (x, y) = match number {
//             0 => (0, 0),
//             1 => (dimensions.0, 0),
//             2 => (0, dimensions.1),
//             3 => (dimensions.0, dimensions.1),
//             _ => panic!("number is not in [0, 1, 2, 3]"),
//         };

//         image.copy_from(&img, x, y).unwrap();

//         let mut bytes = Vec::new();

//         DynamicImage::ImageRgba8(image.clone())
//             .write_to(&mut bytes, ImageFormat::Jpeg)
//             .unwrap();

//         cache = Some(image);

//         sender.send(bytes).unwrap();
//     }
// }

fn websocket_connections(users: Arc<Mutex<Vec<WebSocket<TcpStream>>>>) {
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();

    for stream in server.incoming() {
        println!("new user!");

        let mut users = users.lock().unwrap();

        users.push(accept(stream.unwrap()).unwrap());
    }
}

fn websocket_sender(users: Arc<Mutex<Vec<WebSocket<TcpStream>>>>, rx: mpsc::Receiver<Vec<u8>>) {
    println!("sending image started");
    loop {
        let image = match rx.recv() {
            Ok(image) => image,
            Err(err) => panic!(format!("{}", err)),
        };
        let mut users = users.lock().unwrap();

        for user in (*users).iter_mut() {
            user.write_message(Message::Binary(image.clone())).unwrap();
        }
    }
}

#[tokio::main]
async fn main() {
    let camera = OnvifDevice::new(
        XADDR.to_string(),
        "admin".to_string(),
        "admin1234".to_string(),
    );

    let uri = camera.media().get_profiles()[0].get_stream_url();

    let (sender, receiver) = mpsc::channel();

    let users: Arc<Mutex<Vec<WebSocket<TcpStream>>>> = Arc::new(Mutex::new(Vec::new()));
    let users_1 = Arc::clone(&users);
    let users_2 = Arc::clone(&users);

    thread::spawn(move || websocket_connections(users_1));

    thread::spawn(move || websocket_sender(users_2, receiver));

    let mut stream = Stream::start(uri);
    loop {
        let frame = stream.next().await.unwrap();

        sender.send(frame).unwrap();
    }

    // println!("{:?}", stream.next().await.unwrap());
    // println!("{:?}", stream.next().await.unwrap());

    // thread::sleep(Duration::from_secs(5));

    // stream.stop();

    // println!("sleep");

    // thread::sleep(Duration::from_secs(1));
}
