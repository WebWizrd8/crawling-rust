use std::env;

use base64::Engine;
use serde::Deserialize;
use serde::Serialize;
use util::Result;

#[derive(Serialize, Deserialize, Clone)]
pub struct ApplicationConfig {
    pub db_url: String,
    pub jwt_secret: Box<[u8]>,
    pub admins: Vec<String>,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            jwt_secret: Box::new(b"default_key_insecure".to_owned()),
            db_url: "postgresql://0.0.0.0:5432?dbname=postgres&user=postgres&password=mempools".to_string(),
            admins: vec!["admin@admin.com".to_string()],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub application_config: ApplicationConfig,
}

impl Config {
    pub fn from_env_or_default() -> Result<Self> {
        if let Ok(encoded_config) = env::var("CONFIG") {
            if encoded_config.is_empty() {
                return Err("config env var is empty".into());
            }
            let config_raw = base64::prelude::BASE64_STANDARD.decode(encoded_config)?;
            let mut config: Config = serde_json::from_slice(&config_raw)?;
            config.application_config.db_url = env::var("db_url")?;
            config.application_config.jwt_secret = base64::prelude::BASE64_STANDARD
                .decode(env::var("jwt_secret")?)?
                .into_boxed_slice();
            return Ok(config);
        }

        Ok(Config::default())
    }
}
