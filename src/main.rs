mod onvif;
mod rtsp;
mod web;
mod xml;

use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tungstenite::server::accept;
use tungstenite::Message;
use tungstenite::WebSocket;

use image::{DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Rgb};
use std::sync::mpsc;

// Вместо вызываемой функции сделать так, что бы сюда шли все картинки
// со всех камер формата (i8, Vec<u8>), где i8 - порядковый номер камеры,
// а Vec<u8> - самара картинка. Кешировать все картинки в памяти и заменять
// картинками из потока
fn process_image(sender: mpsc::Sender<Vec<u8>>, receiver: mpsc::Receiver<(i8, Vec<u8>)>) {
    println!("process image");
    let (_, image) = receiver.recv().unwrap();

    sender.send(image).unwrap();
    // let img1 = (0, image::load_from_memory(&data).unwrap());
    // let img2 = (1, image::load_from_memory(&data).unwrap());
    // let img3 = (2, image::load_from_memory(&data).unwrap());
    // let img4 = (3, image::load_from_memory(&data).unwrap());

    // let dimensions1 = img1.1.dimensions();
    // let dimensions2 = img2.1.dimensions();
    // let dimensions3 = img3.1.dimensions();
    // let dimensions4 = img4.1.dimensions();

    // let container1 = img1.1.to_bytes();
    // let container2 = img2.1.to_bytes();
    // let container3 = img3.1.to_bytes();
    // let container4 = img4.1.to_bytes();

    // let pixels1: Vec<&[u8]> = container1.chunks(3).collect();
    // let pixels2: Vec<&[u8]> = container2.chunks(3).collect();
    // let pixels3: Vec<&[u8]> = container3.chunks(3).collect();
    // let pixels4: Vec<&[u8]> = container4.chunks(3).collect();

    // let rows1: Vec<&[&[u8]]> = pixels1.chunks(dimensions1.0 as usize).collect();
    // let rows2: Vec<&[&[u8]]> = pixels2.chunks(dimensions2.0 as usize).collect();
    // let rows3: Vec<&[&[u8]]> = pixels3.chunks(dimensions3.0 as usize).collect();
    // let rows4: Vec<&[&[u8]]> = pixels4.chunks(dimensions4.0 as usize).collect();

    // let mut new_image = Vec::new();

    // for (index, row) in rows1.into_iter().enumerate() {
    //     let mut new_row = Vec::new();
    //     for rgb in row {
    //         new_row.extend_from_slice(rgb);
    //     }

    //     for rgb in rows3[index] {
    //         new_row.extend_from_slice(rgb);
    //     }

    //     new_image.append(&mut new_row);
    // }

    // for (index, row) in rows2.into_iter().enumerate() {
    //     let mut new_row = Vec::new();
    //     for rgb in row {
    //         new_row.extend_from_slice(rgb);
    //     }

    //     for rgb in rows4[index] {
    //         new_row.extend_from_slice(rgb);
    //     }

    //     new_image.append(&mut new_row);
    // }

    // let new_image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_vec(
    //     dimensions2.0 + dimensions4.0,
    //     dimensions1.1 + dimensions3.1,
    //     new_image,
    // )
    // .unwrap();
    // let new_image = DynamicImage::ImageRgb8(new_image);

    // let mut bytes = Vec::new();
    // new_image.write_to(&mut bytes, ImageFormat::Jpeg).unwrap();

    // sender.send(bytes).unwrap();
}

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

use actix_web::{guard, web as router, App, HttpResponse, HttpServer};
// use std::collections::HashMap;
// use std::sync::Mutex;
// use web::onvif::AuthorizedCameras;

fn main() -> std::io::Result<()> {
    let mut rt = tokio::runtime::Runtime::new().unwrap();

    let local = tokio::task::LocalSet::new();

    local.block_on(&mut rt, async move {
        let (sender, receiver) = mpsc::channel();
        let (sender1, receiver1) = mpsc::channel();

        let mut camera = onvif::Camera::connect(
            "http://192.168.1.88:2000/onvif/device_service".to_string(),
            "admin".to_string(),
            "admin1234".to_string(),
        )
        .await
        .unwrap();

        let users: Arc<Mutex<Vec<WebSocket<TcpStream>>>> = Arc::new(Mutex::new(Vec::new()));
        let users_1 = Arc::clone(&users);
        let users_2 = Arc::clone(&users);

        let sender2 = sender.clone();

        thread::spawn(move || process_image(sender2, receiver1));

        thread::spawn(move || websocket_connections(users_1));

        thread::spawn(move || websocket_sender(users_2, receiver));

        println!("start camera");

        camera.start(0, sender1);

        thread::sleep(Duration::from_secs(60));

        // tokio::task::spawn_local(async move {
        //     let local = tokio::task::LocalSet::new();

        //     // let cameras_data = router::Data::new(AuthorizedCameras {
        //     //     cameras: Mutex::new(HashMap::new()),
        //     // });

        //     let sys = actix_rt::System::run_in_tokio("server", &local);
        //     HttpServer::new(move || {
        //         App::new()
        //             // .app_data(cameras_data.clone())
        //             .configure(web::onvif::config)
        //             // 404
        //             .default_service(
        //                 router::resource("")
        //                     .route(router::get().to(HttpResponse::NotFound))
        //                     .route(
        //                         router::route()
        //                             .guard(guard::Not(guard::Get()))
        //                             .to(HttpResponse::MethodNotAllowed),
        //                     ),
        //             )
        //     })
        //     .bind("127.0.0.1:8080")
        //     .unwrap()
        //     .run()
        //     .await
        //     .unwrap();

        //     sys.await.unwrap();
        // })
        // .await
        // .unwrap();

        println!("stop camera");

        camera.stop();
    });

    Ok(())
}
