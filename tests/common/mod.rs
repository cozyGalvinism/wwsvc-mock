use axum_test::{TestServer, TestServerConfig};
use wwsvc_mock::{app, AppConfig, DeserializedRegex, FileOrString, MockResource, MockResourceMethod};
use wwsvc_rs::{collection, WebwareClient};

pub struct TestEnvironment {
    pub server: TestServer,
    pub client: wwsvc_rs::WebwareClient,
    pub config: AppConfig,
}

pub async fn setup(debug: bool) -> anyhow::Result<TestEnvironment> {
    let mut config = AppConfig::default().with_mock_resource(MockResource {
        data_source: FileOrString::File {
            file: "data/artikel_clean.json".to_string(),
        },
        function: "ARTIKEL".to_string(),
        method: MockResourceMethod::Get,
        revision: 3,
        parameters: None,
    }).with_mock_resource(MockResource {
        data_source: FileOrString::File {
            file: "data/artikel_art_nr_clean.json".to_string(),
        },
        function: "ARTIKEL".to_string(),
        method: MockResourceMethod::Get,
        revision: 3,
        parameters: Some(collection! {
            "FELDER".to_string() => DeserializedRegex::new("ART_1_25").unwrap(),
        })
    }).with_mock_resource(MockResource {
        data_source: FileOrString::Empty,
        function: "ARTIKEL".to_string(),
        method: MockResourceMethod::Put,
        revision: 1,
        parameters: Some(collection! {
            "ARTNR".to_string() => DeserializedRegex::new("Artikel19Prozent").unwrap(),
            "ART_51_60".to_string() => DeserializedRegex::new("Eine Bezeichnung").unwrap(),
        })
    }).with_mock_resource(MockResource {
        data_source: FileOrString::String { value: r#"{"ARTNR": "MeinArtikel"}"#.to_string() },
        function: "ARTIKEL".to_string(),
        method: MockResourceMethod::Insert,
        revision: 2,
        parameters: Some(collection! {
            "ARTNR".to_string() => DeserializedRegex::new("MeinArtikel").unwrap(),
        })
    }).with_mock_resource(MockResource {
        data_source: FileOrString::Empty,
        function: "ARTIKEL".to_string(),
        method: MockResourceMethod::Delete,
        revision: 1,
        parameters: Some(collection! {
            "ARTNR".to_string() => DeserializedRegex::new("Artikel19Prozent").unwrap(),
        })
    }).with_mock_resource(MockResource {
        data_source: FileOrString::String { value: r#"{"GET_RESULT": "Hallo"}"#.to_string() },
        function: "GET_RELATION".to_string(),
        method: MockResourceMethod::Exec,
        revision: 1,
        parameters: Some(collection! {
            "NR".to_string() => DeserializedRegex::new("65").unwrap(),
            "P1".to_string() => DeserializedRegex::new("Hallo").unwrap(),
        })
    });

    config.debug = debug;

    println!("----- WEBWARE Mock Server -----");
    println!("Running in test mode");
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
    let server = TestServer::new_with_config(
        app,
        TestServerConfig {
            transport: Some(axum_test::Transport::HttpIpPort {
                ip: "127.0.0.1".parse().ok(),
                port: None,
            }),
            ..Default::default()
        },
    )?;
    println!(
        "Server listening on: {}",
        server.server_address().unwrap().as_str()
    );
    let client = WebwareClient::builder()
        .webware_url(server.server_address().unwrap().as_str())
        .vendor_hash(&config.webware.webservices.vendor_hash)
        .app_hash(&config.webware.webservices.application_hash)
        .revision(config.webware.webservices.version)
        .secret(&config.webware.webservices.application_secret)
        .allow_insecure(true)
        .build();

    Ok(TestEnvironment { server, client, config })
}
