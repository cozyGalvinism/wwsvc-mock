use std::{collections::HashMap, fmt::Display, path::Path, str::FromStr};

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::Deserialize;
use serde_inline_default::serde_inline_default;

use crate::{DeserializedRegex, OptionalJson};

fn generate_hash() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut hash = String::new();
    for _ in 0..32 {
        hash.push_str(&format!("{:x}", rng.gen_range(0..16)));
    }
    hash
}

/// The main configuration of the mock server. See each field for more information.
#[derive(Deserialize, Default, Debug, Clone)]
pub struct AppConfig {
    /// The server configuration, see [ServerConfig] for more information.
    pub server: Option<ServerConfig>,
    /// The webware mocking configuration, see [WebwareConfig] for more information.
    #[serde(default = "WebwareConfig::default")]
    pub webware: WebwareConfig,
    /// A list of mock resources to be used by the server. For more information see [MockResource].
    #[serde(default)]
    pub mock_resources: Vec<MockResource>,
    /// Whether to enable the debug middleware for logging requests and responses.
    #[serde(default)]
    pub debug: bool,
}

impl AppConfig {
    /// Loads the configuration from both the `config.toml` file and the environment variables.
    /// 
    /// The environment variables are prefixed with `APP__` and split by `__`. For example, the
    /// `server.bind_address` field can be set by the `APP__SERVER__BIND_ADDRESS` environment.
    pub fn new() -> Result<Self, figment::Error> {
        Figment::new()
            .merge(Toml::file("config.toml"))
            .merge(Env::prefixed("APP__").split("__"))
            .extract()
    }

    /// Loads the configuration from the specified file and the environment variables.
    ///
    /// The environment variables are prefixed with `APP__` and split by `__`. For example, the
    /// `server.bind_address` field can be set by the `APP__SERVER__BIND_ADDRESS` environment.
    pub fn from_file(file: &Path) -> Result<Self, figment::Error> {
        Figment::new()
            .merge(Toml::file(file))
            .merge(Env::prefixed("APP__").split("__"))
            .extract()
    }

    /// Adds a [mock resource][MockResource] to the configuration.
    pub fn with_mock_resource(mut self, resource: MockResource) -> Self {
        self.mock_resources.push(resource);
        self
    }
}

/// The server configuration. This config only applies for the binary, not the library.
#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    /// The address to bind the server to. For example, `127.0.0.1:3000`.
    pub bind_address: String,
}

/// The mocking configuration for the WEBWARE, which includes the webservices and the associated credentials.
#[derive(Deserialize, Default, Debug, Clone)]
pub struct WebwareConfig {
    /// The configuration for the webservices, see [WebservicesConfig] for more information.
    #[serde(default)]
    pub webservices: WebservicesConfig,
    /// The credentials that the webservices will accept. See [CredentialsConfig] for more information.
    #[serde(default)]
    pub credentials: CredentialsConfig,
}

/// The credentials configuration for the webservices.
#[derive(Deserialize, Debug, Clone)]
pub struct CredentialsConfig {
    /// The service pass that the webservices will accept.
    /// 
    /// If not provided, a random 32 character hash will be generated.
    #[serde(default = "generate_hash")]
    pub service_pass: String,
    /// The application ID that the webservices will accept.
    /// 
    /// If not provided, a random 32 character hash will be generated.
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

/// The configuration for the webservices. You can either provide your own hashes or let the server generate them.
#[serde_inline_default]
#[derive(Deserialize, Debug, Clone)]
pub struct WebservicesConfig {
    /// The vendor hash that the webservices will accept.
    /// 
    /// If not provided, a random 32 character hash will be generated.
    #[serde(default = "generate_hash")]
    pub vendor_hash: String,
    /// The application hash that the webservices will accept.
    /// 
    /// If not provided, a random 32 character hash will be generated.
    #[serde(default = "generate_hash")]
    pub application_hash: String,
    /// The version of the webservices application that the server will accept.
    /// 
    /// If not provided, the version will be set to `1`.
    #[serde_inline_default(1)]
    pub version: u32,
    /// The application secret that the webservices will accept.
    /// 
    /// If not provided, the secret will be set to `1`.
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

/// A data source that can be either a file path, a string or empty.
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum FileOrString {
    /// A file path to read the data from.
    File {
        /// The path to the file.
        file: String
    },
    /// A string to use as the data.
    String {
        /// The string value.
        value: String
    },
    /// An empty data source.
    Empty,
}

impl FileOrString {
    /// Returns the data source as a string.
    /// 
    /// If the data source is a file, it will read the file and return the contents.
    /// If the data source is a string, it will return the string.
    /// If the data source is empty, it will return an empty string.
    pub fn as_string(&self) -> String {
        match self {
            FileOrString::File { file } => std::fs::read_to_string(file).unwrap(),
            FileOrString::String { value } => value.clone(),
            FileOrString::Empty => "".to_string(),
        }
    }

    /// Returns the data source as an [OptionalJson] value.
    /// 
    /// If the data source is a file, it will read the file and parse it as JSON.
    /// If the data source is a string, it will parse the string as JSON.
    /// If the data source is empty, it will return `None`.
    pub fn as_json_value(&self) -> OptionalJson {
        match self {
            FileOrString::File { file: _ } => OptionalJson(Some(serde_json::from_str(&self.as_string()).unwrap())),
            FileOrString::String { value: _ } => OptionalJson(Some(serde_json::from_str(&self.as_string()).unwrap())),
            FileOrString::Empty => OptionalJson(None),
        }
    }
}

/// The method of the mock resource.
/// 
/// These are the methods that the WEBSERVICES accept for functions.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum MockResourceMethod {
    /// The GET method, used for reading data.
    /// 
    /// Serializes and deserializes to and from `GET`.
    #[serde(rename = "GET")]
    Get,
    /// The INSERT method, used for inserting data.
    /// 
    /// Serializes and deserializes to and from `INSERT`.
    #[serde(rename = "INSERT")]
    Insert,
    /// The PUT method, used for updating data.
    /// 
    /// Serializes and deserializes to and from `PUT`.
    #[serde(rename = "PUT")]
    Put,
    /// The DELETE method, used for deleting data.
    /// 
    /// Serializes and deserializes to and from `DELETE`.
    #[serde(rename = "DELETE")]
    Delete,
    /// The EXEC method, used for executing functions.
    /// 
    /// Serializes and deserializes to and from `EXEC`.
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

/// A mock resource that the server will use to mock the WEBSERVICES.
/// 
/// The resource will only return the data from the data source if the function, method, revision and parameters match.
/// There is currently no way to do wildcard matching.
#[derive(Deserialize, Debug, Clone)]
pub struct MockResource {
    /// The [data source][FileOrString] for the mock resource.
    pub data_source: FileOrString,
    /// The function name for the mock resource.
    /// 
    /// This is the name of the function but without the method. For example, `ARTIKEL`.
    pub function: String,
    /// The method for the mock resource. See [MockResourceMethod] for more information.
    pub method: MockResourceMethod,
    /// The revision for the mock resource.
    pub revision: u32,

    /// The parameters for the mock resource.
    pub parameters: Option<HashMap<String, DeserializedRegex>>,
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

    use crate::DeserializedRegex;

    macro_rules! one_line_assert_eq {
        ($fn:ident, $left:expr, $right:expr) => {
            #[test]
            fn $fn() {
                assert_eq!($left, $right);
            }
        };
    }

    #[test]
    fn default_config() {
        let config = super::AppConfig::new().unwrap();
        assert_eq!(config.mock_resources.is_empty(), true);
        assert_eq!(config.debug, false);
        assert_eq!(config.webware.credentials.service_pass.len(), 32);
        assert_eq!(config.webware.credentials.application_id.len(), 32);
        assert_eq!(config.webware.webservices.vendor_hash.len(), 32);
        assert_eq!(config.webware.webservices.application_hash.len(), 32);
        assert_eq!(config.webware.webservices.version, 1);
        assert_eq!(config.webware.webservices.application_secret, "1".to_string());
    }

    #[test]
    fn config_from_file() {
        figment::Jail::expect_with(|jail| {
            jail.create_file("test-config.toml", r#"[server]
            bind_address = "0.0.0.0:3000"
            
            [[mock_resources]]
            data_source.type = "Empty"
            function = "ARTIKEL"
            method = "INSERT"
            revision = 1
            parameters.ARTNR = "MeinArtikel""#)?;

            let config = super::AppConfig::from_file(std::path::Path::new("test-config.toml")).unwrap();
            assert_eq!(config.server.unwrap().bind_address, "0.0.0.0:3000");
            assert_eq!(config.mock_resources.len(), 1);
            assert_eq!(config.mock_resources[0].function, "ARTIKEL");
            assert_eq!(config.mock_resources[0].method, super::MockResourceMethod::Insert);
            assert_eq!(config.mock_resources[0].revision, 1);
            assert_eq!(config.mock_resources[0].parameters.as_ref().unwrap().get("ARTNR").unwrap().is_match("MeinArtikel"), true);

            Ok(())
        });
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
            "FELDER".to_string() => DeserializedRegex(regex::Regex::new("ART_1_25").unwrap()),
        })
    }.to_string(), "MockResource { function: ARTIKEL, method: GET, revision: 3, parameters: {\"FELDER\":\"ART_1_25\"} }");
    one_line_assert_eq!(unknown_method_from_str, super::MockResourceMethod::from_str("UNKNOWN").unwrap_err(), "Unknown method: UNKNOWN");
    one_line_assert_eq!(empty_as_str, super::FileOrString::Empty.as_string(), "");
}
