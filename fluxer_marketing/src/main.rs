// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Context;
use fluxer_marketing::{build_router, config::MarketingConfig};
use tokio::{net::TcpListener, runtime::Builder};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = MarketingConfig::from_env();
    let addr = format!("{}:{}", config.host, config.port);
    let router = build_router(config);
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to create Fluxer marketing async runtime")?;

    runtime.block_on(async move {
        let listener = TcpListener::bind(&addr)
            .await
            .with_context(|| format!("failed to bind Fluxer marketing service on {addr}"))?;
        tracing::info!(%addr, "starting Fluxer marketing service");

        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .context("marketing server exited unexpectedly")
    })?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
