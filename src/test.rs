// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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
