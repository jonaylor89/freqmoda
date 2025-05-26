use std::collections::HashMap;

use secrecy::SecretString;
use serde::Deserialize;
use serde_aux::prelude::deserialize_number_from_string;
use tracing::error;

use crate::streamingpath::normalize::SafeCharsType;

#[derive(serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct Settings {
    // TODO: add config in the config to allow/disallow fetching audios from the internet
    // TODO: add config for unsafe URLs vs force the hash
    // TODO: add secret key or somehow hash with API key?
    #[serde(
        alias = "PORT",
        deserialize_with = "deserialize_number_from_string",
        default = "default_port"
    )]
    pub port: u16,
    pub application: ApplicationSettings,
    pub custom_tags: HashMap<String, String>,
    pub processor: ProcessorSettings,

    // TODO: save audio to result bucket (diff from storage bucket)
    pub storage: StorageSettings,
    pub cache: CacheSettings,
}

#[derive(serde::Deserialize, Clone)]
#[serde(default)]
pub struct ApplicationSettings {
    #[serde(alias = "HOST")]
    pub host: String,
    pub hmac_secret: SecretString,
}

impl Default for ApplicationSettings {
    fn default() -> Self {
        Self {
            host: String::from("127.0.0.1"), // default host
            hmac_secret: SecretString::from("this-is-a-secret".to_string()), // empty secret
        }
    }
}

#[derive(serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct ProcessorSettings {
    pub disabled_filters: Vec<String>,
    pub max_filter_ops: usize,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<usize>,
    pub max_cache_files: i32,
    pub max_cache_mem: i32,
    pub max_cache_size: i32,
}

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct StorageSettings {
    #[serde(default = "default_base_dir")]
    pub base_dir: String,
    pub path_prefix: String,
    pub safe_chars: SafeCharsType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<StorageClient>,
}

#[derive(Deserialize, Clone)]
pub enum StorageClient {
    S3(S3Settings),
    GCS(GCSSettings),
}

#[derive(Deserialize, Clone)]
pub struct S3Settings {
    pub region: String,
    pub bucket: String,

    #[serde(default = "default_s3_endpoint")]
    pub endpoint: String,
    pub access_key: SecretString,
    pub secret_key: SecretString,
}

fn default_s3_endpoint() -> String {
    "https://s3.amazonaws.com".to_string()
}

#[derive(Deserialize, Clone)]
pub struct GCSSettings {
    pub bucket: String,
}

fn default_port() -> u16 {
    8080
}

fn default_base_dir() -> String {
    "uploads".to_string()
}

#[derive(Deserialize, Clone)]
pub enum CacheSettings {
    Redis { uri: String },
    Filesystem(FilesystemCacheSettings),
}

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct FilesystemCacheSettings {
    #[serde(default = "default_cache_base_dir")]
    pub base_dir: String,
}

fn default_cache_base_dir() -> String {
    "cache".to_string()
}

impl Default for CacheSettings {
    fn default() -> Self {
        Self::Filesystem(FilesystemCacheSettings::default())
    }
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`",
                other
            )),
        }
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("config");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    let builder = config::Config::builder()
        .add_source(config::File::from(configuration_directory.join("base")).required(true))
        .add_source(
            config::File::from(configuration_directory.join(environment.as_str())).required(true),
        )
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .add_source(config::Environment::default());

    builder
        .build()?
        .try_deserialize::<Settings>()
        .inspect_err(|e| error!("Failed to load configuration: {}", e))
}
