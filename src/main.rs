// mod onvif;
mod rtsp;

use rtsp::rtp_old::sequence::{
    error::RTPSequenceError, sequence::RTPSequence, sequence::RTPSequenceStatus,
};

// use onvif::prelude::*;
// use onvif::Camera;

// use std::io;

use std::fs::File;
use std::io::prelude::*;
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::thread;
use std::time::Duration;

fn video_handler(socket: UdpSocket) {
    let mut rtp_sequence = RTPSequence::new();

    loop {
        let mut buf = [0; 65_535];

        let amt = socket.recv(&mut buf).unwrap();

        let buf = &buf[..amt];

        match rtp_sequence.push(buf) {
            Ok(status) => match status {
                RTPSequenceStatus::LastPacket => {
                    let data = rtp_sequence.make();

                    let mut file = File::create("frame.jpeg").unwrap();

                    file.write_all(&data).unwrap();

                    rtp_sequence.clean();
                },
                _ => {}
            }
            Err(err) => match err {
                RTPSequenceError::PackageLost => rtp_sequence.clean(),
                _ => panic!("{}", err)
            },
        }
    }
}

fn main() {
    let mut stream = TcpStream::connect("192.168.1.88:554").unwrap();

    let options = b"OPTIONS rtsp://192.168.1.88:554/av0_1 RTSP/1.0\r\n\
                    CSeq: 1\r\n\
                    User-Agent: VLC media player (LIVE555 Streaming Media v2008.07.24)\r\n\
                    \r\n";

    println!("C->S:\r\n{}", String::from_utf8_lossy(options));

    stream.write(options).unwrap();

    let mut buf = [0; 65_535];

    let amt = stream.read(&mut buf).unwrap();

    println!("S->C:\r\n{}", String::from_utf8_lossy(&buf[..amt]));

    let describe = b"DESCRIBE rtsp://192.168.1.88:554/av0_1 RTSP/1.0\r\n\
                     CSeq: 2\r\n\
                     User-Agent: VLC media player (LIVE555 Streaming Media v2008.07.24)\r\n\
                     Accept: application/sdp\r\n\
                     \r\n";

    println!("C->S:\r\n{}", String::from_utf8_lossy(describe));

    stream.write(describe).unwrap();

    let mut buf = [0; 65_535];

    let amt = stream.read(&mut buf).unwrap();

    println!("S->C:\r\n{}", String::from_utf8_lossy(&buf[..amt]));

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

    let setup = format!(
        "SETUP rtsp://192.168.1.88:554/av0_1 RTSP/1.0\r\n\
         CSeq: 3\r\n\
         User-Agent: VLC media player (LIVE555 Streaming Media v2008.07.24)\r\n\
         Transport: RTP/AVP;unicast;client_port={}-{}\r\n\
         \r\n",
        main_socket.local_addr().unwrap().port(),
        second_socket.local_addr().unwrap().port(),
    );

    let setup = setup.as_bytes();

    println!("C->S:\r\n{}", String::from_utf8_lossy(setup));

    stream.write(setup).unwrap();

    let mut buf = [0; 65_535];

    let amt = stream.read(&mut buf).unwrap();

    println!("S->C:\r\n{}", String::from_utf8_lossy(&buf[..amt]));

    let session = String::from_utf8_lossy(&buf[..amt]);

    let session = session
        .split("\r\n")
        .find(|s| s.starts_with("Session:"))
        .unwrap()
        .split(" ")
        .nth(1)
        .unwrap();

    let play = format!(
        "PLAY rtsp://192.168.1.88:554/av0_1 RTSP/1.0\r\n\
         CSeq: 4\r\n\
         User-Agent: VLC media player (LIVE555 Streaming Media v2008.07.24)\r\n\
         Session: {}\r\n\
         Range: npt=0.000-\r\n\
         \r\n",
        session
    );

    let play = play.as_bytes();

    println!("C->S:\r\n{}", String::from_utf8_lossy(play));

    stream.write(play).unwrap();

    let mut buf = [0; 65_535];

    let amt = stream.read(&mut buf).unwrap();

    println!("S->C:\r\n{}", String::from_utf8_lossy(&buf[..amt]));

    thread::sleep(Duration::from_secs(10));

    let teardown = format!(
        "TEARDOWN rtsp://192.168.1.88:554/av0_1 RTSP/1.0\r\n\
         CSeq: 5\r\n\
         User-Agent: VLC media player (LIVE555 Streaming Media v2008.07.24)\r\n\
         Session: {}\r\n\
         \r\n",
        session
    );

    let teardown = teardown.as_bytes();

    println!("C->S:\r\n{}", String::from_utf8_lossy(teardown));

    stream.write(teardown).unwrap();

    let mut buf = [0; 65_535];

    let amt = stream.read(&mut buf).unwrap();

    println!("S->C:\r\n{}", String::from_utf8_lossy(&buf[..amt]));

    video_thread.join().unwrap();
    control_info_thread.join().unwrap();

    // println!("{}", socket.local_addr().unwrap().port());

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
