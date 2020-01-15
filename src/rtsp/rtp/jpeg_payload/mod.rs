mod headers;
mod tables;

use headers::make_headers;
use tables::make_tables;

pub fn parse(buf: &[u8], header_seated: bool) -> (Option<Vec<u8>>, Vec<u8>) {
    let fragment_offset = u32::from_be_bytes([0, buf[3], buf[2], buf[1]]);
    let jpeg_type = buf[4];
    let q = buf[5];
    let width = buf[6];
    let height = buf[7];

    let width_converted = (width as u16) * 8;
    let height_converted = (height as u16) * 8;

    let has_quantization_header = q > 127;

    if fragment_offset == 0 {
        let (lqt, cqt) = if has_quantization_header && !header_seated {
            let mut lqt: [u8; 64] = [0; 64];
            lqt.copy_from_slice(&buf[12..12 + 64]);

            let mut cqt: [u8; 64] = [0; 64];
            cqt.copy_from_slice(&buf[13 + 64..13 + (64 * 2)]);

            (lqt, cqt)
        } else {
            make_tables(q)
        };

        (
            Some(make_headers(
                jpeg_type,
                height_converted,
                width_converted,
                lqt,
                cqt,
            )),
            Vec::from(&buf[140..]),
        )
    } else {
        (None, Vec::from(&buf[8..]))
    }
}
