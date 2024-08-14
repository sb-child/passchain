// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PasschainError {
    #[error("tracing error")]
    TracingSetGlobalDefaultError(#[from] tracing::subscriber::SetGlobalDefaultError),
    #[error("should exit")]
    ShouldExit,
    #[error("unknown error")]
    Unknown,
}
