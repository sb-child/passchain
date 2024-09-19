// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use base64::{engine::general_purpose::URL_SAFE, Engine};

pub fn b64enc(x: &dyn AsRef<[u8]>) -> String {
    let mut s = String::new();
    URL_SAFE.encode_string(x, &mut s);
    s
}

pub fn b64dec(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    URL_SAFE.decode(input)
}

pub fn b93enc(b: &[u8]) -> String {
    use convert_base::Convert;
    let table: [char; 93] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
        'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'M', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '~', '!', '@', '#', '$', '%', '^', '&', '*', '(',
        ')', '-', '_', '+', '=', '[', '{', ']', '}', '\\', '|', ';', ':', '"', '\'', ',', '<', '.',
        '>', '/', '?',
    ];
    let mut base = Convert::new(256, table.len() as u64);
    let output: Vec<u8> = base.convert::<u8, u8>(b);
    output
        .iter()
        .map(|x| {
            if *x < table.len() as u8 {
                table[*x as usize]
            } else {
                ' '
            }
        })
        .collect()
}
