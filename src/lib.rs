#![forbid(missing_docs)]
#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]
#![doc = include_str!("../README.md")]

use std::sync::Arc;

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
