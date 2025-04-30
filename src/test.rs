// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::hint::black_box;

use crate::utils::base_x::b93enc;

#[test]
fn test_entropy() {
    let r = new_random_block();
    let e = entropy::shannon_entropy(r);
    println!("e = {e}");
}

fn new_random_block() -> [u8; 512] {
    use ring::rand::{SecureRandom, SystemRandom};
    let rnd = SystemRandom::new();
    let mut tmp = [0; 512];
    rnd.fill(&mut tmp).unwrap();
    tmp
}

#[test]
fn test_block128_to_password() {
    use crate::utils;

    fn new_random_block() -> [u8; 128] {
        use ring::rand::{SecureRandom, SystemRandom};
        let rnd = SystemRandom::new();
        let mut tmp = [0; 128];
        rnd.fill(&mut tmp).unwrap();
        tmp
    }
    let r = new_random_block();
    let x1 = utils::base_x::b93enc(&r);
    let x2 = utils::base_x::b64enc(&r);

    let b3 = utils::hash::blake3_64(&r);
    let x3 = utils::base_x::b93enc(&b3);

    println!("x1 = {x1}");
    println!("x2 = {x2}");
    println!("x3 = {x3}");
}

#[test]
fn test_block64_to_password() {
    use crate::utils;

    fn new_random_block() -> [u8; 64] {
        use ring::rand::{SecureRandom, SystemRandom};
        let rnd = SystemRandom::new();
        let mut tmp = [0; 64];
        rnd.fill(&mut tmp).unwrap();
        tmp
    }
    let r = new_random_block();
    let x1 = utils::base_x::b93enc(&r);
    let x2 = utils::base_x::b64enc(&r);

    let b3 = utils::hash::blake3_64(&r);
    let x3 = utils::base_x::b93enc(&b3);

    println!("x1 = {x1}");
    println!("x2 = {x2}");
    println!("x3 = {x3}");
}

#[test]
fn benchmark_blake3_hash() {
    use crate::utils::hash::blake3_64;

    fn fill_random(buf: &mut [u8]) {
        use ring::rand::{SecureRandom, SystemRandom};
        let rnd = SystemRandom::new();
        rnd.fill(buf).unwrap();
    }

    fn do_hash<const L: usize>() {
        let mut buf = vec![0u8; L];
        fill_random(&mut buf);
        let start = std::time::Instant::now();
        let res = blake3_64(&buf);
        let stop = std::time::Instant::now();
        black_box(res);
        let hash_cost = (stop - start).as_secs_f64();
        println!("hash {L} cost {hash_cost} sec");
    }

    do_hash::<512>();
    do_hash::<5120>();
    do_hash::<51200>();
    do_hash::<512000>(); // 0.001381922 sec
    do_hash::<5120000>(); // 0.000707347 sec
    do_hash::<51200000>(); // 0.002713202 sec
}

#[test]
fn benchmark_sha3_512_hash() {
    use crate::utils::hash::sha3_512;

    fn fill_random(buf: &mut [u8]) {
        use ring::rand::{SecureRandom, SystemRandom};
        let rnd = SystemRandom::new();
        rnd.fill(buf).unwrap();
    }

    fn do_hash<const L: usize>() {
        let mut buf = vec![0u8; L];
        fill_random(&mut buf);
        let start = std::time::Instant::now();
        let res = sha3_512(&buf);
        let stop = std::time::Instant::now();
        black_box(res);
        let hash_cost = (stop - start).as_secs_f64();
        println!("hash {L} cost {hash_cost} sec");
    }

    do_hash::<512>();
    do_hash::<5120>();
    do_hash::<51200>();
    do_hash::<512000>();
    do_hash::<5120000>(); // about 1.75 sec
    // do_hash::<51200000>(); // about 17.6 sec
}

#[test]
fn benchmark_argon2id_hash() {
    const BLOCK_SIZE: usize = 64;
    type Block = [u8; BLOCK_SIZE];

    fn new_hasher<'k>() -> argonautica::Hasher<'k> {
        use argonautica::config::{Backend, Variant, Version};
        let mut h = argonautica::Hasher::default();
        h.configure_backend(Backend::C)
            .configure_hash_len(BLOCK_SIZE as u32)
            .configure_iterations(9)
            .configure_memory_size(1048576)
            .configure_password_clearing(false)
            .configure_secret_key_clearing(false)
            .configure_threads(4)
            .configure_lanes(4)
            .configure_variant(Variant::Argon2id)
            .configure_version(Version::_0x13)
            // i don't use this for now
            .opt_out_of_secret_key(true);
        h
    }

    fn calculate_hash<'k>(
        mut argon: argonautica::Hasher<'k>,
        pwd: &'k [u8],
        salt: &'k [u8],
    ) -> Result<Block, argonautica::Error> {
        let r = argon.with_password(pwd).with_salt(salt).hash_raw()?;
        let r = r.raw_hash_bytes();
        let mut res = [0u8; BLOCK_SIZE];
        res.copy_from_slice(r);
        Ok(res)
    }

    fn new_random_block() -> Block {
        use ring::rand::{SecureRandom, SystemRandom};
        let rnd = SystemRandom::new();
        let mut tmp: Block = [0; BLOCK_SIZE];
        rnd.fill(&mut tmp).unwrap();
        tmp
    }

    let h = new_hasher();

    let b1 = new_random_block();
    let b2 = new_random_block();
    let start = std::time::Instant::now();
    let r = calculate_hash(h, &b1, &b2).unwrap();
    let stop = std::time::Instant::now();
    black_box(r);
    let hash_cost = (stop - start).as_secs_f64();
    println!("hash cost {hash_cost} sec");
}

#[test]
fn test_recovery_code() {
    use crate::utils::base_x::{recovery_code_64_dec, recovery_code_64_enc};

    let mut buf = [0u8; 64];

    fn fill_random(buf: &mut [u8]) {
        use ring::rand::{SecureRandom, SystemRandom};
        let rnd = SystemRandom::new();
        rnd.fill(buf).unwrap();
    }

    fill_random(&mut buf);

    let mut rc = recovery_code_64_enc(&buf);
    for (i, v) in rc.iter().enumerate() {
        println!("{}: {v}", i + 1);
    }
    rc[12] = "22222".to_string();
    rc[23] = "44444".to_string();
    rc[1] = "66666".to_string();
    rc[5] = "888888".to_string();
    let raw_bytes = recovery_code_64_dec(&rc).unwrap();

    assert_eq!(buf, *raw_bytes);

    let password = b93enc(&buf);
    println!("{password}");
}

#[test]
fn test_b32() {
    use crate::utils::base_x::{b32dec, b32enc};

    let mut buf = [0u8; 64];

    fn fill_random(buf: &mut [u8]) {
        use ring::rand::{SecureRandom, SystemRandom};
        let rnd = SystemRandom::new();
        rnd.fill(buf).unwrap();
    }

    fill_random(&mut buf);
    println!("origin: {buf:?}");

    let enc = b32enc(&buf);
    println!("enc: {enc}");

    let dec = b32dec(&enc);
    println!("dec: {dec:?}");

    assert_eq!(buf, *dec);
}
