// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::{Parser, Subcommand};
use passchain::{cli, errors, utils};
use tracing::error;

/// Multi-factor authentication for LUKS
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Force to keyscript mode
    #[arg(short, long, default_value_t = false)]
    keyscript: bool,
    /// Subcommands
    #[command(subcommand)]
    sub: Option<SubCommands>,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    /// Create a new chain in the interactive shell
    Create(cli::create::Args),
}

#[tokio::main]
async fn main() -> anyhow::Result<(), errors::PasschainError> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .without_time()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(false)
        .with_target(false)
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();
    if let Some(sc) = args.sub {
        return match sc {
            SubCommands::Create(x) => x.build()?.execute().await,
        };
    };
    if !args.keyscript && !utils::keyscript::is_early_stage().await {
        error!("It looks like the environment is not early stage.");
        error!(
            "pass `--help` to print help message, pass `--keyscript` to force to keyscript mode."
        );
        return Err(errors::PasschainError::ShouldExit);
    };
    let ks = cli::keyscript::Executor {};
    ks.execute().await
}
