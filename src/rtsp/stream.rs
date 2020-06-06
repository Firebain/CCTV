use super::client::RtspClient;
use super::rtp::sequence::{RTPSequence, RTPSequenceStatus};
// use futures::stream::Stream as FuturesStream;
use std::net::{SocketAddr, UdpSocket};
// use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex};
// use std::task::{Context, Poll, Waker};
use std::thread;

struct SharedState {
    showed: bool,
    value: Vec<u8>,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            showed: true,
            value: Vec::new(),
        }
    }
}

pub struct RtspStream {
    rtsp: RtspClient,
    session: String,
    sync_pair: Arc<(Mutex<SharedState>, Condvar)>,
}

impl RtspStream {
    pub fn start(uri: String) -> Self {
        let mut rtsp = RtspClient::connect(uri);

        rtsp.describe();

        let free_socket_addr = SocketAddr::from(([0, 0, 0, 0], 0));
        let main_socket = UdpSocket::bind(free_socket_addr).unwrap();

        let main_socket_port = main_socket.local_addr().unwrap().port();

        let sync_pair = Arc::new((Mutex::new(SharedState::default()), Condvar::new()));

        let thread_sync_pair = sync_pair.clone();
        thread::spawn(|| Self::main_loop(main_socket, thread_sync_pair));

        let session = rtsp.setup(main_socket_port, main_socket_port + 1);

        rtsp.play(&session);

        Self {
            rtsp,
            session,
            sync_pair,
        }
    }

    pub fn next(&mut self) -> Vec<u8> {
        let (shared_state_lock, cvar) = &*self.sync_pair;

        let mut shared_state = cvar
            .wait_while(shared_state_lock.lock().unwrap(), |state| state.showed)
            .unwrap();

        shared_state.showed = true;

        shared_state.value.clone()
    }

    // pub fn stop(mut self) {
    //     self.rtsp.teardown(&self.session);
    // }

    fn main_loop(socket: UdpSocket, thread_sync_pair: Arc<(Mutex<SharedState>, Condvar)>) {
        let mut rtp_sequence = RTPSequence::new();

        loop {
            let mut buf = [0; 65_535];

            let amt = socket.recv(&mut buf).unwrap();

            let buf = &buf[..amt];

            match rtp_sequence.push(buf) {
                Ok(status) => {
                    if let RTPSequenceStatus::LastPacket(data) = status {
                        let (shared_state_lock, cvar) = &*thread_sync_pair;

                        let mut shared_state = shared_state_lock.lock().unwrap();

                        shared_state.value = data;
                        shared_state.showed = false;

                        cvar.notify_one();
                    }
                }
                Err(_err) => {
                    // println!("{}", _err)
                }
            }
        }
    }
}

impl Drop for RtspStream {
    fn drop(&mut self) {
        println!("Stream ended");

        self.rtsp.teardown(&self.session);
    }
}

// impl FuturesStream for Stream {
//     type Item = Vec<u8>;

//     fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         let mut shared_state = self.shared_state.lock().unwrap();

//         if shared_state.last_value.showed {
//             shared_state.waker = Some(ctx.waker().clone());
//             Poll::Pending
//         } else {
//             shared_state.last_value.showed = true;
//             Poll::Ready(Some(shared_state.last_value.value.clone()))
//         }
//     }
// }
