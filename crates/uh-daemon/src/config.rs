//! Configuration file loader (`uh.toml`).
//!
//! Two sections: `[server]` and `[llm]`. The `[llm]` section is the
//! primary entry point — users configure OpenAI-compatible endpoints
//! (DeepSeek / OpenAI / vLLM / Ollama) here.

use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uh_llm::LlmConfig;

use crate::AppState;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("read config: {0}")]
    Read(String),
    #[error("parse config: {0}")]
    Parse(String),
    #[error("invalid config: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String { "127.0.0.1".into() }
fn default_port() -> u16 { 3080 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    /// Tools to enable. If empty, all default tools are enabled.
    #[serde(default)]
    pub enable: Vec<String>,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self { enable: vec![] }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    pub llm: LlmConfig,
    #[serde(default)]
    pub tools: ToolsConfig,
}

impl Config {
    /// Load config from a given path. The path can be relative to CWD.
    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::Read(format!(
                "config not found: {} — copy uh.toml.example to {} and edit",
                path.display(),
                path.display()
            )));
        }
        let text = std::fs::read_to_string(path).map_err(|e| ConfigError::Read(e.to_string()))?;
        let cfg: Self = toml::from_str(&text).map_err(|e| ConfigError::Parse(e.to_string()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Load config from `uh.toml` in the current directory.
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from(Path::new("uh.toml"))
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.llm.api_key.is_empty() {
            return Err(ConfigError::Invalid("llm.api_key is empty".into()));
        }
        if self.llm.base_url.is_empty() {
            return Err(ConfigError::Invalid("llm.base_url is empty".into()));
        }
        if self.llm.model.is_empty() {
            return Err(ConfigError::Invalid("llm.model is empty".into()));
        }
        Ok(())
    }

    /// Build an `AppState` from this config.
    pub fn into_state(self) -> Result<AppState, ConfigError> {
        AppState::from_config(self)
    }
}
