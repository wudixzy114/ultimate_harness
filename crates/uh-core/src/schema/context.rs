//! Context snapshot — for cache-aware context management.
//!
//! The context sent to the LLM is split into a **cacheable prefix**
//! and a **variable suffix**. Working Memory and Step Memory MUST
//! live in the variable suffix. This module gives the web frontend
//! a structured view of where the context is.
//!
//! This is the canonical example of "data-flow with side-effects on
//! the cache boundary" — any change to `is_cacheable` flags here
//! is a potential prompt-cache regression.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::SessionId;

/// A snapshot of the current context layout. Emitted to the web
/// frontend so users can see what the LLM "sees" and how much
/// of the context is cacheable.
///
/// # @tag schema observer
/// # @invariant
/// The sum of `layers[i].tokens` for `is_cacheable == true` MUST equal
/// `cache_boundary.prefix_tokens` (cacheable prefix is the union of
/// cacheable layers).
/// The sum for `is_cacheable == false` MUST equal
/// `total_input_tokens - cache_boundary.prefix_tokens`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextSnapshot {
    pub session_id: SessionId,
    pub timestamp: DateTime<Utc>,
    pub model: ModelInfo,
    pub cache_boundary: CacheBoundary,
    pub layers: Vec<ContextLayer>,
    pub total_input_tokens: u32,
    pub max_context_window: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelInfo {
    /// `"openai"`, `"anthropic"`, `"deepseek"`, `"ollama"`, etc.
    pub provider: String,
    /// `"gpt-5"`, `"claude-opus-4"`, `"deepseek-v4-pro"`, etc.
    pub model: String,
    /// Whether this provider supports prompt caching.
    pub supports_caching: bool,
    /// Number of explicit cache breakpoints supported (e.g. 4 for Anthropic).
    /// 0 means auto-caching only.
    pub cache_breakpoints: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheBoundary {
    /// Tokens in the cacheable prefix.
    pub prefix_tokens: u32,
    /// Hash of the cacheable prefix, for cache-hit verification.
    pub prefix_hash: String,
    /// Whether the current provider supports caching.
    pub provider_cache_supported: bool,
    /// Cumulative estimated cache hits.
    pub cache_hits_estimated: u32,
}

/// A single layer of the LLM context. Layers are ordered: cacheable
/// ones first, then the cache boundary, then variable (non-cacheable)
/// ones.
///
/// # @tag schema
/// # @invariant
/// `LayerKind::WorkingMemory` and `LayerKind::StepMemory` MUST have
/// `is_cacheable = false`. All other `LayerKind` variants SHOULD have
/// `is_cacheable = true` to enable prompt caching.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextLayer {
    pub layer: LayerKind,
    pub tokens: u32,
    pub token_budget: u32,
    pub item_count: u32,
    /// **CRITICAL invariant**: `WorkingMemory` and `StepMemory` MUST
    /// be `false`. Other layers SHOULD be `true` to enable caching.
    pub is_cacheable: bool,
}

/// The five memory layers, ordered from outermost (most stable) to
/// innermost (most volatile).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LayerKind {
    /// Cross-project, semantic memory (RAG).
    WorldMemory,
    /// This project's indexed content.
    ProjectMemory,
    /// This session's early turns (older ones may be compressed).
    SessionMemory,
    /// This task's mutable state — plan, hypotheses, decisions.
    /// **Always in variable suffix.**
    WorkingMemory,
    /// This step's request + response. **Always in variable suffix.**
    StepMemory,
}

impl LayerKind {
    /// Whether this layer can participate in the cacheable prefix.
    /// Working and Step memory cannot — they change every turn.
    #[must_use]
    pub const fn is_cacheable_by_default(self) -> bool {
        matches!(
            self,
            Self::WorldMemory | Self::ProjectMemory | Self::SessionMemory
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn working_memory_is_not_cacheable() {
        assert!(!LayerKind::WorkingMemory.is_cacheable_by_default());
        assert!(!LayerKind::StepMemory.is_cacheable_by_default());
    }

    #[test]
    fn outer_layers_are_cacheable() {
        assert!(LayerKind::WorldMemory.is_cacheable_by_default());
        assert!(LayerKind::ProjectMemory.is_cacheable_by_default());
        assert!(LayerKind::SessionMemory.is_cacheable_by_default());
    }
}
