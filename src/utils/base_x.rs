// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

const B32_TABLE: [u8; 32] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'#', b'C', b'T', b'E',
    b'F', // 16
    b'Z', b'X', b'V', b'N', b'M', b'R', b'W', b'P', b'%', b'-', b'K', b'L', b'+', b'?', b'H',
    b'Q', // 32
];

const B93_TABLE: [u8; 93] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z', b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L',
    b'M', b'M', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'~', b'!',
    b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')', b'-', b'_', b'+', b'=', b'[', b'{', b']',
    b'}', b'\\', b'|', b';', b':', b'"', b'\'', b',', b'<', b'.', b'>', b'/', b'?',
];

pub fn b64enc(x: &dyn AsRef<[u8]>) -> String {
    use base64::{Engine, engine::general_purpose::URL_SAFE};
    let mut s = String::new();
    URL_SAFE.encode_string(x, &mut s);
    s
}

pub fn b64dec(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::{Engine, engine::general_purpose::URL_SAFE};
    URL_SAFE.decode(input)
}

pub fn b32enc(x: &[u8]) -> String {
    use convert_base::Convert;
    let mut base = Convert::new(256, B32_TABLE.len() as u64);
    let output: Vec<u8> = base.convert::<u8, u8>(x);
    let output = output
        .iter()
        .map(|x| {
            if *x < B32_TABLE.len() as u8 {
                B32_TABLE[*x as usize]
            } else {
                unreachable!("why there is a number {x} >= {}", B32_TABLE.len())
            }
        })
        .collect();
    String::from_utf8(output).unwrap()
}

pub fn b32dec(input: &str) -> Vec<u8> {
    use convert_base::Convert;
    let mut base = Convert::new(B32_TABLE.len() as u64, 256);
    let converted: Vec<u8> = input
        .as_bytes()
        .iter()
        .map(|x| {
            B32_TABLE
                .iter()
                .enumerate()
                .find(|(_i, y)| x == *y)
                .map(|(i, _y)| i as u8) // table 确实没那么长
                .unwrap_or(0)
        })
        .collect();

    let output: Vec<u8> = base.convert::<u8, u8>(&converted);
    output
}

pub fn b93enc(b: &[u8]) -> String {
    use convert_base::Convert;
    let mut base = Convert::new(256, B93_TABLE.len() as u64);
    let output: Vec<u8> = base.convert::<u8, u8>(b);
    let output = output
        .iter()
        .map(|x| {
            if *x < B93_TABLE.len() as u8 {
                B93_TABLE[*x as usize]
            } else {
                unreachable!("why there is a number {x} >= {}", B93_TABLE.len())
            }
        })
        .collect();
    String::from_utf8(output).unwrap()
}

/// `x` must be 64 bytes long
pub fn recovery_code_64_enc(x: &[u8]) -> Vec<String> {
    let encoder = reed_solomon::Encoder::<32>::new();
    let buf = encoder.encode(x);
    let data = buf.data(); // 64 bytes (512 bits)
    let ecc = buf.ecc(); // 32 bytes (256 bits)
    let length = data.len() + ecc.len();
    let mut combined = vec![0u8; length];
    combined[0..data.len()].copy_from_slice(data);
    combined[data.len()..data.len() + ecc.len()].copy_from_slice(ecc);
    let chunks: Vec<String> = combined.chunks(4).map(b32enc).collect();
    chunks
}

pub fn recovery_code_64_dec(x: &[String]) -> Result<Vec<u8>, RecoveryDecodeError> {
    if x.len() != (64 + 32) / 4 {
        return Err(RecoveryDecodeError::IncorrectInputLength(
            (64 + 32) / 4,
            x.len(),
        ));
    }
    let raw_bytes: Vec<u8> = x
        .iter()
        .map(|x| {
            let mut y = b32dec(x);
            y.resize(4, 0);
            y
        })
        .flatten()
        .collect();
    let decoder = reed_solomon::Decoder::<32>::new();
    let buf = decoder
        .correct(&raw_bytes, None)
        .map_err(RecoveryDecodeError::ReedSolomonError)?;
    let data = buf.data(); // 64 bytes (512 bits)
    if data.len() != 64 {
        return Err(RecoveryDecodeError::IncorrectDataLength(64, data.len()));
    }
    // assert_eq!(data.len(), 64);
    Ok(data.to_vec())
}

#[derive(thiserror::Error, Debug)]
pub enum RecoveryDecodeError {
    #[error("Reed Solomon ECC decode error: {0:?}")]
    ReedSolomonError(reed_solomon::DecoderError),

    #[error("Incorrect data length: expect {0}, got {1}")]
    IncorrectDataLength(usize, usize),

    #[error("Incorrect input length: expect {0}, got {1}")]
    IncorrectInputLength(usize, usize),
}
