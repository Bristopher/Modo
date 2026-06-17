// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::Parser;
use fluxer_media_proxy::{cli, run};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let args = cli::Args::parse();
    let cfg = cli::load_config(&args)?;
    run(cfg).await
}
