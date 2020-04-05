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

pub fn make_tables(q: u8) -> ([u8; 64], [u8; 64]) {
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
