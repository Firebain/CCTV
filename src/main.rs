// mod onvif;
// mod rtsp;
mod onvif;
mod soap;
mod xml;

// use std::net::{TcpListener, TcpStream};
// use std::sync::{Arc, Mutex};
// use std::thread;
// use std::time::Duration;
// use tungstenite::server::accept;
// use tungstenite::Message;
// use tungstenite::WebSocket;

// use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageFormat, Rgb, Rgba};
// use std::sync::mpsc;

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

// // use std::collections::HashMap;
// // use std::sync::Mutex;
// // use web::onvif::AuthorizedCameras;

// fn main() -> std::io::Result<()> {
//     let mut rt = tokio::runtime::Runtime::new().unwrap();

//     let local = tokio::task::LocalSet::new();

//     local.block_on(&mut rt, async move {
//         let (sender, receiver) = mpsc::channel();
//         let (sender1, receiver1) = mpsc::channel();

//         let mut camera1 = onvif::Camera::connect(
//             "http://192.168.1.88:2000/onvif/device_service".to_string(),
//             "admin".to_string(),
//             "admin1234".to_string(),
//         )
//         .await
//         .unwrap();

//         let mut camera2 = onvif::Camera::connect(
//             "http://192.168.1.88:2000/onvif/device_service".to_string(),
//             "admin".to_string(),
//             "admin1234".to_string(),
//         )
//         .await
//         .unwrap();

//         let users: Arc<Mutex<Vec<WebSocket<TcpStream>>>> = Arc::new(Mutex::new(Vec::new()));
//         let users_1 = Arc::clone(&users);
//         let users_2 = Arc::clone(&users);

//         let sender2 = sender.clone();

//         thread::spawn(move || process_image(sender2, receiver1));

//         thread::spawn(move || websocket_connections(users_1));

//         thread::spawn(move || websocket_sender(users_2, receiver));

//         println!("start camera");

//         camera1.start(1, sender1.clone());

//         camera2.start(0, sender1.clone());

//         thread::sleep(Duration::from_secs(60));

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

//         println!("stop camera");

//         camera1.stop();
//         camera2.stop();
//     });

//     Ok(())
// }

// struct Ping(i32);

// struct Connect(i32, Option<Waker>);

// impl Stream for Connect {
//     type Item = Ping;

//     fn poll_next(mut self: Pin<&mut Self>, cx: &mut StdContext<'_>) -> Poll<Option<Self::Item>> {
//         self.0 += 1;

//         println!("{} self.0", self.0);

//         if self.0 > 10 {
//             println!("end");
//             Poll::Ready(None)
//         } else if self.0 < 0 {
//             println!("pending");
//             Poll::Pending
//         } else {
//             println!("ready");
//             Poll::Ready(Some(Ping(self.0)))
//         }
//     }
// }

// use actix::prelude::*;
// use futures::stream::{Stream, StreamExt};
// use std::pin::Pin;
// use std::sync::{Arc, Mutex};
// use std::task::{Context, Poll, Waker};
// use std::thread;
// use std::time::Duration;
// // use tokio::stream::Stream;

// struct NumberTicker {
//     name: &'static str,
//     shared_state: Arc<Mutex<SharedState<u64>>>,
// }

// struct SharedState<T> {
//     last_number: ShowOnce<T>,
//     waker: Option<Waker>,
// }

// struct ShowOnce<T> {
//     showed: bool,
//     value: T,
// }

// impl<T> ShowOnce<T> {
//     fn new(value: T) -> Self {
//         Self {
//             showed: false,
//             value,
//         }
//     }
// }

// impl NumberTicker {
//     pub fn new(name: &'static str) -> Self {
//         let shared_state = Arc::new(Mutex::new(SharedState {
//             last_number: ShowOnce::new(0),
//             waker: None,
//         }));

//         let thread_shared_state = shared_state.clone();
//         thread::spawn(|| Self::main_loop(thread_shared_state));

//         Self { name, shared_state }
//     }

//     fn main_loop(thread_shared_state: Arc<Mutex<SharedState<u64>>>) {
//         loop {
//             thread::sleep(Duration::from_secs(1));

//             let mut shared_state = thread_shared_state.lock().unwrap();

//             shared_state.last_number.value += 1;
//             shared_state.last_number.showed = false;

//             if let Some(waker) = shared_state.waker.take() {
//                 waker.wake();
//             }
//         }
//     }
// }

// impl Stream for NumberTicker {
//     type Item = String;

//     fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         let mut shared_state = self.shared_state.lock().unwrap();

//         if shared_state.last_number.showed {
//             shared_state.waker = Some(ctx.waker().clone());
//             Poll::Pending
//         } else {
//             shared_state.last_number.showed = true;
//             Poll::Ready(Some(format!(
//                 "{}: {}",
//                 self.name, shared_state.last_number.value
//             )))
//         }
//     }
// }

// #[tokio::main]
// async fn main() {
//     let mut first_ticker = NumberTicker::new("First ticker").map(|val| (0, val));
//     let mut second_ticker = NumberTicker::new("Second ticker").map(|val| (1, val));
//     let mut third_ticker = NumberTicker::new("Third ticker").map(|val| (2, val));

//     let three = tokio::join!(
//         first_ticker.next(),
//         second_ticker.next(),
//         third_ticker.next()
//     );
//     //
//     // let mut number_ticker = number_ticker.merge(third_ticker);

//     println!("{:?}", three);
//     // let mut stream = Connect(-2, None);

//     // stream.next().await;
//     // stream.next().await;
//     // stream.next().await;
//     // stream.next().await;
//     // stream.next().await;
//     // let addr = MyActor.start();
//     // addr.send(Connect(-2, None)).await.unwrap();
//     // addr.send(connection).await.unwrap();

//     // println!("111");
// }

use onvif::Camera;

const XADDR: &str = "http://192.168.1.88:2000/onvif/device_service";

fn main() {
    let camera = Camera::new(
        XADDR.to_string(),
        "admin".to_string(),
        "admin1234".to_string(),
    );

    let uri = camera
        .media()
        .get_profiles()
        .get(1)
        .unwrap()
        .get_stream_url();

    println!("uri: {}", uri);
}
