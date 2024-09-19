// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub fn sha3_512(x: impl AsRef<[u8]>) -> [u8; 64] {
    use sha3::Digest;
    let mut hash = sha3::Sha3_512::new();
    hash.update(x);
    let hash = hash.finalize();
    let mut res = [0u8; 64];
    res.copy_from_slice(&hash);
    return res;
}

pub fn sha2_256(x: impl AsRef<[u8]>) -> [u8; 32] {
    use sha2::Digest;
    let mut hash = sha2::Sha256::new();
    hash.update(x);
    let hash = hash.finalize();
    let mut res = [0u8; 32];
    res.copy_from_slice(&hash);
    return res;
}

pub fn blake3_64(x: &[u8]) -> [u8; 64] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(x);
    let mut o = hasher.finalize_xof();
    let mut output = [0; 64];
    o.fill(&mut output);
    output
}
