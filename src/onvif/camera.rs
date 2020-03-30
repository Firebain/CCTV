use std::sync::mpsc;

use super::services::{Devicemgmt, Media};
use super::soap::headers::UsernameToken;
use super::soap::Client;

use crate::rtsp::client::RTSPClient;
use crate::rtsp::rtp::sequence::{RTPSequence, RTPSequenceError, RTPSequenceStatus};

use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::{error, fmt};

fn video_handler(number: usize, socket: UdpSocket, sender: mpsc::Sender<(usize, Vec<u8>)>) {
    println!("video handler start");
    let mut rtp_sequence = RTPSequence::new();

    loop {
        let mut buf = [0; 65_535];

        let amt = socket.recv(&mut buf).unwrap();

        let buf = &buf[..amt];

        match rtp_sequence.push(buf) {
            Ok(status) => {
                if let RTPSequenceStatus::LastPacket(data) = status {
                    match sender.send((number, data)) {
                        Ok(_) => (),
                        Err(err) => panic!(format!("{}", err)),
                    };

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

    pub fn start(&mut self, number: usize, sender: mpsc::Sender<(usize, Vec<u8>)>) {
        self.rtsp.describe().unwrap();

        let free_socket_addr = SocketAddr::from(([0, 0, 0, 0], 0));
        let main_socket = UdpSocket::bind(free_socket_addr).expect("Could not bind to udp socket");
        let next_socket_addr =
            SocketAddr::from(([0, 0, 0, 0], main_socket.local_addr().unwrap().port() + 1));
        let second_socket =
            UdpSocket::bind(next_socket_addr).expect("Could not bind to udp socket");

        let cloned_main_socket = main_socket.try_clone().unwrap();
        println!("video handler spawn");
        thread::spawn(move || video_handler(number, cloned_main_socket, sender));

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
