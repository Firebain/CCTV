// mod onvif;
mod rtsp;

use rtsp::client::RTSPClient;
use rtsp::rtp::sequence::{RTPSequence, RTPSequenceError, RTPSequenceStatus};

// use onvif::prelude::*;
// use onvif::Camera;

// use std::io;

use std::fs::File;
use std::io::prelude::*;
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;

fn video_handler(socket: UdpSocket) {
    let mut rtp_sequence = RTPSequence::new();

    loop {
        let mut buf = [0; 65_535];

        let amt = socket.recv(&mut buf).unwrap();

        let buf = &buf[..amt];

        match rtp_sequence.push(buf) {
            Ok(status) => {
                if let RTPSequenceStatus::LastPacket(data) = status {
                    let mut file = File::create("frame.jpeg").unwrap();

                    file.write_all(&data).unwrap();

                    rtp_sequence.clean();
                }
            }
            Err(err) => match err {
                RTPSequenceError::PackageLost => rtp_sequence.clean(),
                _ => panic!("{}", err),
            },
        }
    }
}

fn main() {
    let mut client = RTSPClient::connect("rtsp://192.168.1.88:554/av0_1".to_string()).unwrap();

    client.describe().unwrap();

    let free_socket_addr = SocketAddr::from(([0, 0, 0, 0], 0));
    let main_socket = UdpSocket::bind(free_socket_addr).expect("Could not bind to udp socket");

    let next_socket_addr =
        SocketAddr::from(([0, 0, 0, 0], main_socket.local_addr().unwrap().port() + 1));
    let second_socket = UdpSocket::bind(next_socket_addr).expect("Could not bind to udp socket");

    let cloned_main_socket = main_socket.try_clone().unwrap();
    let video_thread = thread::spawn(move || video_handler(cloned_main_socket));

    let cloned_second_socket = second_socket.try_clone().unwrap();
    let control_info_thread = thread::spawn(move || loop {
        let mut buf = [0; 32];

        cloned_second_socket.recv(&mut buf).unwrap();

        println!("{:?}", buf);
    });

    let session = client
        .setup(
            main_socket.local_addr().unwrap().port(),
            second_socket.local_addr().unwrap().port(),
        )
        .unwrap();

    client.play(&session).unwrap();

    thread::sleep(Duration::from_secs(10));

    client.teardown(&session).unwrap();

    video_thread.join().unwrap();
    control_info_thread.join().unwrap();

    // let mut buf = vec![];

    // loop {
    //     match stream.read_to_end(&mut buf) {
    //         Ok(_) => break,
    //         Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
    //             break
    //         }
    //         Err(e) => panic!("encountered IO error: {}", e),
    //     };
    // };

    // println!("{}", String::from_utf8(buf).unwrap());
}

// fn main() {
//     const XADDR: &str = "http://192.168.1.88:2000/onvif/device_service";

//     let camera = Camera::new(
//         XADDR.to_string(),
//         "admin".to_string(),
//         "admin1234".to_string(),
//     )
//     .unwrap();

//     let media = camera.media();

//     let profiles = media.get_profiles().unwrap();

//     let uri = media
//         .get_stream_url(profiles.first().unwrap().token())
//         .unwrap();

//     println!("{}", uri);
// }
