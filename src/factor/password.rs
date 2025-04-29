use crate::{impl_statement_conversions, types::Block};

use super::{Factor, FactorOutputError, Mode, Question, QuestionProps, macros::MaybeMut};

pub struct PasswordFactor {
    mode: Mode,
    block: Block,
    state: PasswordFactorStatement,
}

impl PasswordFactor {
    fn next_question_create(&mut self) -> Option<Question> {
        let state = PasswordFactorCreateStatement::maybe_mut(&mut self.state)?;
        let (q, chan) = Question::new(
            "Set a password".into(),
            QuestionProps::Password {
                allow_toggle: true,
                confirm: true,
            },
        );
        Some(q)
    }

    fn next_question_verify(&mut self) -> Option<Question> {
        let state = PasswordFactorVerifyStatement::maybe_mut(&mut self.state)?;
        let (q, chan) = Question::new(
            "Input password".into(),
            QuestionProps::Password {
                allow_toggle: true,
                confirm: false,
            },
        );
        Some(q)
    }
}

impl Factor for PasswordFactor {
    fn new(block: Block, mode: Mode) -> Self {
        Self {
            mode,
            block,
            state: mode.into(),
        }
    }

    fn next_question(&mut self) -> Option<Question> {
        match self.mode {
            Mode::Create => self.next_question_create(),
            Mode::Verify => self.next_question_verify(),
        }
    }

    fn result(self) -> Result<Block, FactorOutputError> {
        Ok([0u8; 64])
    }
}

pub enum PasswordFactorStatement {
    Create(PasswordFactorCreateStatement),
    Verify(PasswordFactorVerifyStatement),
}

#[derive(Default)]
pub enum PasswordFactorCreateStatement {
    #[default]
    Start,
}

#[derive(Default)]
pub enum PasswordFactorVerifyStatement {
    #[default]
    Start,
}

impl_statement_conversions!(
    PasswordFactorStatement,
    PasswordFactorCreateStatement,
    PasswordFactorVerifyStatement
);
