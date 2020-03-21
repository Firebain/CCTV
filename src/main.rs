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

use tokio::sync::mpsc;

fn websocket_connections(users: Arc<Mutex<Vec<WebSocket<TcpStream>>>>) {
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();

    for stream in server.incoming() {
        println!("new user!");

        let mut users = users.lock().unwrap();

        users.push(accept(stream.unwrap()).unwrap());
    }
}

async fn websocket_sender(
    users: Arc<Mutex<Vec<WebSocket<TcpStream>>>>,
    mut rx: mpsc::Receiver<Vec<u8>>,
) {
    println!("sending image started");
    while let Some(image) = rx.recv().await {
        println!("sending image");

        let mut users = users.lock().unwrap();

        for user in (*users).iter_mut() {
            user.write_message(Message::Binary(image.clone())).unwrap();
        }
    }
    println!("sending image stoped");
}

use actix_web::{guard, web as router, App, HttpResponse, HttpServer};
// use std::collections::HashMap;
// use std::sync::Mutex;
// use web::onvif::AuthorizedCameras;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let (sender, receiver) = mpsc::channel(100);

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

    thread::spawn(move || websocket_connections(users_1));

    tokio::spawn(websocket_sender(users_2, receiver));

    println!("start camera");

    camera.start(sender);

    thread::sleep(Duration::from_secs(30));

    println!("stop camera");

    camera.stop();

    // let cameras_data = router::Data::new(AuthorizedCameras {
    //     cameras: Mutex::new(HashMap::new())
    // });

    HttpServer::new(move || {
        App::new()
            // .app_data(cameras_data.clone())
            .configure(web::onvif::config)
            // 404
            .default_service(
                router::resource("")
                    .route(router::get().to(HttpResponse::NotFound))
                    .route(
                        router::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
