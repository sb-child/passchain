use crate::{impl_statement_conversions, types::Block};

use super::{Factor, Mode};

pub struct FidoKeyFactor {}

impl Factor for FidoKeyFactor {
    fn new(block: Block, mode: Mode) -> Self {
        todo!()
    }

    fn next_question(&mut self) -> Option<super::Question> {
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
}

#[derive(Default)]
pub enum FidoKeyFactorVerifyStatement {
    #[default]
    Start,
}

impl_statement_conversions!(
    FidoKeyFactorStatement,
    FidoKeyFactorCreateStatement,
    FidoKeyFactorVerifyStatement
);
