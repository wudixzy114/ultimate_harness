//! Skill trait — the plugin interface.
//!
//! A Skill is a structured capability that the agent can invoke.
//! It observes Plan/Step lifecycle events and may emit Plans or
//! transform the Working Memory.
//!
//! The trait is the **only** legal way for a third-party capability
//! to touch core state. Concrete skills live in their own crates
//! (e.g. `uh-skill-plan-first`).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::ids::SkillId;
use crate::schema::context::ContextSnapshot;
use crate::schema::delta::{Delta, StepSpec};
use crate::schema::memory::WorkingMemory;
use crate::schema::plan::{Plan, Retrospective, Step};

/// Metadata describing a Skill (used by UI and loader).
///
/// # @tag schema
/// # @invariant
/// `id` MUST be unique across the loaded skill registry.
/// `version` MUST be a semver string; the loader rejects duplicates
/// of equal `id` but mismatched `version` unless `--force` is set.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillManifest {
    pub id: SkillId,
    pub name: String,
    pub version: String,
    pub description: String,
    /// What intent types this skill is best for.
    #[serde(default)]
    pub prefers_intents: Vec<crate::schema::plan::StepIntent>,
}

/// Result of a skill invocation: the skill may mutate Plan/WorkingMemory,
/// or simply observe and return context updates.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SkillOutput {
    /// Updated Plan (if the skill produced or modified one).
    pub plan: Option<Plan>,
    /// Updated Working Memory (if the skill mutated it).
    pub working_memory: Option<WorkingMemory>,
    /// Optional retrospective for the step that triggered this skill.
    pub retrospective: Option<Retrospective>,
    /// Human-readable notes for the user.
    #[serde(default)]
    pub notes: Vec<String>,
    /// A new step to insert after the current one (rare; for "iterate" skills).
    pub next_step: Option<StepSpec>,
}

/// Events the agent loop fires into skills. Each maps to a UI action.
///
/// # @tag schema executor
/// # @data-flow
/// input: agent loop state (Plan / Step / Delta / Context)
/// output: SkillEvent (discriminated by `type`)
/// depends-on: Plan / Step / Delta / Context lifecycle
/// # @invariant
/// Every `Plan` referenced inside a `SkillEvent` MUST already be
/// persisted in the session store before the event is dispatched —
/// skills are observers, not writers of truth.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SkillEvent {
    /// User just submitted a task; the LLM should produce a Plan.
    TaskSubmitted { task: String },
    /// Plan is ready; user is reviewing.
    PlanReady { plan: Plan },
    /// User submitted a Delta (approve / reject / edit).
    DeltaSubmitted { delta: Delta },
    /// Step is starting execution.
    StepStarting { plan_id: crate::ids::PlanId, step: Step },
    /// Step is finishing; the LLM should produce a retrospective.
    StepFinishing { plan_id: crate::ids::PlanId, step: Step },
    /// Context is being assembled for a new turn.
    ContextAssembling {
        snapshot: ContextSnapshot,
        working_memory: WorkingMemory,
    },
}

/// Trait every skill implements. The default trait methods do nothing
/// (a skill only needs to override events it cares about).
#[async_trait]
pub trait Skill: Send + Sync {
    fn manifest(&self) -> &SkillManifest;

    /// Called on every event. Default: no-op.
    async fn on_event(&self, _event: SkillEvent) -> SkillOutput {
        SkillOutput::default()
    }
}

