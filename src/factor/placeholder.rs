use crate::{impl_statement_conversions, types::Block};

use super::{Factor, Mode};

pub struct PlaceholderFactor {}

impl Factor for PlaceholderFactor {
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

pub enum PlaceholderFactorStatement {
    Create(PlaceholderFactorCreateStatement),
    Verify(PlaceholderFactorVerifyStatement),
}

#[derive(Default)]
pub enum PlaceholderFactorCreateStatement {
    #[default]
    Start,
}

#[derive(Default)]
pub enum PlaceholderFactorVerifyStatement {
    #[default]
    Start,
}

impl_statement_conversions!(
    PlaceholderFactorStatement,
    PlaceholderFactorCreateStatement,
    PlaceholderFactorVerifyStatement
);
