// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tokio::sync::oneshot;

use crate::{
    factor::macros::TakeFinalValue,
    impl_statement_conversions,
    types::{BLOCK_SIZE, Block},
};

use super::{
    Factor, FactorOutputError, Mode, Question, QuestionProps, QuestionResponse,
    QuestionResponseError, QuestionResponseType, macros::MaybeMut,
};

pub struct PasswordFactor {
    mode: Mode,
    block: Block,
    state: PasswordFactorStatement,
}

impl PasswordFactor {
    fn next_question_create(&mut self) -> Result<Option<Question>, QuestionResponseError> {
        let state = PasswordFactorCreateStatement::maybe_mut(&mut self.state)
            .ok_or(QuestionResponseError::IncorrectUsage)?;
        match state {
            PasswordFactorCreateStatement::Start => {
                let (q, chan) = Question::new(
                    "Set a password".into(),
                    QuestionProps::Password {
                        allow_toggle: true,
                        confirm: true,
                    },
                );
                *state = PasswordFactorCreateStatement::Wait(Some(chan));
                Ok(Some(q))
            }
            PasswordFactorCreateStatement::Wait(chan) => {
                // chan 这个时候肯定不是 None
                let chan = chan.take().unwrap();
                // 只要 sender 发送了数据, 那这边肯定不会出事的
                let password = chan.blocking_recv().unwrap()?;
                match password {
                    QuestionResponse::Password(p) => {
                        *state = PasswordFactorCreateStatement::Final(Some(p));
                    }
                    _ => {
                        return Err(QuestionResponseError::IncorrectUsage);
                    }
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn next_question_verify(&mut self) -> Result<Option<Question>, QuestionResponseError> {
        let state = PasswordFactorVerifyStatement::maybe_mut(&mut self.state)
            .ok_or(QuestionResponseError::IncorrectUsage)?;
        match state {
            PasswordFactorVerifyStatement::Start => {
                let (q, chan) = Question::new(
                    "Input password".into(),
                    QuestionProps::Password {
                        allow_toggle: true,
                        confirm: false,
                    },
                );
                *state = PasswordFactorVerifyStatement::Wait(Some(chan));
                Ok(Some(q))
            }
            PasswordFactorVerifyStatement::Wait(chan) => {
                // chan 这个时候肯定不是 None
                let chan = chan.take().unwrap();
                // 只要 sender 发送了数据, 那这边肯定不会出事的
                let password = chan.blocking_recv().unwrap()?;
                match password {
                    QuestionResponse::Password(p) => {
                        *state = PasswordFactorVerifyStatement::Final(Some(p));
                    }
                    _ => {
                        return Err(QuestionResponseError::IncorrectUsage);
                    }
                }
                Ok(None)
            }
            _ => Ok(None),
        }
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

    fn next_question(&mut self) -> Result<Option<Question>, QuestionResponseError> {
        match self.mode {
            Mode::Create => self.next_question_create(),
            Mode::Verify => self.next_question_verify(),
        }
    }

    fn result(mut self) -> Result<Block, FactorOutputError> {
        let password = self
            .state
            .take_final()
            .ok_or(FactorOutputError::IncorrectUsage)?;
        let password_bytes = password.as_bytes();
        use sha3::Digest;
        let mut hasher = sha3::Sha3_512::new();
        hasher.update(self.block);
        hasher.update(password_bytes);
        let hash = hasher.finalize();
        let mut res: Block = [0u8; BLOCK_SIZE];
        res.copy_from_slice(&hash);
        Ok(res)
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
    Wait(Option<oneshot::Receiver<QuestionResponseType>>),
    Final(Option<String>),
}

#[derive(Default)]
pub enum PasswordFactorVerifyStatement {
    #[default]
    Start,
    Wait(Option<oneshot::Receiver<QuestionResponseType>>),
    Final(Option<String>),
}

impl_statement_conversions!(
    PasswordFactorStatement,
    PasswordFactorCreateStatement,
    PasswordFactorVerifyStatement,
    Option<String>
);
