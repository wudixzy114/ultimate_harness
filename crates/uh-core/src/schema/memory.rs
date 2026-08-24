//! Working Memory — the agent's mutable scratchpad.
//!
//! **Cache constraint**: Working Memory must always be in the
//! variable suffix of the prompt, NEVER in the cacheable prefix.
//! See [`crate::schema::context::ContextLayer::is_cacheable`].

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ids::{PlanId, StepId};
use crate::schema::plan::Retrospective;

/// The agent's mutable working state. This is the one place where
/// "current thinking" lives — and it's also the one place that
/// must NEVER participate in prompt caching.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WorkingMemory {
    pub current_plan_id: Option<PlanId>,
    pub current_step_id: Option<StepId>,
    /// Competing explanations the agent is currently weighing.
    #[serde(default)]
    pub hypotheses: Vec<Hypothesis>,
    /// Open questions the agent is tracking.
    #[serde(default)]
    pub open_questions: Vec<Question>,
    /// Decisions made during this task, with reasoning.
    #[serde(default)]
    pub decisions: Vec<Decision>,
    /// Short, stable facts the agent wants to keep visible.
    /// (Long-lived facts belong in Project/World memory instead.)
    #[serde(default)]
    pub pinned_facts: Vec<PinnedFact>,
    /// The most recent retrospective, for quick reference.
    pub last_retrospective: Option<Retrospective>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Hypothesis {
    pub id: Uuid,
    pub text: String,
    /// 0.0-1.0, the agent's current confidence.
    pub confidence: f32,
    /// Evidence supporting this hypothesis.
    #[serde(default)]
    pub evidence: Vec<String>,
    pub status: HypothesisStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HypothesisStatus {
    Active,
    Confirmed,
    Rejected,
    /// Not yet tested.
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Question {
    pub id: Uuid,
    pub text: String,
    pub blocking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Decision {
    pub id: Uuid,
    /// What was decided.
    pub text: String,
    /// Why.
    pub reason: String,
    /// What alternatives were considered and not chosen.
    #[serde(default)]
    pub alternatives: Vec<String>,
    pub made_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PinnedFact {
    pub id: Uuid,
    pub text: String,
    /// Where this fact came from (file path, tool output, etc.).
    pub source: String,
    pub pinned_at: DateTime<Utc>,
}
