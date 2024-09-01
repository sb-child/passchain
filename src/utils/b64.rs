use base64::{engine::general_purpose::URL_SAFE, Engine};

pub fn b64enc(x: &dyn AsRef<[u8]>) -> String {
    let mut s = String::new();
    URL_SAFE.encode_string(x, &mut s);
    s
}

pub fn b64dec(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    URL_SAFE.decode(input)
}
