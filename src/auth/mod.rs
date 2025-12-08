use crate::config::{ApiKey, AuthConfig};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct AuthValidator {
    enabled: bool,
    api_keys: Vec<ApiKey>,
}

impl AuthValidator {
    pub fn new(config: &AuthConfig) -> Self {
        Self {
            enabled: config.enabled,
            api_keys: config.api_keys.clone(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn validate(&self, key: &str) -> bool {
        if !self.enabled {
            return true;
        }

        let hashed = hash_key(key);
        self.api_keys
            .iter()
            .any(|api_key| api_key.key_hash == hashed)
    }
}

fn hash_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}
