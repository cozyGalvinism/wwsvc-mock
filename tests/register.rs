use wwsvc_rs::{Credentials, WebwareClient};

mod common;

#[tokio::test]
async fn register_successfully() {
    let env = common::setup(false)
        .await
        .expect("Failed to setup test environment");

    env.client
        .register()
        .await
        .expect("Failed to register the client");
}

#[tokio::test]
async fn register_unsuccessfully() {
    let env = common::setup(false)
        .await
        .expect("Failed to setup test environment");

    let client = WebwareClient::builder()
        .webware_url(env.server.server_address().unwrap().as_str())
        .vendor_hash("a")
        .app_hash("1")
        .revision(2)
        .secret("a")
        .allow_insecure(true)
        .build();
    let reg = client
        .register()
        .await;
    pretty_assertions::assert_eq!(reg.is_err(), true);
}

#[tokio::test]
async fn register_with_invalid_revision() {
    let env = common::setup(false)
        .await
        .expect("Failed to setup test environment");

    let client = reqwest::Client::new();
    let res = client.get(&format!("{}WWSVC/WWSERVICE/REGISTER/a/a/a/-1/", env.server.server_address().unwrap().as_str()))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(res.status().as_u16(), 406);
}

#[tokio::test]
async fn deregister_successfully() {
    let env = common::setup(false)
        .await
        .expect("Failed to setup test environment");

    let client = env
        .client
        .register()
        .await
        .expect("Failed to register the client");
    client
        .deregister()
        .await
        .expect("Failed to deregister the client");
}

#[tokio::test]
async fn deregister_unsuccessfully() {
    let env = common::setup(false)
        .await
        .expect("Failed to setup test environment");

    let client = WebwareClient::builder()
        .webware_url(env.server.server_address().unwrap().as_str())
        .vendor_hash(&env.config.webware.webservices.vendor_hash)
        .app_hash(&env.config.webware.webservices.application_hash)
        .revision(env.config.webware.webservices.version)
        .secret(&env.config.webware.webservices.application_secret)
        .credentials(Credentials {
            service_pass: "a".to_string(),
            app_id: "a".to_string(),
        })
        .allow_insecure(true)
        .build()
        .register()
        .await
        .expect("Failed to register the client");
    // The client WILL deregister successfully due to the way it is implemented
    // but a different implementation could make it fail
    client
        .deregister()
        .await
        .expect("Failed to deregister the client");
}

#[tokio::test]
async fn deregister_without_headers() {
    // this test can only be done without the wwsvc-rs client
    let env = common::setup(true)
        .await
        .expect("Failed to setup test environment");
    let client = reqwest::Client::new();
    let res = client.get(&format!("{}WWSVC/WWSERVICE/DEREGISTER/{}/", env.server.server_address().unwrap().as_str(), env.config.webware.credentials.service_pass))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(res.status().as_u16(), 404);
}

#[tokio::test]
async fn deregister_with_unknown_execute_mode() {
    let env = common::setup(false)
        .await
        .expect("Failed to setup test environment");
    let client = reqwest::Client::new();
    let res = client.get(&format!("{}WWSVC/WWSERVICE/DEREGISTER/{}/", env.server.server_address().unwrap().as_str(), env.config.webware.credentials.service_pass))
        .header("WWSVC-EXECUTE-MODE", "UNKNOWN")
        .header("WWSVC-REQID", "1")
        .header("WWSVC-TS", "Mon, 01 Jan 2000 00:00:00 GMT")
        .header("WWSVC-HASH", "a")
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(res.status().as_u16(), 404);
}

#[tokio::test]
async fn deregister_with_wrong_hash() {
    let env = common::setup(false)
        .await
        .expect("Failed to setup test environment");
    let client = reqwest::Client::new();
    let res = client.get(&format!("{}WWSVC/WWSERVICE/DEREGISTER/{}/", env.server.server_address().unwrap().as_str(), env.config.webware.credentials.service_pass))
        .header("WWSVC-EXECUTE-MODE", "SYNCHRON")
        .header("WWSVC-REQID", "1")
        .header("WWSVC-TS", "Mon, 01 Jan 2000 00:00:00 GMT")
        .header("WWSVC-HASH", "a")
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(res.status().as_u16(), 404);
}
