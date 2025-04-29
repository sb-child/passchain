// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::hint::black_box;

use crate::utils;

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
fn test_block_to_password() {
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
fn test_argon2id_hash() {
    const BLOCK_SIZE: usize = 64;
    type Block = [u8; BLOCK_SIZE];

    fn new_hasher<'k>() -> argonautica::Hasher<'k> {
        use argonautica::config::{Backend, Variant, Version};
        let mut h = argonautica::Hasher::default();
        h.configure_backend(Backend::C)
            .configure_hash_len(BLOCK_SIZE as u32)
            .configure_iterations(2)
            .configure_memory_size(2u32.pow(18))
            .configure_password_clearing(false)
            .configure_secret_key_clearing(false)
            .configure_threads(2)
            .configure_lanes(2)
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
    let hash_cost = (stop - start).as_secs_f64();
    println!("hash cost {hash_cost} sec");
    black_box(r);
}
