#![forbid(missing_docs)]
#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]
#![doc = include_str!("../README.md")]

use std::{ops::Deref, sync::Arc};

use axum::{
    body::{Body, Bytes},
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, put},
    Router,
};
use http_body_util::BodyExt;

mod app_config;
mod routes;

pub use app_config::{AppConfig, FileOrString, MockResource, MockResourceMethod, ServerConfig, WebwareConfig, WebservicesConfig, CredentialsConfig};
use routes::{
    exec_json::exec_json,
    service_pass::{handle_deregister, handle_register},
};

#[derive(axum::extract::FromRef, Clone)]
struct AppState {
    pub config: Arc<AppConfig>,
}

#[cfg(not(tarpaulin_include))]
async fn logging_middleware(
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = request.into_parts();
    let bytes =
        buffer_and_print(&format!("--> {} {}", parts.method, parts.uri.path()), body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));
    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print(&format!("<-- {}", parts.status), body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

#[cfg(not(tarpaulin_include))]
async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{} {}", direction, body);
    }

    Ok(bytes)
}

/// A wrapper for `serde_json::Value` that serializes as an empty object if `None`.
#[derive(serde::Serialize, Debug)]
pub struct OptionalJson(
    #[serde(skip_serializing_if = "Option::is_none")] Option<serde_json::Value>,
);

/// Generates the router for the mock server using the provided configuration.
/// 
/// It currently supports the following routes:
/// 
/// - `PUT/POST/DELETE /WWSVC/EXECJSON/`
/// - `PUT/POST/DELETE /WWSVC/EXECJSON`
/// - `GET /WWSVC/WWSERVICE/REGISTER/:vendor_hash/:app_hash/:secret/:revision/`
/// - `GET /WWSVC/WWSERVICE/DEREGISTER/:service_pass/`
pub async fn app(config: &AppConfig) -> anyhow::Result<Router> {
    let registering_routes = Router::new()
        .route(
            "/REGISTER/:vendor_hash/:app_hash/:secret/:revision/",
            get(handle_register),
        )
        .route("/DEREGISTER/:service_pass/", get(handle_deregister));

    let wwsvc_router = Router::new()
        .route(
            "/EXECJSON/",
            put(exec_json).post(exec_json).delete(exec_json),
        )
        .route(
            "/EXECJSON",
            put(exec_json).post(exec_json).delete(exec_json),
        )
        .nest("/WWSERVICE", registering_routes);

    let mut router = Router::new()
        .nest("/WWSVC", wwsvc_router)
        .with_state(AppState {
            config: Arc::new(config.clone()),
        });

    if config.debug {
        router = router.layer(axum::middleware::from_fn(logging_middleware));
    }

    Ok(router)
}

/// A wrapper for `regex::Regex` that deserializes from a string.
#[derive(Debug, Clone)]
pub struct DeserializedRegex(regex::Regex);

impl DeserializedRegex {
    /// Creates a new `DeserializedRegex` from a [String]
    pub fn new(s: &str) -> Result<Self, regex::Error> {
        regex::Regex::new(s).map(DeserializedRegex)
    }
}

impl<'de> serde::Deserialize<'de> for DeserializedRegex {
    fn deserialize<D>(deserializer: D) -> Result<DeserializedRegex, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        regex::Regex::new(&s)
            .map(DeserializedRegex)
            .map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for DeserializedRegex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        self.0.as_str().serialize(serializer)
    }
}

impl Deref for DeserializedRegex {
    type Target = regex::Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<regex::Regex> for DeserializedRegex {
    fn as_ref(&self) -> &regex::Regex {
        &self.0
    }
}

#[cfg(test)]
// why is this necessary?
#[cfg(not(tarpaulin_include))]
mod tests {
    #[test]
    fn deserialize_regex() {
        let deserialized = serde_json::from_str::<super::DeserializedRegex>(r#""^abc$""#).unwrap();
        assert_eq!(deserialized.as_str(), "^abc$");
    }

    #[test]
    fn serialize_regex() {
        let serialized = serde_json::to_string(&super::DeserializedRegex::new("^abc$").unwrap()).unwrap();
        assert_eq!(serialized, r#""^abc$""#);
    }
}
