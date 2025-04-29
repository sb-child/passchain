// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{impl_statement_conversions, types::Block};

use super::{Factor, Mode, Question, QuestionResponseError};

pub struct FidoKeyFactor {}

impl Factor for FidoKeyFactor {
    fn new(block: Block, mode: Mode) -> Self {
        todo!()
    }

    fn next_question(&mut self) -> Result<Option<Question>, QuestionResponseError> {
        todo!()
    }

    fn result(self) -> Result<Block, super::FactorOutputError> {
        todo!()
    }
}

pub enum FidoKeyFactorStatement {
    Create(FidoKeyFactorCreateStatement),
    Verify(FidoKeyFactorVerifyStatement),
}

#[derive(Default)]
pub enum FidoKeyFactorCreateStatement {
    #[default]
    Start,
    Final(Option<()>),
}

#[derive(Default)]
pub enum FidoKeyFactorVerifyStatement {
    #[default]
    Start,
    Final(Option<()>),
}

impl_statement_conversions!(
    FidoKeyFactorStatement,
    FidoKeyFactorCreateStatement,
    FidoKeyFactorVerifyStatement,
    Option<()>
);
