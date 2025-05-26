use gateway_service::config::{
    ClaudeConfig, DatabaseSettings, RedisConfig, ServerConfig, Settings, StreamingEngineConfig,
};
use secrecy::{ExposeSecret, SecretString};
use std::collections::HashMap;
use tempfile::TempDir;

pub struct TestConfigBuilder {
    server: Option<ServerConfig>,
    database: Option<DatabaseSettings>,
    claude: Option<ClaudeConfig>,
    streaming_engine: Option<StreamingEngineConfig>,
    temp_dir: Option<TempDir>,
}

impl Default for TestConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TestConfigBuilder {
    pub fn new() -> Self {
        Self {
            server: None,
            database: None,
            claude: None,
            streaming_engine: None,
            temp_dir: None,
        }
    }

    pub fn with_server(mut self, server: ServerConfig) -> Self {
        self.server = Some(server);
        self
    }

    pub fn with_database(mut self, database: DatabaseSettings) -> Self {
        self.database = Some(database);
        self
    }

    pub fn with_claude(mut self, claude: ClaudeConfig) -> Self {
        self.claude = Some(claude);
        self
    }

    pub fn with_streaming_engine(mut self, streaming_engine: StreamingEngineConfig) -> Self {
        self.streaming_engine = Some(streaming_engine);
        self
    }

    pub fn with_temp_dir(mut self, temp_dir: TempDir) -> Self {
        self.temp_dir = Some(temp_dir);
        self
    }

    pub fn build(self) -> Settings {
        Settings {
            port: 0, // Let OS assign a free port
            server: self.server.unwrap_or_else(default_test_server_config),
            database: self.database.unwrap_or_else(default_test_database_config),
            claude: self.claude.unwrap_or_else(default_test_claude_config),
            streaming_engine: self
                .streaming_engine
                .unwrap_or_else(default_test_streaming_engine_config),
            redis: default_test_redis_config(),
            posthog: None, // No PostHog config in tests by default
        }
    }
}

pub fn default_test_server_config() -> ServerConfig {
    ServerConfig {
        host: "127.0.0.1".to_string(),
    }
}

pub fn default_test_database_config() -> DatabaseSettings {
    DatabaseSettings {
        username: "postgres".to_string(),
        password: SecretString::new("password".to_string().into()),
        port: 5432,
        host: "localhost".to_string(),
        database_name: "gateway_test".to_string(),
        require_ssl: false,
    }
}

pub fn default_test_claude_config() -> ClaudeConfig {
    ClaudeConfig {
        api_key: SecretString::from("test-api-key-do-not-use-in-production"),
        base_url: "http://localhost:3001".to_string(), // Will be mocked
        model: "claude-3-5-sonnet-20241022".to_string(),
    }
}

pub fn default_test_streaming_engine_config() -> StreamingEngineConfig {
    StreamingEngineConfig {
        base_url: "http://localhost:8080".to_string(), // Will be mocked
    }
}

pub fn default_test_redis_config() -> RedisConfig {
    RedisConfig {
        url: SecretString::from("redis://localhost:6379"),
    }
}

pub fn test_settings_with_database_url(database_url: &str) -> Settings {
    let mut database = default_test_database_config();

    // Parse the database URL to extract components
    if let Ok(url) = url::Url::parse(database_url) {
        if let Some(host) = url.host_str() {
            database.host = host.to_string();
        }
        if let Some(port) = url.port() {
            database.port = port;
        }
        if let Some(username) = url.username().strip_prefix("") {
            if !username.is_empty() {
                database.username = username.to_string();
            }
        }
        if let Some(password) = url.password() {
            database.password = SecretString::new(password.to_string().into());
        }

        let path = url.path().trim_start_matches('/');
        if !path.is_empty() {
            database.database_name = path.to_string();
        }
    }

    TestConfigBuilder::new().with_database(database).build()
}

pub fn test_settings_with_mock_urls(claude_mock_url: &str, streaming_mock_url: &str) -> Settings {
    let claude = ClaudeConfig {
        api_key: SecretString::from("test-api-key"),
        base_url: claude_mock_url.to_string(),
        model: "claude-3-5-sonnet-20241022".to_string(),
    };

    let streaming_engine = StreamingEngineConfig {
        base_url: streaming_mock_url.to_string(),
    };

    TestConfigBuilder::new()
        .with_claude(claude)
        .with_streaming_engine(streaming_engine)
        .build()
}

pub fn create_test_config_file_content() -> String {
    r#"
port: 0
server:
  host: "127.0.0.1"

database:
  username: "postgres"
  password: "password"
  port: 5432
  host: "localhost"
  database_name: "gateway_test"
  require_ssl: false

claude:
  api_key: "test-api-key"
  base_url: "http://localhost:3001"
  model: "claude-3-5-sonnet-20241022"

streaming_engine:
  base_url: "http://localhost:8080"

redis:
  url: "redis://localhost:6379"
"#
    .trim()
    .to_string()
}

pub fn create_test_env_vars() -> HashMap<String, String> {
    let mut env_vars = HashMap::new();

    env_vars.insert("APP_ENVIRONMENT".to_string(), "test".to_string());
    env_vars.insert("PORT".to_string(), "0".to_string());
    env_vars.insert(
        "GATEWAY_DATABASE__USERNAME".to_string(),
        "postgres".to_string(),
    );
    env_vars.insert(
        "GATEWAY_DATABASE__PASSWORD".to_string(),
        "password".to_string(),
    );
    env_vars.insert(
        "GATEWAY_DATABASE__DATABASE_NAME".to_string(),
        "gateway_test".to_string(),
    );
    env_vars.insert(
        "GATEWAY_DATABASE__REQUIRE_SSL".to_string(),
        "false".to_string(),
    );
    env_vars.insert(
        "GATEWAY_CLAUDE__API_KEY".to_string(),
        "test-api-key".to_string(),
    );
    env_vars.insert(
        "GATEWAY_CLAUDE__BASE_URL".to_string(),
        "http://localhost:3001".to_string(),
    );

    env_vars
}

pub fn apply_test_env_vars(_env_vars: &HashMap<String, String>) {
    // Environment variable manipulation removed for safety
    // Tests should use direct config construction instead
}

pub fn cleanup_test_env_vars(_env_vars: &HashMap<String, String>) {
    // Environment variable manipulation removed for safety
    // Tests should use direct config construction instead
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder_defaults() {
        let settings = TestConfigBuilder::new().build();

        assert_eq!(settings.server.host, "127.0.0.1");
        assert_eq!(settings.port, 0);
        assert_eq!(settings.database.username, "postgres");
        assert_eq!(settings.claude.model, "claude-3-5-sonnet-20241022");
    }

    #[test]
    fn test_config_builder_overrides() {
        let custom_server = ServerConfig {
            host: "0.0.0.0".to_string(),
        };

        let settings = TestConfigBuilder::new().with_server(custom_server).build();

        assert_eq!(settings.server.host, "0.0.0.0");
        assert_eq!(settings.port, 0);
    }

    #[test]
    fn test_database_url_parsing() {
        let database_url = "postgresql://testuser:testpass@testhost:5433/testdb";
        let settings = test_settings_with_database_url(database_url);

        assert_eq!(settings.database.host, "testhost");
        assert_eq!(settings.database.port, 5433);
        assert_eq!(settings.database.username, "testuser");
        assert_eq!(settings.database.database_name, "testdb");
    }

    #[test]
    fn test_mock_urls() {
        let claude_url = "http://localhost:3001";
        let streaming_url = "http://localhost:8080";

        let settings = test_settings_with_mock_urls(claude_url, streaming_url);

        assert_eq!(settings.claude.base_url, claude_url);
        assert_eq!(settings.streaming_engine.base_url, streaming_url);
    }

    #[test]
    fn test_env_vars_creation() {
        let env_vars = create_test_env_vars();

        assert!(env_vars.contains_key("APP_ENVIRONMENT"));
        assert_eq!(env_vars.get("APP_ENVIRONMENT").unwrap(), "test");
        assert!(env_vars.contains_key("GATEWAY_CLAUDE__API_KEY"));
    }

    #[test]
    fn test_redis_config() {
        let redis_config = default_test_redis_config();
        assert!(redis_config.url.expose_secret().starts_with("redis://"));
    }
}
