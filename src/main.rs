use tokio::net::TcpListener;
use wwsvc_mock::{app, AppConfig};

#[cfg(not(tarpaulin_include))]
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C/SIGINT");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to listen for SIGTERM")
            .recv()
            .await
            .expect("Failed to receive SIGTERM");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C/SIGINT");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM");
        }
    }
}

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = AppConfig::new()?;

    let Some(server_config) = &config.server else {
        anyhow::bail!(
            "No server configuration found in config.toml or environment variables. Exiting."
        );
    };

    tracing::info!("----- WEBWARE Mock Server -----");
    tracing::info!(
        "Server listening on: http://{}/",
        server_config.bind_address
    );
    tracing::info!("Mocked Resources: {}", config.mock_resources.len());
    tracing::info!("Vendor Hash: {}", config.webware.webservices.vendor_hash);
    tracing::info!(
        "Application Hash: {}",
        config.webware.webservices.application_hash
    );
    tracing::info!("Revision: {}", config.webware.webservices.version);
    tracing::info!(
        "Application Secret: {}",
        config.webware.webservices.application_secret
    );
    tracing::info!("--------- Credentials ---------");
    tracing::info!("Service Pass: {}", config.webware.credentials.service_pass);
    tracing::info!(
        "Application ID: {}",
        config.webware.credentials.application_id
    );
    tracing::info!("-------------------------------");

    let app = app(&config).await?;
    let tcp_listener = TcpListener::bind(&server_config.bind_address).await?;
    axum::serve(tcp_listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
