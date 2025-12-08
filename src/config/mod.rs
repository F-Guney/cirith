use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::sync::Arc;
// crate imports
use crate::auth::AuthValidator;
use crate::metrics::Metrics;
use crate::rate_limit::RateLimiter;
use crate::storage::Database;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub http_client: reqwest::Client,
    pub rate_limiter: RateLimiter,
    pub auth_validator: AuthValidator,
    pub metrics: Arc<Metrics>,
    pub database: Arc<Database>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub rate_limit: RateLimitConfig,
    pub auth: AuthConfig,
    pub routes: Vec<Route>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub enabled: bool,
    pub api_keys: Vec<ApiKey>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Route {
    pub path: String,
    pub upstream: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiKey {
    pub name: String,
    pub key_hash: String,
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(serde_yaml::Error),
    InvalidValue(String),
}

impl std::error::Error for ConfigError {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "Config file error: {}", e),
            Self::Parse(e) => write!(f, "Config parse error: {}", e),
            Self::InvalidValue(e) => write!(f, "Invalid value: {}", e),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::Io(err)
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(err: serde_yaml::Error) -> Self {
        ConfigError::Parse(err)
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;

        if config.rate_limit.max_requests == 0 {
            return Err(ConfigError::InvalidValue(
                "max_requests cannot be 0".to_string(),
            ));
        }

        if config.rate_limit.window_secs == 0 {
            return Err(ConfigError::InvalidValue(
                "window_secs cannot be 0".to_string(),
            ));
        }

        Ok(config)
    }

    pub fn find_route(&self, request_path: &str) -> Option<&Route> {
        self.routes
            .iter()
            .filter(|route| request_path.starts_with(&route.path))
            .max_by_key(|route| route.path.len())
    }
}
