use config::{Config, ConfigError, Environment, File};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde_aux::prelude::deserialize_number_from_string;
use sqlx::ConnectOptions;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use tracing::error;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    #[serde(
        alias = "PORT",
        deserialize_with = "deserialize_number_from_string",
        default = "default_port"
    )]
    pub port: u16,
    pub server: ServerConfig,
    pub database: DatabaseSettings,
    pub claude: ClaudeConfig,
    pub streaming_engine: StreamingEngineConfig,
    pub redis: RedisConfig,
    pub posthog: Option<PostHogConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,

    pub port: u16,

    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClaudeConfig {
    pub api_key: SecretString,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StreamingEngineConfig {
    pub base_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: SecretString,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PostHogConfig {
    pub api_key: SecretString,
    pub host: String,
}

#[derive(Debug, Clone)]
pub enum AppEnvironment {
    Local,
    Production,
}

impl AppEnvironment {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppEnvironment::Local => "local",
            AppEnvironment::Production => "production",
        }
    }
}

impl TryFrom<String> for AppEnvironment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}

impl DatabaseSettings {
    pub fn with_db(&self) -> PgConnectOptions {
        let mut options = self.without_db().database(&self.database_name);

        options = options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }

    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("config");

    let environment: AppEnvironment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    let builder = Config::builder()
        .add_source(File::from(configuration_directory.join("base")).required(true))
        .add_source(File::from(configuration_directory.join(environment.as_str())).required(true))
        .add_source(
            Environment::with_prefix("GATEWAY")
                .prefix_separator("_")
                .separator("__"),
        )
        .add_source(Environment::default());

    builder
        .build()?
        .try_deserialize::<Settings>()
        .inspect_err(|e| error!("Failed to load configuration: {}", e))
}

fn default_port() -> u16 {
    9000
}
