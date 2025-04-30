// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::types::{BLOCK_SIZE, Block};

use super::{Factor, Mode, Question, QuestionResponseError};

pub struct PlaceholderFactor {
    block: Block,
}

impl Factor for PlaceholderFactor {
    fn new(block: Block, _mode: Mode) -> Self {
        Self { block }
    }

    fn next_question(&mut self) -> Result<Option<Question>, QuestionResponseError> {
        Ok(None)
    }

    fn result(self) -> Result<Block, super::FactorOutputError> {
        use sha3::Digest;
        let mut hasher = sha3::Sha3_512::new();
        hasher.update(self.block);
        hasher.update(b"passchain-placeholder-factor");
        let hash = hasher.finalize();
        let mut res: Block = [0u8; BLOCK_SIZE];
        res.copy_from_slice(&hash);
        Ok(res)
    }
}
