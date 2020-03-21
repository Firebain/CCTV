use tokio::sync::mpsc;

use super::services::{Devicemgmt, Media};
use super::soap::headers::UsernameToken;
use super::soap::Client;

use image::{DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Rgb};

use crate::rtsp::client::RTSPClient;
use crate::rtsp::rtp::sequence::{RTPSequence, RTPSequenceError, RTPSequenceStatus};

use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::{error, fmt};

use tokio::runtime::Runtime;

async fn process_image(data: Vec<u8>, mut tx: mpsc::Sender<Vec<u8>>) {
    let img = image::load_from_memory(&data).unwrap();

    let dimensions = img.dimensions();

    let container = img.to_bytes();
    let pixels: Vec<&[u8]> = container.chunks(3).collect();
    let rows: Vec<&[&[u8]]> = pixels.chunks(dimensions.0 as usize).collect();

    let mut new_image = Vec::new();

    for row in rows {
        let mut new_row = Vec::new();
        for rgb in row {
            new_row.extend_from_slice(rgb);
        }

        new_image.append(&mut new_row.repeat(2));
    }

    let new_image: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_vec(dimensions.0 * 2, dimensions.1, new_image).unwrap();
    let new_image = DynamicImage::ImageRgb8(new_image);

    let mut bytes = Vec::new();
    new_image.write_to(&mut bytes, ImageFormat::Jpeg).unwrap();

    tx.send(bytes).await.unwrap();
}

fn video_handler(socket: UdpSocket, sender: mpsc::Sender<Vec<u8>>) {
    println!("video handler start");
    let mut rtp_sequence = RTPSequence::new();

    let rt = Runtime::new().unwrap();

    loop {
        let mut buf = [0; 65_535];

        let amt = socket.recv(&mut buf).unwrap();

        let buf = &buf[..amt];

        match rtp_sequence.push(buf) {
            Ok(status) => {
                if let RTPSequenceStatus::LastPacket(data) = status {
                    let tx = sender.clone();

                    rt.spawn(process_image(data, tx));

                    rtp_sequence.clean();
                }
            }
            Err(err) => match err {
                RTPSequenceError::PackageLost => rtp_sequence.clean(),
                RTPSequenceError::HeaderMissing => rtp_sequence.clean(),
                _ => panic!("{}", err),
            },
        }
    }
}

#[derive(Debug)]
pub enum Error {
    UnexpectedError(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedError(err) => write!(f, "Unexpected err: {}", err),
        }
    }
}

impl error::Error for Error {}

pub struct Camera {
    rtsp: RTSPClient,
    session: String,
    thread: Option<thread::JoinHandle<()>>,
}

impl Camera {
    pub async fn connect(xaddr: String, username: String, password: String) -> Result<Self, Error> {
        let wsse_client = Client {
            header: UsernameToken::new(username, password),
        };

        let devicemgmt = Devicemgmt::new(&xaddr, &wsse_client);

        let capabilities = devicemgmt
            .get_capabilities()
            .await
            .map_err(|_| Error::UnexpectedError("Unexpected err"))?;

        let media_xaddr = capabilities
            .get("media")
            .ok_or(Error::UnexpectedError("Media xaddr is missing"))?;

        let media = Media::new(&media_xaddr, &wsse_client);

        let profiles = media
            .get_profiles()
            .await
            .map_err(|_| Error::UnexpectedError("Unexpected err"))?;

        let uri = media
            .get_stream_url(&profiles[1].token) // TODO: Remove this hardcoded index
            .await
            .map_err(|_| Error::UnexpectedError("Unexpected err"))?;

        let rtsp = RTSPClient::connect(uri).await.unwrap();

        Ok(Self {
            rtsp,
            thread: None,
            session: String::new(), // TODO: Стремно
        })
    }

    pub fn start(&mut self, sender: mpsc::Sender<Vec<u8>>) {
        self.rtsp.describe().unwrap();

        let free_socket_addr = SocketAddr::from(([0, 0, 0, 0], 0));
        let main_socket = UdpSocket::bind(free_socket_addr).expect("Could not bind to udp socket");
        let next_socket_addr =
            SocketAddr::from(([0, 0, 0, 0], main_socket.local_addr().unwrap().port() + 1));
        let second_socket =
            UdpSocket::bind(next_socket_addr).expect("Could not bind to udp socket");

        let cloned_main_socket = main_socket.try_clone().unwrap();
        println!("video handler spawn");
        self.thread = Some(thread::spawn(|| video_handler(cloned_main_socket, sender)));

        self.session = self
            .rtsp
            .setup(
                main_socket.local_addr().unwrap().port(),
                second_socket.local_addr().unwrap().port(),
            )
            .unwrap();

        println!("session: {}", self.session);

        self.rtsp.play(&self.session).unwrap();
    }

    pub fn stop(&mut self) {
        self.rtsp.teardown(&self.session).unwrap();
    }
}
