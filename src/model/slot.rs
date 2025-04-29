// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::factor::Factor;

pub struct SlotModel {}

impl SlotModel {
    pub fn new(max_size: usize) -> Self {
        Self {}
    }

    pub fn put(&mut self, item: impl Factor) {
        
    }
}
