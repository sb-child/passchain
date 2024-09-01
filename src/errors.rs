// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum PasschainError {
    #[error("tracing error: {0}")]
    TracingSetGlobalDefaultError(#[from] tracing::subscriber::SetGlobalDefaultError),
    #[error("ask error: {0}")]
    AskError(#[from] AskError),
    #[error("join error: {0}")]
    JoinError(#[from] JoinError),
    #[error("config rror: {0}")]
    ConfigError(#[from] ConfigError),
    #[error("should exit")]
    ShouldExit,
    #[error("unknown error")]
    Unknown,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("io error: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("encode error: {0}")]
    EncodeError(#[from] toml::ser::Error),
    #[error("decode error: {0}")]
    DecodeError(#[from] toml::de::Error),
}

#[derive(Error, Debug)]
pub enum AskError {
    #[error("inquire error: {0}")]
    InquireError(inquire::InquireError),
    #[error("interrupted")]
    Interrupted,
    #[error("canceled")]
    Canceled,
}

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("join error: {0}")]
    JoinError(#[from] JoinError),
    #[error("fido error: {0}")]
    FidoError(anyhow::Error),
    #[error("hasher error: {0}")]
    HasherError(argon2::Error),
    #[error("no assertion found")]
    NoAssertionFound,
    #[error("multiple assertion found")]
    MultipleAssertionFound,
    #[error("entropy too low")]
    LowEntropy,
    // #[error("receive error")]
    // OneshotReceiveError(#[from] tokio::sync::oneshot::error::RecvError),
    #[error("sender dropped")]
    SenderDropped,
    #[error("receiver dropped")]
    ReceiverDropped,
}

impl From<inquire::InquireError> for AskError {
    fn from(value: inquire::InquireError) -> Self {
        use inquire::InquireError;
        match value {
            x @ InquireError::NotTTY => AskError::InquireError(x),
            x @ InquireError::InvalidConfiguration(_) => AskError::InquireError(x),
            x @ InquireError::IO(_) => AskError::InquireError(x),
            InquireError::OperationCanceled => AskError::Canceled,
            InquireError::OperationInterrupted => AskError::Interrupted,
            x @ InquireError::Custom(_) => AskError::InquireError(x),
        }
    }
}
