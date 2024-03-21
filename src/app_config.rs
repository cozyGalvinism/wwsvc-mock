use std::{collections::HashMap, fmt::Display, str::FromStr};

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::Deserialize;
use serde_inline_default::serde_inline_default;

use crate::OptionalJson;

fn generate_hash() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut hash = String::new();
    for _ in 0..32 {
        hash.push_str(&format!("{:x}", rng.gen_range(0..16)));
    }
    hash
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct AppConfig {
    pub server: Option<ServerConfig>,
    #[serde(default = "WebwareConfig::default")]
    pub webware: WebwareConfig,
    #[serde(default)]
    pub mock_resources: Vec<MockResource>,
    #[serde(default)]
    pub debug: bool,
}

impl AppConfig {
    pub fn new() -> Result<Self, figment::Error> {
        Figment::new()
            .merge(Toml::file("config.toml"))
            .merge(Env::prefixed("APP__").split("__"))
            .extract()
    }

    pub fn with_mock_resource(mut self, resource: MockResource) -> Self {
        self.mock_resources.push(resource);
        self
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub bind_address: String,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct WebwareConfig {
    pub webservices: WebservicesConfig,

    pub credentials: CredentialsConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CredentialsConfig {
    #[serde(default = "generate_hash")]
    pub service_pass: String,
    #[serde(default = "generate_hash")]
    pub application_id: String,
}

impl Default for CredentialsConfig {
    fn default() -> Self {
        CredentialsConfig {
            service_pass: generate_hash(),
            application_id: generate_hash(),
        }
    }
}

#[serde_inline_default]
#[derive(Deserialize, Debug, Clone)]
pub struct WebservicesConfig {
    #[serde(default = "generate_hash")]
    pub vendor_hash: String,
    #[serde(default = "generate_hash")]
    pub application_hash: String,
    #[serde_inline_default(1)]
    pub version: u32,
    #[serde_inline_default("1".to_string())]
    pub application_secret: String,
}

impl Default for WebservicesConfig {
    fn default() -> Self {
        WebservicesConfig {
            vendor_hash: generate_hash(),
            application_hash: generate_hash(),
            version: 1,
            application_secret: "1".to_string(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum FileOrString {
    File { file: String },
    String { value: String },
    Empty,
}

impl FileOrString {
    pub fn as_string(&self) -> String {
        match self {
            FileOrString::File { file } => std::fs::read_to_string(file).unwrap(),
            FileOrString::String { value } => value.clone(),
            FileOrString::Empty => "".to_string(),
        }
    }

    pub fn as_json_value(&self) -> OptionalJson {
        match self {
            FileOrString::File { file: _ } => OptionalJson(Some(serde_json::from_str(&self.as_string()).unwrap())),
            FileOrString::String { value: _ } => OptionalJson(Some(serde_json::from_str(&self.as_string()).unwrap())),
            FileOrString::Empty => OptionalJson(None),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum MockResourceMethod {
    #[serde(rename = "GET")]
    Get,
    #[serde(rename = "INSERT")]
    Insert,
    #[serde(rename = "PUT")]
    Put,
    #[serde(rename = "DELETE")]
    Delete,
    #[serde(rename = "EXEC")]
    Exec,
}

impl FromStr for MockResourceMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(MockResourceMethod::Get),
            "INSERT" => Ok(MockResourceMethod::Insert),
            "PUT" => Ok(MockResourceMethod::Put),
            "DELETE" => Ok(MockResourceMethod::Delete),
            "EXEC" => Ok(MockResourceMethod::Exec),
            _ => Err(format!("Unknown method: {}", s)),
        }
    }
}

impl Display for MockResourceMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MockResourceMethod::Get => write!(f, "GET"),
            MockResourceMethod::Insert => write!(f, "INSERT"),
            MockResourceMethod::Put => write!(f, "PUT"),
            MockResourceMethod::Delete => write!(f, "DELETE"),
            MockResourceMethod::Exec => write!(f, "EXEC"),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct MockResource {
    pub data_source: FileOrString,
    pub function: String,
    pub method: MockResourceMethod,
    pub revision: u32,

    pub parameters: Option<HashMap<String, String>>,
}

impl Display for MockResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MockResource {{ function: {}, method: {}, revision: {}, parameters: {} }}",
            self.function, self.method, self.revision, match self.parameters {
                Some(ref parameters) => serde_json::to_string(parameters).unwrap(),
                None => "None".to_string(),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use pretty_assertions::assert_eq;

    macro_rules! one_line_assert_eq {
        ($fn:ident, $left:expr, $right:expr) => {
            #[test]
            fn $fn() {
                assert_eq!($left, $right);
            }
        };
    }

    #[test]
    fn test_config_parsing() {
        let config = super::AppConfig::new().unwrap();
        println!("{:?}", config);
        assert_eq!(config.mock_resources.is_empty(), true);
        assert_eq!(config.debug, false);
        assert_eq!(config.webware.credentials.service_pass.len(), 32);
        assert_eq!(config.webware.credentials.application_id.len(), 32);
        assert_eq!(config.webware.webservices.vendor_hash.len(), 32);
        assert_eq!(config.webware.webservices.application_hash.len(), 32);
        assert_eq!(config.webware.webservices.version, 1);
        assert_eq!(config.webware.webservices.application_secret, "1".to_string());
    }

    one_line_assert_eq!(method_get_to_string, super::MockResourceMethod::Get.to_string(), "GET");
    one_line_assert_eq!(method_insert_to_string, super::MockResourceMethod::Insert.to_string(), "INSERT");
    one_line_assert_eq!(method_put_to_string, super::MockResourceMethod::Put.to_string(), "PUT");
    one_line_assert_eq!(method_delete_to_string, super::MockResourceMethod::Delete.to_string(), "DELETE");
    one_line_assert_eq!(method_exec_to_string, super::MockResourceMethod::Exec.to_string(), "EXEC");
    one_line_assert_eq!(mock_resource_without_params_to_string, super::MockResource {
        data_source: super::FileOrString::File {
            file: "data/artikel_clean.json".to_string(),
        },
        function: "ARTIKEL".to_string(),
        method: super::MockResourceMethod::Get,
        revision: 3,
        parameters: None,
    }.to_string(), "MockResource { function: ARTIKEL, method: GET, revision: 3, parameters: None }");
    one_line_assert_eq!(mock_resource_with_params_to_string, super::MockResource {
        data_source: super::FileOrString::File {
            file: "data/artikel_art_nr_clean.json".to_string(),
        },
        function: "ARTIKEL".to_string(),
        method: super::MockResourceMethod::Get,
        revision: 3,
        parameters: Some(wwsvc_rs::collection! {
            "FELDER".to_string() => "ART_1_25".to_string(),
        })
    }.to_string(), "MockResource { function: ARTIKEL, method: GET, revision: 3, parameters: {\"FELDER\":\"ART_1_25\"} }");
    one_line_assert_eq!(unknown_method_from_str, super::MockResourceMethod::from_str("UNKNOWN").unwrap_err(), "Unknown method: UNKNOWN");
    one_line_assert_eq!(empty_as_str, super::FileOrString::Empty.as_string(), "");
}
