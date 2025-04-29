// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tokio::sync::oneshot;

use crate::types::Block;

pub mod fidokey;
pub mod password;
pub mod placeholder;

pub mod macros;

pub trait Factor {
    fn new(block: Block, mode: Mode) -> Self;
    fn next_question(&mut self) -> Result<Option<Question>, QuestionResponseError>;
    fn result(self) -> Result<Block, FactorOutputError>;
}

pub enum Factors {
    Placeholder,
    Password,
    FidoKey,
}

pub enum QuestionTypes {
    PlainText,
    Number,
    FloatNumber,
    Password,
    SingleSelect,
    MultiSelect,
    TextEdit,
    Confirm,
}

pub struct Question {
    pub title: String,
    pub props: QuestionProps,
    pub response: oneshot::Sender<QuestionResponseType>,
}

impl Question {
    pub fn new(
        title: String,
        props: QuestionProps,
    ) -> (Self, oneshot::Receiver<QuestionResponseType>) {
        let (tx, rx) = oneshot::channel();
        return (
            Question {
                title,
                props,
                response: tx,
            },
            rx,
        );
    }
}

pub enum QuestionProps {
    PlainText {
        min: usize,
        max: usize,
    },
    Number {
        min: i64,
        max: i64,
    },
    FloatNumber {
        min: f64,
        max: f64,
    },
    Password {
        allow_toggle: bool,
        confirm: bool,
    },
    SingleSelect {
        list: Vec<String>,
    },
    MultiSelect {
        list: Vec<String>,
        min: usize,
        max: usize,
    },
    TextEdit,
    Confirm {
        default: bool,
    },
}

pub type QuestionResponseType = Result<QuestionResponse, QuestionResponseError>;

#[derive(thiserror::Error, Debug)]
pub enum QuestionResponseError {
    #[error("Request cancelled")]
    Cancelled,

    #[error("Backend Error")]
    BackendError,

    #[error("Incorrect usage")]
    IncorrectUsage,
}

pub enum QuestionResponse {
    PlainText(String),
    Number(i64),
    FloatNumber(f64),
    Password(String),
    SingleSelect(usize),
    MultiSelect(Vec<bool>),
    TextEdit(String),
    Confirm(bool),
}

#[derive(Clone, Copy)]
pub enum Mode {
    Create,
    Verify,
}

#[derive(thiserror::Error, Debug)]
pub enum FactorOutputError {
    #[error("Incorrect usage")]
    IncorrectUsage,
}
