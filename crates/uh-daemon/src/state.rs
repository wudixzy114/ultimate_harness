//! AppState — the in-memory state of the daemon.

use std::sync::Arc;

use serde::Serialize;
use tokio::sync::{Mutex, RwLock};
use tracing::info;
use uh_core::transport::Message;
use uh_llm::Llm;
use uh_tools::ToolRegistry;
use uuid::Uuid;

use crate::config::Config;
use crate::peers::Peers;

pub struct AppState {
    pub llm: Arc<dyn Llm>,
    pub tools: ToolRegistry,
    pub session: RwLock<Session>,
    pub peers: Peers,
    pub cancel_token: Mutex<Option<Arc<tokio::sync::Notify>>>,
    pub config: Arc<Config>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub id: Uuid,
    pub messages: Vec<Message>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            messages: Vec::new(),
            created_at: chrono::Utc::now(),
        }
    }
}

#[derive(Clone)]
pub struct PeerHandle {
    pub id: Uuid,
    pub sender: tokio::sync::mpsc::UnboundedSender<serde_json::Value>,
}

impl AppState {
    pub fn from_config(config: Config) -> Result<Self, crate::config::ConfigError> {
        let llm: Arc<dyn Llm> = match config.llm.provider {
            uh_llm::LlmProvider::OpenAiCompat => Arc::new(
                uh_llm::OpenAiCompat::new(config.llm.clone())
                    .map_err(|e| crate::config::ConfigError::Invalid(format!("llm client: {e}")))?,
            ),
        };
        let tools = uh_tools::default_registry();

        info!(
            provider = %llm.name(),
            model = %llm.model(),
            tools = tools.specs().len(),
            "AppState initialized"
        );

        Ok(Self {
            llm,
            tools,
            session: RwLock::new(Session::new()),
            peers: Arc::new(Mutex::new(Vec::new())),
            cancel_token: Mutex::new(None),
            config: Arc::new(config),
        })
    }
}
