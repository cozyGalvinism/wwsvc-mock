use tokio::net::TcpListener;
use wwsvc_mock::{app, AppConfig};

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::new()?;

    let Some(server_config) = &config.server else {
        anyhow::bail!(
            "No server configuration found in config.toml or environment variables. Exiting."
        );
    };

    println!("----- WEBWARE Mock Server -----");
    println!(
        "Server listening on: http://{}/",
        server_config.bind_address
    );
    println!("Mocked Resources: {}", config.mock_resources.len());
    println!("Vendor Hash: {}", config.webware.webservices.vendor_hash);
    println!(
        "Application Hash: {}",
        config.webware.webservices.application_hash
    );
    println!("Revision: {}", config.webware.webservices.version);
    println!(
        "Application Secret: {}",
        config.webware.webservices.application_secret
    );
    println!("--------- Credentials ---------");
    println!("Service Pass: {}", config.webware.credentials.service_pass);
    println!(
        "Application ID: {}",
        config.webware.credentials.application_id
    );
    println!("-------------------------------");

    let app = app(&config).await?;
    let tcp_listener = TcpListener::bind(&server_config.bind_address).await?;
    axum::serve(tcp_listener, app.into_make_service()).await?;

    Ok(())
}
