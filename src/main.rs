// mod onvif;
mod rtsp;

use rtsp::client::RTSPClient;
use rtsp::rtp::sequence::{RTPSequence, RTPSequenceError, RTPSequenceStatus};

// use onvif::prelude::*;
// use onvif::Camera;

// use std::io;

use image::{ImageBuffer, GenericImageView, Rgb};
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{sync_channel, Receiver};

fn jpeg_reader(receiver: Receiver<Vec<u8>>) {
    loop {
        let data = receiver.recv().unwrap();

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

        let new_image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_vec(dimensions.0 * 2, dimensions.1, new_image).unwrap();

        new_image.save("frame.jpeg").unwrap();
    }
}

fn video_handler(socket: UdpSocket) {
    let mut rtp_sequence = RTPSequence::new();

    let (sync_sender, receiver) = sync_channel(100);

    thread::spawn(move || jpeg_reader(receiver));

    loop {
        let mut buf = [0; 65_535];

        let amt = socket.recv(&mut buf).unwrap();

        let buf = &buf[..amt];

        match rtp_sequence.push(buf) {
            Ok(status) => {
                if let RTPSequenceStatus::LastPacket(data) = status {
                    if let Err(_) = sync_sender.try_send(data) {
                        println!("Buffer is full");
                    }

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