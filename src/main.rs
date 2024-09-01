// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::{Parser, Subcommand};
use passchain::{cli, config, errors, utils};
use tracing::{error, instrument::WithSubscriber, level_filters::LevelFilter, Level};

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
    use tracing_indicatif::IndicatifLayer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    let indicatif_layer = IndicatifLayer::new();
    let subscriber = tracing_subscriber::fmt::layer()
        .compact()
        .without_time()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(false)
        .with_target(false)
        .with_writer(indicatif_layer.get_stderr_writer());
    tracing_subscriber::registry()
        .with(subscriber)
        .with(indicatif_layer)
        .with(LevelFilter::from_level(Level::DEBUG))
        .init();

    // let dp = tracing_subscriber::registry()
    //     .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
    //     .with(indicatif_layer)
    //     .with_subscriber(subscriber);
    // let dp = dp.dispatcher();
    // tracing_subscriber::;
    // .with_subscriber(subscriber);
    // let disp = binding.dispatcher();
    // tracing::dispatcher::set_global_default(dp.clone())?;
    // tracing_subscriber::registry()
    //     .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
    //     .with(indicatif_layer)
    //     .init();
    // tracing::subscriber::set_global_default(subscriber)?;

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
