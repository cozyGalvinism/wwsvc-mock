use std::str::FromStr;

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::app_config::{MockResource, MockResourceMethod};

pub mod exec_json;
pub mod service_pass;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComResult {
    #[serde(rename = "STATUS")]
    pub status: u32,
    #[serde(rename = "CODE")]
    pub code: String,
    #[serde(rename = "INFO")]
    pub info: String,
    #[serde(rename = "INFO2")]
    pub info2: Option<String>,
    #[serde(rename = "INFO3")]
    pub info3: Option<String>,
    #[serde(rename = "ERRNO")]
    pub errno: Option<String>,
    #[serde(rename = "BEREICH")]
    pub bereich: Option<String>,
    #[serde(rename = "ERRNOTXT")]
    pub errnotxt: Option<String>,
}

#[derive(Default)]
pub struct ComResultBuilder {
    status: Option<u32>,
    code: Option<String>,
    info: Option<String>,
    info2: Option<String>,
    info3: Option<String>,
    errno: Option<String>,
    bereich: Option<String>,
    errnotxt: Option<String>,
}

impl ComResultBuilder {
    pub fn new() -> Self {
        ComResultBuilder::default()
    }

    pub fn with_status(status: StatusCode) -> Self {
        ComResultBuilder::new().status(status.as_u16() as u32)
    }

    pub fn status(mut self, status: u32) -> Self {
        self.status = Some(status);
        self
    }

    pub fn code(mut self, code: &str) -> Self {
        self.code = Some(code.to_string());
        self
    }

    pub fn info(mut self, info: &str) -> Self {
        self.info = Some(info.to_string());
        self
    }

    pub fn info2(mut self, info2: &str) -> Self {
        self.info2 = Some(info2.to_string());
        self
    }

    pub fn info3(mut self, info3: &str) -> Self {
        self.info3 = Some(info3.to_string());
        self
    }

    pub fn errno(mut self, errno: &str) -> Self {
        self.errno = Some(errno.to_string());
        self
    }

    pub fn bereich(mut self, bereich: &str) -> Self {
        self.bereich = Some(bereich.to_string());
        self
    }

    pub fn errnotxt(mut self, errnotxt: &str) -> Self {
        self.errnotxt = Some(errnotxt.to_string());
        self
    }

    pub fn build(self) -> anyhow::Result<ComResult> {
        Ok(ComResult {
            status: self.status.ok_or(anyhow::anyhow!("status is required"))?,
            code: self.code.ok_or(anyhow::anyhow!("code is required"))?,
            info: self.info.ok_or(anyhow::anyhow!("info is required"))?,
            info2: self.info2,
            info3: self.info3,
            errno: self.errno,
            bereich: self.bereich,
            errnotxt: self.errnotxt,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceResponse<T> {
    #[serde(rename = "COMRESULT")]
    pub comresult: ComResult,
    #[serde(flatten)]
    pub body: T,
}

impl<T: Serialize> IntoResponse for ServiceResponse<T> {
    fn into_response(self) -> axum::response::Response {
        // appeasing the coverage gods
        let try_status = StatusCode::from_u16(self.comresult.status as u16);
        let status = try_status.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebserviceParameter {
    #[serde(rename = "PNAME")]
    pub name: String,
    #[serde(rename = "PCONTENT")]
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebserviceFunction {
    #[serde(rename = "FUNCTIONNAME")]
    pub function_name: String,
    #[serde(rename = "REVISION")]
    pub revision: u32,
    #[serde(rename = "PARAMETER")]
    pub parameter: Vec<WebserviceParameter>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebservicePassInfo {
    #[serde(rename = "SERVICEPASS")]
    pub service_pass: String,
    #[serde(rename = "APPHASH")]
    pub app_hash: String,
    #[serde(rename = "TIMESTAMP")]
    pub timestamp: String,
    #[serde(rename = "REQUESTID")]
    pub request_id: usize,
    #[serde(rename = "EXECUTE_MODE")]
    pub execute_mode: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebserviceRequest {
    #[serde(rename = "WWSVC_FUNCTION")]
    pub function: WebserviceFunction,
    #[serde(rename = "WWSVC_PASSINFO")]
    pub pass_info: WebservicePassInfo,
}

impl WebserviceRequest {
    pub fn lookup_resource(&self, resources: &[MockResource]) -> Option<MockResource> {
        resources
            .iter()
            .find(|resource| {
                let split = self
                    .function
                    .function_name
                    .split('.')
                    .collect::<Vec<&str>>();
                if split.len() != 2 {
                    return false;
                }
                let function_name = split[0];
                let method = MockResourceMethod::from_str(split[1]).unwrap();
                resource.function == function_name
                    && resource.method == method
                    && match resource.parameters {
                        Some(ref parameters) => parameters.iter().all(|(k, v)| {
                            self.function.parameter.iter().any(|request_parameter| {
                                request_parameter.name == *k && request_parameter.value == *v
                            })
                        }),
                        None => self.function.parameter.is_empty(),
                    }
            })
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;

    #[test]
    fn invalid_status_code_service_response() {
        let response = super::ServiceResponse {
            comresult: super::ComResult {
                status: 1001,
                code: "1001".to_string(),
                info: "Invalid".to_string(),
                info2: None,
                info3: None,
                errno: None,
                bereich: None,
                errnotxt: None,
            },
            body: (),
        };

        let response = response.into_response();
        assert_eq!(response.status(), 500);
    }
}
