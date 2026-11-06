//! Server that hosts many blogs, each on its own domain.

use std::net::{Ipv4Addr, SocketAddr};

use anyhow::Context;
use clap::Parser;
use cuhlumn_server::{config::Config, router, state::AppState};
use tracing_subscriber::{EnvFilter, filter::LevelFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()
        .context("RUST_LOG is not a valid tracing filter")?;
    tracing_subscriber::fmt().with_env_filter(filter).init();
    let config = Config::parse();
    if config.token().is_none() {
        tracing::warn!(
            "CUHLUMN_TOKEN is not set, so the admin API and the console are off; blogs are still served"
        );
    }
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.port));
    let data = config.data.clone();
    let state = AppState::open(config)
        .await
        .with_context(|| format!("cannot open the data folder {}", data.display()))?;
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .with_context(|| format!("cannot listen on {address}"))?;
    tracing::info!(
        %address,
        data = %data.display(),
        version = env!("CARGO_PKG_VERSION"),
        "cuhlumn-server is listening"
    );
    axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown())
        .await
        .context("the server stopped with an error")?;
    tracing::info!("cuhlumn-server stopped");
    Ok(())
}

/// Waits for Ctrl-C or, on Unix, SIGTERM, so the requests in flight can finish.
async fn shutdown() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            tracing::error!(error = %err, "cannot listen for Ctrl-C");
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(err) => {
                tracing::error!(error = %err, "cannot listen for SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => (),
        () = terminate => (),
    }
    tracing::info!("shutdown signal received, finishing requests in flight");
}
