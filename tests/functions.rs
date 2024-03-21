use pretty_assertions::assert_eq;
use serde_json::json;
use wwsvc_rs::{collection, futures::FutureExt, Method};

mod common;

static ARTIKEL_JSON_STR: &str = include_str!("../data/artikel.json");
static ARTIKEL_ART_NR_JSON_STR: &str = include_str!("../data/artikel_art_nr.json");

#[tokio::test]
async fn artikel_get_all_fields() {
    let env = common::setup(false)
        .await
        .expect("Failed to setup test environment");
    let expected_json: serde_json::Value =
        serde_json::from_str(ARTIKEL_JSON_STR).expect("Failed to parse artikel.json");

    let response = env
        .client
        .with_registered(|client| {
            async {
                client
                    .request_as_response(Method::PUT, "ARTIKEL.GET", 3, collection! {}, None)
                    .await
            }
            .boxed()
        })
        .await
        .expect("Failed to register the client")
        .expect("Failed to send request");

    let status = response.status();
    let text_body = response
        .text()
        .await
        .expect("Failed to parse response body");
    assert_eq!(status.as_u16(), 200);
    let body: serde_json::Value =
        serde_json::from_str(&text_body).expect("Failed to parse response body");
    assert_eq!(body, expected_json);
}

#[tokio::test]
async fn artikel_get_art_nr() {
    let env = common::setup(false)
        .await
        .expect("Failed to setup test environment");
    let expected_json: serde_json::Value =
        serde_json::from_str(ARTIKEL_ART_NR_JSON_STR).expect("Failed to parse artikel.json");

    let response = env
        .client
        .with_registered(|client| {
            async {
                client
                    .request_as_response(Method::PUT, "ARTIKEL.GET", 3, collection! {
                        "FELDER" => "ART_1_25"
                    }, None)
                    .await
            }
            .boxed()
        })
        .await
        .expect("Failed to register the client")
        .expect("Failed to send request");

    let status = response.status();
    let text_body = response
        .text()
        .await
        .expect("Failed to parse response body");
    assert_eq!(status.as_u16(), 200);
    let body: serde_json::Value =
        serde_json::from_str(&text_body).expect("Failed to parse response body");
    assert_eq!(body, expected_json);
}

#[tokio::test]
async fn artikel_put() {
    let env = common::setup(false)
        .await
        .expect("Failed to setup test environment");
    let expected_json = json!({
        "COMRESULT": {
            "BEREICH": "WWSVC",
            "STATUS": 200,
            "CODE": "200 OK",
            "INFO": "Kein Fehler",
            "INFO2": "",
            "INFO3": "",
            "ERRNO": "0",
            "ERRNOTXT": "SVCERR_NO_ERROR (0)"
        }
    });
    let response = env
        .client
        .with_registered(|client| {
            async {
                client
                    .request_as_response(Method::PUT, "ARTIKEL.PUT", 1, collection! {
                        "ARTNR" => "Artikel19Prozent",
                        "ART_51_60" => "Eine Bezeichnung"
                    }, None)
                    .await
            }
            .boxed()
        })
        .await
        .expect("Failed to register the client")
        .expect("Failed to send request");
    let status = response.status();
    let text_body = response
        .text()
        .await
        .expect("Failed to parse response body");
    assert_eq!(status.as_u16(), 200);
    let body: serde_json::Value =
        serde_json::from_str(&text_body).expect("Failed to parse response body");
    assert_eq!(body, expected_json);
}

#[tokio::test]
async fn artikel_insert() {
    let env = common::setup(false)
        .await
        .expect("Failed to setup test environment");
    let expected_json = json!({
        "COMRESULT": {
            "BEREICH": "WWSVC",
            "STATUS": 200,
            "CODE": "200 OK",
            "INFO": "Kein Fehler",
            "INFO2": "",
            "INFO3": "",
            "ERRNO": "0",
            "ERRNOTXT": "SVCERR_NO_ERROR (0)"
        },
        "ARTNR": "MeinArtikel"
    });
    let response = env
        .client
        .with_registered(|client| {
            async {
                client
                    .request_as_response(Method::PUT, "ARTIKEL.INSERT", 2, collection! {
                        "ARTNR" => "MeinArtikel",
                    }, None)
                    .await
            }
            .boxed()
        })
        .await
        .expect("Failed to register the client")
        .expect("Failed to send request");
    let status = response.status();
    let text_body = response
        .text()
        .await
        .expect("Failed to parse response body");
    assert_eq!(status.as_u16(), 200);
    let body: serde_json::Value =
        serde_json::from_str(&text_body).expect("Failed to parse response body");
    assert_eq!(body, expected_json);
}

#[tokio::test]
async fn artikel_delete() {
    let env = common::setup(false)
        .await
        .expect("Failed to setup test environment");
    let expected_json = json!({
        "COMRESULT": {
            "BEREICH": "WWSVC",
            "STATUS": 200,
            "CODE": "200 OK",
            "INFO": "Kein Fehler",
            "INFO2": "",
            "INFO3": "",
            "ERRNO": "0",
            "ERRNOTXT": "SVCERR_NO_ERROR (0)"
        }
    });
    let response = env
        .client
        .with_registered(|client| {
            async {
                client
                    .request_as_response(Method::PUT, "ARTIKEL.DELETE", 1, collection! {
                        "ARTNR" => "Artikel19Prozent"
                    }, None)
                    .await
            }
            .boxed()
        })
        .await
        .expect("Failed to register the client")
        .expect("Failed to send request");
    let status = response.status();
    let text_body = response
        .text()
        .await
        .expect("Failed to parse response body");
    assert_eq!(status.as_u16(), 200);
    let body: serde_json::Value =
        serde_json::from_str(&text_body).expect("Failed to parse response body");
    assert_eq!(body, expected_json);
}

#[tokio::test]
async fn get_relation_exec() {
    let env = common::setup(false)
        .await
        .expect("Failed to setup test environment");
    let expected_json = json!({
        "COMRESULT": {
            "BEREICH": "WWSVC",
            "STATUS": 200,
            "CODE": "200 OK",
            "INFO": "Kein Fehler",
            "INFO2": "",
            "INFO3": "",
            "ERRNO": "0",
            "ERRNOTXT": "SVCERR_NO_ERROR (0)"
        },
        "GET_RESULT": "Hallo"
    });
    let response = env
        .client
        .with_registered(|client| {
            async {
                client
                    .request_as_response(Method::PUT, "GET_RELATION.EXEC", 1, collection! {
                        "NR" => "65",
                        "P1" => "Hallo"
                    }, None)
                    .await
            }
            .boxed()
        })
        .await
        .expect("Failed to register the client")
        .expect("Failed to send request");
    let status = response.status();
    let text_body = response
        .text()
        .await
        .expect("Failed to parse response body");
    assert_eq!(status.as_u16(), 200);
    let body: serde_json::Value =
        serde_json::from_str(&text_body).expect("Failed to parse response body");
    assert_eq!(body, expected_json);
}

#[tokio::test]
async fn non_existant_function() {
    let env = common::setup(false)
        .await
        .expect("Failed to setup test environment");
    let expected_json = json!({
        "COMRESULT": {
            "BEREICH": "WWSVC",
            "STATUS": 400,
            "CODE": "400 Bad Request",
            "INFO": "Es wurde eine fehlerhafte Anforderung übergeben.",
            "INFO2": "Funktionsname nicht bekannt.",
            "INFO3": "TURBO.FISCH.GET",
            "ERRNO": "20",
            "ERRNOTXT": "SVCERR_UNKNOWN_FUNCTION (20)"
        }
    });
    let response = env
        .client
        .with_registered(|client| {
            async {
                client
                    .request_as_response(Method::PUT, "TURBO.FISCH.GET", 1, collection! {}, None)
                    .await
            }
            .boxed()
        })
        .await
        .expect("Failed to register the client")
        .expect("Failed to send request");
    let status = response.status();
    let text_body = response
        .text()
        .await
        .expect("Failed to parse response body");
    assert_eq!(status.as_u16(), 400);
    let body: serde_json::Value =
        serde_json::from_str(&text_body).expect("Failed to parse response body");
    assert_eq!(body, expected_json);
}
