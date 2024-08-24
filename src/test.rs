// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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
