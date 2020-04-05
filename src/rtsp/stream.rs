use super::client::Client;
use super::rtp::sequence::{RTPSequence, RTPSequenceStatus};
use futures::stream::Stream as FuturesStream;
use std::net::{SocketAddr, UdpSocket};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;

struct SharedState {
    last_value: ShowOnce,
    waker: Option<Waker>,
}

struct ShowOnce {
    showed: bool,
    value: Vec<u8>,
}

impl ShowOnce {
    fn empty() -> Self {
        Self {
            showed: true,
            value: Vec::new(),
        }
    }
}

pub struct Stream {
    // rtsp: Client,
    // session: String,
    shared_state: Arc<Mutex<SharedState>>,
}

impl Stream {
    pub fn start(uri: String) -> Self {
        let mut rtsp = Client::connect(uri);

        rtsp.describe();

        let free_socket_addr = SocketAddr::from(([0, 0, 0, 0], 0));
        let main_socket = UdpSocket::bind(free_socket_addr).unwrap();

        let main_socket_port = main_socket.local_addr().unwrap().port();

        let shared_state = Arc::new(Mutex::new(SharedState {
            last_value: ShowOnce::empty(),
            waker: None,
        }));

        let thread_shared_state = shared_state.clone();
        thread::spawn(|| Self::main_loop(main_socket, thread_shared_state));

        let session = rtsp.setup(main_socket_port, main_socket_port + 1);

        rtsp.play(&session);

        Self {
            // rtsp,
            // session,
            shared_state,
        }
    }

    // pub fn stop(mut self) {
    //     self.rtsp.teardown(&self.session);
    // }

    fn main_loop(socket: UdpSocket, thread_shared_state: Arc<Mutex<SharedState>>) {
        let mut rtp_sequence = RTPSequence::new();

        loop {
            let mut buf = [0; 65_535];

            let amt = socket.recv(&mut buf).unwrap();

            let buf = &buf[..amt];

            match rtp_sequence.push(buf) {
                Ok(status) => {
                    if let RTPSequenceStatus::LastPacket(data) = status {
                        let mut shared_state = thread_shared_state.lock().unwrap();

                        shared_state.last_value.value = data;
                        shared_state.last_value.showed = false;

                        if let Some(waker) = shared_state.waker.take() {
                            waker.wake();
                        }
                    }
                }
                Err(err) => println!("{}", err),
            }
        }
    }
}

impl FuturesStream for Stream {
    type Item = Vec<u8>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut shared_state = self.shared_state.lock().unwrap();

        if shared_state.last_value.showed {
            shared_state.waker = Some(ctx.waker().clone());
            Poll::Pending
        } else {
            shared_state.last_value.showed = true;
            Poll::Ready(Some(shared_state.last_value.value.clone()))
        }
    }
}
