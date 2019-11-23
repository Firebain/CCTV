// mod onvif;

// use onvif::prelude::*;
// use onvif::Camera;

// use std::io;

use std::io::prelude::*;
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::thread;
use std::time::Duration;

const JPEG_LUMA_QUANTIZER: [u16; 64] = [
    16, 11, 10, 16, 24, 40, 51, 61, 12, 12, 14, 19, 26, 58, 60, 55, 14, 13, 16, 24, 40, 57, 69, 56,
    14, 17, 22, 29, 51, 87, 80, 62, 18, 22, 37, 56, 68, 109, 103, 77, 24, 35, 55, 64, 81, 104, 113,
    92, 49, 64, 78, 87, 103, 121, 120, 101, 72, 92, 95, 98, 112, 100, 103, 99,
];

const JPEG_CHROMA_QUANTIZER: [u16; 64] = [
    17, 18, 24, 47, 99, 99, 99, 99, 18, 21, 26, 66, 99, 99, 99, 99, 24, 26, 56, 99, 99, 99, 99, 99,
    47, 66, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
];

const LUM_DC_CODELENS: [u8; 16] = [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0];

const LUM_DC_SYMBOLS: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

const LUM_AC_CODELENS: [u8; 16] = [0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 0x7d];

const LUM_AC_SYMBOLS: [u8; 162] = [
    0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07,
    0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xa1, 0x08, 0x23, 0x42, 0xb1, 0xc1, 0x15, 0x52, 0xd1, 0xf0,
    0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0a, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x25, 0x26, 0x27, 0x28,
    0x29, 0x2a, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49,
    0x4a, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69,
    0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
    0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7,
    0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5,
    0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xe1, 0xe2,
    0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8,
    0xf9, 0xfa,
];

const CHM_DC_CODELENS: [u8; 16] = [0, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0];

const CHM_DC_SYMBOLS: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

const CHM_AC_CODELENS: [u8; 16] = [0, 2, 1, 2, 4, 4, 3, 4, 7, 5, 4, 4, 0, 1, 2, 0x77];

const CHM_AC_SYMBOLS: [u8; 162] = [
    0x00, 0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21, 0x31, 0x06, 0x12, 0x41, 0x51, 0x07, 0x61, 0x71,
    0x13, 0x22, 0x32, 0x81, 0x08, 0x14, 0x42, 0x91, 0xa1, 0xb1, 0xc1, 0x09, 0x23, 0x33, 0x52, 0xf0,
    0x15, 0x62, 0x72, 0xd1, 0x0a, 0x16, 0x24, 0x34, 0xe1, 0x25, 0xf1, 0x17, 0x18, 0x19, 0x1a, 0x26,
    0x27, 0x28, 0x29, 0x2a, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
    0x49, 0x4a, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68,
    0x69, 0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
    0x88, 0x89, 0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5,
    0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3,
    0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda,
    0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8,
    0xf9, 0xfa,
];

fn make_tables(q: u8) -> ([u8; 64], [u8; 64]) {
    let mut factor = q;

    if q < 1 {
        factor = 1
    };
    if q > 99 {
        factor = 99
    };
    let q = if q < 50 {
        5000 / (factor as u16)
    } else {
        200 - (factor as u16) * 2
    };

    let mut lqt: [u8; 64] = [0; 64];
    let mut cqt: [u8; 64] = [0; 64];

    for i in 0..64 {
        let mut lq = (JPEG_LUMA_QUANTIZER[i] * q + 50) / 100;
        let mut cq = (JPEG_CHROMA_QUANTIZER[i] * q + 50) / 100;

        if lq < 1 {
            lq = 1
        } else if lq > 255 {
            lq = 255
        }
        lqt[i] = lq as u8;

        if cq < 1 {
            cq = 1
        } else if cq > 255 {
            cq = 255
        }
        cqt[i] = cq as u8;
    }

    (lqt, cqt)
}

fn make_quant_header(headers: &mut Vec<u8>, qt: [u8; 64], table_no: u8) {
    headers.push(0xff);
    headers.push(0xdb);
    headers.push(0);
    headers.push(67);
    headers.push(table_no);
    headers.extend(qt.iter());
}

fn make_huffman_header(headers: &mut Vec<u8>, codelens: &[u8], ncodes: usize, symbols: &[u8], nsymbols: usize, table_no: u8, table_class: u8) {
    headers.push(0xff);
    headers.push(0xc4);
    headers.push(0);
    headers.push((3 + ncodes + nsymbols) as u8);
    headers.push((table_class << 4) | table_no);
    headers.extend(codelens.iter());
    headers.extend(symbols.iter());
}

fn make_headers(jpeg_type: u8, height: u16, width: u16, lqt: [u8; 64], cqt: [u8; 64]) -> Vec<u8> {
    let mut headers = Vec::new();

    headers.push(0xff);
    headers.push(0xd8);
    make_quant_header(&mut headers, lqt, 0);
    make_quant_header(&mut headers, cqt, 1);
    headers.push(0xff);
    headers.push(0xc0);
    headers.push(0);
    headers.push(17);
    headers.push(8);
    headers.push((height >> 8) as u8);
    headers.push((height & 255) as u8);
    headers.push((width >> 8) as u8);
    headers.push((width & 255) as u8);
    headers.push(3);
    headers.push(0);
    if jpeg_type == 0 {
        headers.push(0x21);
    } else {
        headers.push(0x22);
    }
    headers.push(0);
    headers.push(1);
    headers.push(0x11);
    headers.push(1);
    headers.push(2);
    headers.push(0x11);
    headers.push(1);
    make_huffman_header(&mut headers, &LUM_DC_CODELENS, LUM_DC_CODELENS.len(), &LUM_DC_SYMBOLS, LUM_DC_SYMBOLS.len(), 0, 0);
    make_huffman_header(&mut headers, &LUM_AC_CODELENS, LUM_AC_CODELENS.len(), &LUM_AC_SYMBOLS, LUM_AC_SYMBOLS.len(), 0, 1);
    make_huffman_header(&mut headers, &CHM_DC_CODELENS, CHM_DC_CODELENS.len(), &CHM_DC_SYMBOLS, CHM_DC_SYMBOLS.len(), 1, 0);
    make_huffman_header(&mut headers, &CHM_AC_CODELENS, CHM_AC_CODELENS.len(), &CHM_AC_SYMBOLS, CHM_AC_SYMBOLS.len(), 1, 1);
    headers.push(0xff);
    headers.push(0xda);
    headers.push(0);
    headers.push(12);
    headers.push(3);
    headers.push(0);
    headers.push(0);
    headers.push(1);
    headers.push(0x11);
    headers.push(2);
    headers.push(0x11);
    headers.push(0);
    headers.push(63);
    headers.push(0);

    headers
}

fn get_n_bit(byte: u8, n: usize) -> bool {
    if n > 8 { panic!("n more than 8") }

    let mask = 1 << n;
    let masked_byte = byte & mask;
    let bit = masked_byte >> n;

    bit == 1
}

fn video_handler(socket: UdpSocket) {
    let mut buf = [0; 65_535];

    socket.recv(&mut buf).unwrap();

    println!("first byte: {:08b}", buf[0]);
    println!("version: {}", buf[0] >> 6);
    println!("p: {}", get_n_bit(buf[0], 5));

    // loop {
    //     let mut buf = [0; 65_535];

    //     let amt = socket.recv(&mut buf).unwrap();

    //     println!("amt: {}", amt);

    //     println!();

    //     println!("main header: {:?}", &buf[..8]);

    //     let type_specific = buf[0];
    //     let fragment_offset = [buf[3], buf[2], buf[1]];
    //     let jpeg_type = buf[4];
    //     let q = buf[5];
    //     let width = buf[6];
    //     let height = buf[7];

    //     let width_converted = (width as u16) * 8;
    //     let height_converted = (height as u16) * 8;

    //     println!("type_specific: {:?}", type_specific);
    //     println!("fragment_offset: {:?}", fragment_offset);
    //     println!("jpeg_type: {:?}", jpeg_type);
    //     println!("q: {:?}", q);
    //     println!("width_original: {:?}", width);
    //     println!("width_converted: {:?}", width_converted);
    //     println!("height: {:?}", height);
    //     println!("height_converted: {:?}", height_converted);

    //     println!();

    //     let has_marker_header = jpeg_type > 63 && jpeg_type < 128;

    //     // TODO: This is Restart Marker header impl. For now it's look like useless header.
    //     // if has_marker_header {
    //     //     let restart_interval = [buf[8], buf[9]];
    //     //     let restart_count = [buf[10], buf[11]];
    //     // }

    //     let offset = if has_marker_header { 12 } else { 8 };

    //     let has_quantization_header = q > 127;

    //     let (lqt, cqt) = if has_quantization_header {
    //         println!("quantization_header: {:?}", &buf[offset..offset + 4]);

    //         let mbz = buf[offset];
    //         let precision = buf[offset + 1];
    //         let length = [buf[offset + 2], buf[offset + 3]];

    //         println!("mbz: {:?}", mbz);
    //         println!("precision: {:?}", precision);
    //         println!("length: {:?}", length);

    //         let mut lqt: [u8; 64] = [0; 64];
    //         lqt.copy_from_slice(&buf[offset + 4..offset + 4 + 64]);

    //         let mut cqt: [u8; 64] = [0; 64];
    //         cqt.copy_from_slice(&buf[offset + 5 + 64..offset + 5 + (64 * 2)]);

    //         (lqt, cqt)
    //     } else {
    //         make_tables(q)
    //     };

    //     let offset = if has_quantization_header {
    //         offset + 5 + (64 * 2) + 1
    //     } else {
    //         offset
    //     };

    //     let jpeg_data = &buf[offset..amt];

    //     println!(
    //         "{:?}",
    //         make_headers(jpeg_type, height_converted, width_converted, lqt, cqt)
    //     );

    //     println!("-----------------------------------------");
    // }
}

fn main() {
    let mut stream = TcpStream::connect("192.168.1.88:554").unwrap();

    let options = b"OPTIONS rtsp://192.168.1.88:554/av0_0 RTSP/1.0\r\n\
                    CSeq: 1\r\n\
                    User-Agent: VLC media player (LIVE555 Streaming Media v2008.07.24)\r\n\
                    \r\n";

    println!("C->S:\r\n{}", String::from_utf8_lossy(options));

    stream.write(options).unwrap();

    let mut buf = [0; 65_535];

    let amt = stream.read(&mut buf).unwrap();

    println!("S->C:\r\n{}", String::from_utf8_lossy(&buf[..amt]));

    let describe = b"DESCRIBE rtsp://192.168.1.88:554/av0_0 RTSP/1.0\r\n\
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
        "SETUP rtsp://192.168.1.88:554/av0_0 RTSP/1.0\r\n\
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
        "PLAY rtsp://192.168.1.88:554/av0_0 RTSP/1.0\r\n\
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

    // thread::sleep(Duration::from_secs(2));

    let teardown = format!(
        "TEARDOWN rtsp://192.168.1.88:554/av0_0 RTSP/1.0\r\n\
         CSeq: 5\r\n\
         User-Agent: VLC media player (LIVE555 Streaming Media v2008.07.24)\r\n\
         Session: {}\r\n\
         \r\n",
        session
    );

    let teardown = teardown.as_bytes();

    // println!("C->S:\r\n{}", String::from_utf8_lossy(teardown));

    stream.write(teardown).unwrap();

    let mut buf = [0; 65_535];

    let amt = stream.read(&mut buf).unwrap();

    // println!("S->C:\r\n{}", String::from_utf8_lossy(&buf[..amt]));

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
