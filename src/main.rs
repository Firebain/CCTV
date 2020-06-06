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

use std::sync::mpsc;

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

use actix_web::{web::Data, App, HttpServer};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let (sender, receiver) = mpsc::channel();

    let users: Arc<Mutex<Vec<WebSocket<TcpStream>>>> = Arc::new(Mutex::new(Vec::new()));

    let users_1 = Arc::clone(&users);
    let users_2 = Arc::clone(&users);

    thread::spawn(move || websocket_connections(users_1));

    thread::spawn(move || websocket_sender(users_2, receiver));

    let state = Data::new(State::new(sender));

    HttpServer::new(move || App::new().app_data(state.clone()).configure(config))
        .bind("127.0.0.1:8080")?
        .run()
        .await

    // let mut stream1 = RtspStream::start(uri.clone());
    // let mut stream2 = RtspStream::start(uri.clone());
    // let mut stream3 = RtspStream::start(uri.clone());
    // let mut stream4 = RtspStream::start(uri.clone());
    // loop {
    //     let bytes = concat_streams(&mut stream1, &mut stream2, &mut stream3, &mut stream4);

    //     sender.send(bytes).unwrap();
    // }
}
