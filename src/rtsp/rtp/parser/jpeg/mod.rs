mod headers;

use headers::make_headers;

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

pub fn parse(buf: &[u8]) -> (Vec<u8>, Vec<u8>) {
    // println!();

    // println!("main header: {:?}", &buf[..8]);

    // let type_specific = buf[0];
    // let fragment_offset = [buf[3], buf[2], buf[1]];
    let jpeg_type = buf[4];
    let q = buf[5];
    let width = buf[6];
    let height = buf[7];

    let width_converted = (width as u16) * 8;
    let height_converted = (height as u16) * 8;

    // println!("type_specific: {:?}", type_specific);
    // println!("fragment_offset: {:?}", fragment_offset);
    // println!("jpeg_type: {:?}", jpeg_type);
    // println!("q: {:?}", q);
    // println!("width_original: {:?}", width);
    // println!("width_converted: {:?}", width_converted);
    // println!("height: {:?}", height);
    // println!("height_converted: {:?}", height_converted);

    // TODO: I skipped Restart Marker header impl. For now it's look like useless header.

    let has_quantization_header = q > 127;

    // println!();

    let (lqt, cqt) = if has_quantization_header {
        // println!("quantization_header: {:?}", &buf[8..12]);

        // let mbz = buf[8];
        // let precision = buf[9];
        // let length = [buf[10], buf[11]];

        // println!("mbz: {:?}", mbz);
        // println!("precision: {:?}", precision);
        // println!("length: {:?}", length);

        let mut lqt: [u8; 64] = [0; 64];
        lqt.copy_from_slice(&buf[12..12 + 64]);

        let mut cqt: [u8; 64] = [0; 64];
        cqt.copy_from_slice(&buf[13 + 64..13 + (64 * 2)]);

        (lqt, cqt)
    } else {
        make_tables(q)
    };

    // println!();

    let offset = if has_quantization_header {
        13 + (64 * 2) + 1
    } else {
        8
    };

    (
        make_headers(jpeg_type, height_converted, width_converted, lqt, cqt),
        Vec::from(&buf[offset..]),
    )
}