mod onvif;
// mod rtsp;

// use rtsp::client::RTSPClient;
// use rtsp::rtp::sequence::{RTPSequence, RTPSequenceError, RTPSequenceStatus};

use onvif::prelude::*;
use onvif::Camera;

// // use std::io;

// use std::net::{TcpListener, TcpStream};
// use image::{DynamicImage, ImageBuffer, GenericImageView, Rgb, ImageFormat};
// use std::net::{SocketAddr, UdpSocket};
// use std::thread;
// use std::time::Duration;
// use tungstenite::server::accept;
// use tungstenite::WebSocket;
// use std::sync::{Arc, Mutex};
// use tungstenite::Message;

// use tokio::sync::mpsc;

// fn websocket_connections(users: Arc<Mutex<Vec<WebSocket<TcpStream>>>>) {
//     let server = TcpListener::bind("127.0.0.1:9001").unwrap();

//     for stream in server.incoming() {
//         let mut users = users.lock().unwrap();

//         users.push(accept(stream.unwrap()).unwrap());
//     }
// }

// async fn websocket_sender(users: Arc<Mutex<Vec<WebSocket<TcpStream>>>>, mut rx: mpsc::Receiver<Vec<u8>>) {
//     while let Some(image) = rx.recv().await {
//         let mut users = users.lock().unwrap();

//         for user in (*users).iter_mut() {
//             user.write_message(Message::Binary(image.clone())).unwrap();
//         }
//     }
// }

// fn jpeg_reader(receiver: Receiver<Vec<u8>>, users: Arc<Mutex<Vec<WebSocket<TcpStream>>>>) {
    // loop {
    //     let data = receiver.recv().unwrap();

    //     let img = image::load_from_memory(&data).unwrap();

    //     let dimensions = img.dimensions();

    //     let container = img.to_bytes();
    //     let pixels: Vec<&[u8]> = container.chunks(3).collect();
    //     let rows: Vec<&[&[u8]]> = pixels.chunks(dimensions.0 as usize).collect();

    //     let mut new_image = Vec::new();

    //     for row in rows {
    //         let mut new_row = Vec::new();
    //         for rgb in row {
    //             new_row.extend_from_slice(rgb);
    //         }

    //         new_image.append(&mut new_row.repeat(2));
    //     }

    //     let new_image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_vec(dimensions.0 * 2, dimensions.1, new_image).unwrap();
    //     let new_image = DynamicImage::ImageRgb8(new_image);

    //     let mut users = users.lock().unwrap();

    //     let mut bytes = Vec::new();
    //     new_image.write_to(&mut bytes, ImageFormat::Jpeg).unwrap();

        // for user in (*users).iter_mut() {
        //     user.write_message(Message::Binary(bytes.clone())).unwrap();
        // }
    // }
// }

// async fn process_image(data: Vec<u8>, mut tx: mpsc::Sender<Vec<u8>>) {
//     let img = image::load_from_memory(&data).unwrap();

//     let dimensions = img.dimensions();

//     let container = img.to_bytes();
//     let pixels: Vec<&[u8]> = container.chunks(3).collect();
//     let rows: Vec<&[&[u8]]> = pixels.chunks(dimensions.0 as usize).collect();

//     let mut new_image = Vec::new();

//     for row in rows {
//         let mut new_row = Vec::new();
//         for rgb in row {
//             new_row.extend_from_slice(rgb);
//         }

//         new_image.append(&mut new_row.repeat(2));
//     }

//     let new_image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_vec(dimensions.0 * 2, dimensions.1, new_image).unwrap();
//     let new_image = DynamicImage::ImageRgb8(new_image);

//     let mut bytes = Vec::new();
//     new_image.write_to(&mut bytes, ImageFormat::Jpeg).unwrap();

//     tx.send(bytes).await.unwrap();
// }

// async fn video_handler(socket: UdpSocket) {
//     let mut rtp_sequence = RTPSequence::new();

//     // let (sync_sender, receiver) = sync_channel(100);
    
//     let users: Arc<Mutex<Vec<WebSocket<TcpStream>>>> = Arc::new(Mutex::new(Vec::new()));
//     let users_1 = Arc::clone(&users);
//     let users_2 = Arc::clone(&users);

//     // thread::spawn(move || jpeg_reader(receiver, users_1));

//     let (tx, rx) = mpsc::channel(100);

//     thread::spawn(move || websocket_connections(users_1));

//     tokio::spawn(websocket_sender(users_2, rx));

//     loop {
//         let mut buf = [0; 65_535];

//         let amt = socket.recv(&mut buf).unwrap();

//         let buf = &buf[..amt];

//         match rtp_sequence.push(buf) {
//             Ok(status) => {
//                 if let RTPSequenceStatus::LastPacket(data) = status {
//                     let tx = tx.clone();

//                     tokio::spawn(process_image(data, tx));
//                     // if let Err(_) = sync_sender.try_send(data) {
//                     //     println!("Buffer is full");
//                     // }

//                     rtp_sequence.clean();
//                 }
//             }
//             Err(err) => match err {
//                 RTPSequenceError::PackageLost => rtp_sequence.clean(),
//                 RTPSequenceError::HeaderMissing => rtp_sequence.clean(),
//                 _ => panic!("{}", err),
//             },
//         }
//     }
// }

// #[tokio::main]
// async fn main() {
//     let mut client = RTSPClient::connect("rtsp://192.168.1.88:554/av0_1".to_string()).unwrap();

//     client.describe().unwrap();

//     let free_socket_addr = SocketAddr::from(([0, 0, 0, 0], 0));
//     let main_socket = UdpSocket::bind(free_socket_addr).expect("Could not bind to udp socket");

//     let next_socket_addr =
//         SocketAddr::from(([0, 0, 0, 0], main_socket.local_addr().unwrap().port() + 1));
//     let second_socket = UdpSocket::bind(next_socket_addr).expect("Could not bind to udp socket");

//     let cloned_main_socket = main_socket.try_clone().unwrap();
//     let video_thread = tokio::spawn(video_handler(cloned_main_socket));

//     let cloned_second_socket = second_socket.try_clone().unwrap();
//     let control_info_thread = thread::spawn(move || loop {
//         let mut buf = [0; 32];

//         cloned_second_socket.recv(&mut buf).unwrap();

//         println!("{:?}", buf);
//     });

//     let session = client
//         .setup(
//             main_socket.local_addr().unwrap().port(),
//             second_socket.local_addr().unwrap().port(),
//         )
//         .unwrap();

//     client.play(&session).unwrap();

//     thread::sleep(Duration::from_secs(30));

//     client.teardown(&session).unwrap();

//     video_thread.await.unwrap();
//     control_info_thread.join().unwrap();

//     let mut buf = vec![];

//     loop {
//         match stream.read_to_end(&mut buf) {
//             Ok(_) => break,
//             Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
//                 break
//             }
//             Err(e) => panic!("encountered IO error: {}", e),
//         };
//     };

//     println!("{}", String::from_utf8(buf).unwrap());
// }

#[tokio::main]
async fn main() {
    const XADDR: &str = "http://192.168.1.88:2000/onvif/device_service";
    const LOGIN: &str = "admin";
    const PASS: &str = "admin1234";

    let camera = Camera::new(
        XADDR.to_string(),
        LOGIN.to_string(),
        PASS.to_string(),
    )
    .await
    .unwrap();

    let media = camera.media();

    let profiles = media.get_profiles().await.unwrap();

    let uri = media
        .get_stream_url(profiles.first().unwrap().token())
        .await
        .unwrap();

    println!("{}", uri);
}