//! Server that hosts many blogs, each on its own domain.

use std::{
    future::IntoFuture,
    net::{Ipv4Addr, SocketAddr},
    process::ExitCode,
};

use anyhow::Context;
use cuhlumn_server::{
    admin_router,
    config::{Config, Task},
    healthcheck, public_router, router,
    state::AppState,
};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{EnvFilter, filter::LevelFilter};

#[tokio::main]
async fn main() -> anyhow::Result<ExitCode> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()
        .context("RUST_LOG is not a valid tracing filter")?;
    tracing_subscriber::fmt().with_env_filter(filter).init();
    let config = Config::from_env().unwrap_or_else(|err| err.exit());
    if config.command == Some(Task::Healthcheck) {
        let port = config.port;
        let healthy = tokio::task::spawn_blocking(move || healthcheck::run(port))
            .await
            .context("the health check task failed")?;
        return Ok(if healthy {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        });
    }
    serve(config).await?;
    Ok(ExitCode::SUCCESS)
}

async fn serve(config: Config) -> anyhow::Result<()> {
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.port));
    let admin_address = config.admin_addr;
    if admin_address.is_some() && config.console_host.is_some() {
        tracing::warn!("CUHLUMN_CONSOLE_HOST has no effect, because CUHLUMN_ADMIN_ADDR is set");
    }
    let data = config.data.clone();
    let state = AppState::open(config)
        .await
        .with_context(|| format!("cannot open the data folder {}", data.display()))?;
    if state.settings().needs_setup() {
        tracing::warn!("the server has no admin token yet, open the console to set the token");
    }
    let listener = bind(address, "PORT").await?;
    let Some(admin_address) = admin_address else {
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
        return Ok(());
    };
    let admin_listener = bind(admin_address, "CUHLUMN_ADMIN_ADDR").await?;
    tracing::info!(
        %address,
        %admin_address,
        data = %data.display(),
        version = env!("CARGO_PKG_VERSION"),
        "cuhlumn-server is listening, the blogs on the public address and the console on the admin address"
    );
    serve_both(state, listener, admin_listener).await?;
    tracing::info!("cuhlumn-server stopped");
    Ok(())
}

/// Serves the public and the admin listener until one shutdown signal stops both.
async fn serve_both(
    state: AppState,
    public: TcpListener,
    admin: TcpListener,
) -> anyhow::Result<()> {
    let stop = CancellationToken::new();
    let signal = stop.clone();
    tokio::spawn(async move {
        shutdown().await;
        signal.cancel();
    });
    let blogs = axum::serve(public, public_router(state.clone()))
        .with_graceful_shutdown(stop.clone().cancelled_owned())
        .into_future();
    let console = axum::serve(admin, admin_router(state))
        .with_graceful_shutdown(stop.cancelled_owned())
        .into_future();
    tokio::try_join!(blogs, console).context("the server stopped with an error")?;
    Ok(())
}

/// Binds `address`. An error names the address and the `variable` that sets it.
async fn bind(address: SocketAddr, variable: &str) -> anyhow::Result<TcpListener> {
    TcpListener::bind(address)
        .await
        .with_context(|| format!("cannot listen on {address}, which {variable} sets"))
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
