use std::sync::Arc;

use axum::{
    async_trait,
    extract::{FromRequestParts, Path, State},
    http::{request::Parts, HeaderMap, StatusCode},
};
use encoding_rs::WINDOWS_1252;
use serde::de::DeserializeOwned;

use crate::AppConfig;

use super::{ComResultBuilder, ServiceResponse};

#[derive(serde::Serialize)]
pub struct RegisterResponse {
    #[serde(skip_serializing_if = "Option::is_none", rename = "SERVICEPASS")]
    service_pass: Option<ServicePass>,
}

impl RegisterResponse {
    fn success(service_pass: &str, app_id: &str) -> ServiceResponse<Self> {
        ServiceResponse {
            comresult: ComResultBuilder::with_status(StatusCode::OK)
                .code("200 OK")
                .info("REGISTER OK")
                .build()
                .unwrap(),
            body: RegisterResponse {
                service_pass: Some(ServicePass {
                    pass_id: service_pass.to_string(),
                    app_id: app_id.to_string(),
                }),
            },
        }
    }

    fn error() -> ServiceResponse<Self> {
        ServiceResponse {
            comresult: ComResultBuilder::with_status(StatusCode::NOT_ACCEPTABLE)
                .code("406 Not Acceptable")
                .info("REGISTER is not possible")
                .build()
                .unwrap(),
            body: RegisterResponse { service_pass: None },
        }
    }
}

#[derive(serde::Serialize, Clone)]
pub struct ServicePass {
    #[serde(rename = "PASSID")]
    pub pass_id: String,
    #[serde(rename = "APPID")]
    pub app_id: String,
}

pub async fn handle_register(
    RegisterPath((vendor_hash, app_hash, secret, revision)): RegisterPath<(
        String,
        String,
        String,
        u32,
    )>,
    State(app_config): State<Arc<AppConfig>>,
) -> ServiceResponse<RegisterResponse> {
    if app_config.webware.webservices.vendor_hash != vendor_hash
        || app_config.webware.webservices.application_hash != app_hash
        || app_config.webware.webservices.application_secret != secret
        || app_config.webware.webservices.version != revision
    {
        RegisterResponse::error()
    } else {
        RegisterResponse::success(
            &app_config.webware.credentials.service_pass,
            &app_config.webware.credentials.application_id,
        )
    }
}

pub async fn handle_deregister(
    Path(service_pass): Path<String>,
    State(app_config): State<Arc<AppConfig>>,
    headers: HeaderMap,
) -> ServiceResponse<()> {
    if service_pass != app_config.webware.credentials.service_pass {
        return ServiceResponse {
            comresult: ComResultBuilder::with_status(StatusCode::NOT_FOUND)
                .code("404 Resource not found")
                .info("ERROR ServicePass not known")
                .info2("wwsvc-mock: ServicePass not known")
                .build()
                .unwrap(),
            body: (),
        };
    }

    if headers.get("WWSVC-EXECUTE-MODE").is_none()
        || headers.get("WWSVC-REQID").is_none()
        || headers.get("WWSVC-TS").is_none()
        || headers.get("WWSVC-HASH").is_none()
    {
        return ServiceResponse {
            comresult: ComResultBuilder::with_status(StatusCode::NOT_FOUND)
                .code("404 Resource not found")
                .info("ERROR ServicePass not known")
                .info2("wwsvc-mock: Mandatory header missing")
                .build()
                .unwrap(),
            body: (),
        };
    }

    let execute_mode = headers.get("WWSVC-EXECUTE-MODE").unwrap().to_str().unwrap();
    let ts = headers.get("WWSVC-TS").unwrap().to_str().unwrap();
    let hash = headers.get("WWSVC-HASH").unwrap().to_str().unwrap();

    if !["SYNCHRON", "ASYNCHRON"].contains(&execute_mode) {
        return ServiceResponse {
            comresult: ComResultBuilder::with_status(StatusCode::NOT_FOUND)
                .code("404 Resource not found")
                .info("ERROR ServicePass not known")
                .info2("wwsvc-mock: Execute mode not known")
                .build()
                .unwrap(),
            body: (),
        };
    }

    let expected_pre_hash = format!("{}{}", app_config.webware.credentials.application_id, ts);
    let (cow, _, _) = WINDOWS_1252.encode(expected_pre_hash.as_str());
    let expected_hash = format!("{:x}", md5::compute(cow));

    if hash != expected_hash {
        return ServiceResponse {
            comresult: ComResultBuilder::with_status(StatusCode::NOT_FOUND)
                .code("404 Resource not found")
                .info("ERROR ServicePass not known")
                .info2("wwsvc-mock: Hash not correct")
                .build()
                .unwrap(),
            body: (),
        };
    }

    ServiceResponse {
        comresult: ComResultBuilder::with_status(StatusCode::OK)
            .code("200 OK")
            .info("SERVICEPASS DEREGISTERED")
            .build()
            .unwrap(),
        body: (),
    }
}

pub struct RegisterPath<T>(T);

#[async_trait]
impl<S, T> FromRequestParts<S> for RegisterPath<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ServiceResponse<RegisterResponse>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<T>::from_request_parts(parts, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(_) => Err(RegisterResponse::error()),
        }
    }
}
