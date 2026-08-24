//! Plan and Step types — the heart of structured iteration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ids::{PlanId, SessionId, StepId};

/// A Plan is the top-level iteration object. The LLM emits one when
/// it receives a task; the user approves/modifies it via Deltas; the
/// LLM executes its steps and reports retrospectives.
///
/// # Example
///
/// ```json
/// {
///   "id": "...",
///   "title": "Refactor auth module to use JWT",
///   "task": "Migrate auth from sessions to JWT with token rotation",
///   "goal": "Replace session-based auth with JWT + rotation, behind a flag",
///   "success_criteria": [
///     { "id": "...", "text": "All existing tests pass", "verified": false }
///   ],
///   "assumptions": [
///     { "id": "...", "text": "Token rotation is acceptable per spec", "critical": true }
///   ],
///   "steps": [...],
///   "status": "Pending"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Plan {
    pub id: PlanId,
    pub session_id: SessionId,
    /// One-line title for display in lists.
    pub title: String,
    /// The original user task (verbatim or paraphrased).
    pub task: String,
    /// What we want to achieve. 1-2 sentences.
    pub goal: String,
    /// How we know we're done. Each criterion is independently checkable.
    pub success_criteria: Vec<Criterion>,
    /// What we believe is true. If a critical assumption is wrong, the plan fails.
    pub assumptions: Vec<Assumption>,
    /// What we don't know. A blocking unknown must be resolved before execution.
    pub unknowns: Vec<Unknown>,
    /// Ordered execution steps.
    pub steps: Vec<Step>,
    pub status: PlanStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Criterion {
    pub id: Uuid,
    pub text: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Assumption {
    pub id: Uuid,
    pub text: String,
    /// If true, an invalid assumption is a hard failure.
    pub critical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Unknown {
    pub id: Uuid,
    pub text: String,
    /// If true, this must be resolved before the plan can execute.
    pub blocking: bool,
}

/// A single step in a Plan. Steps are ordered by `order` and may have
/// additional `depends_on` constraints for non-linear dependencies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Step {
    pub id: StepId,
    pub plan_id: PlanId,
    pub order: u32,
    /// One-line title for display.
    pub title: String,
    pub intent: StepIntent,
    /// 1-2 sentences: what this step achieves.
    pub what: String,
    /// Optional: how we'll do it.
    pub how: Option<String>,
    /// Optional: what could go wrong.
    pub risk: Option<String>,
    /// Optional: how to roll back if this step fails.
    pub backout: Option<String>,
    /// Non-linear dependencies (beyond `order`).
    #[serde(default)]
    pub depends_on: Vec<StepId>,
    pub status: StepStatus,
    /// Required when status is `Done` or `Failed`. Captures what
    /// actually happened vs. what was planned.
    pub retrospective: Option<Retrospective>,
    /// Side-effect summary (files touched, commands run, etc.).
    #[serde(default)]
    pub outputs: Vec<StepOutput>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepIntent {
    /// Read, search, understand. No side effects.
    Investigate,
    /// Write, edit, create. Has side effects.
    Implement,
    /// Run tests, check result, validate.
    Verify,
    /// Block on user input.
    AskUser,
    /// Check, lint, audit, review.
    Review,
    /// Just talk, no side effect.
    Communicate,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    InProgress,
    Done,
    Failed,
    Skipped,
    /// Blocked by an unknown or a dependency.
    Blocked,
}

/// What actually happened when a step finished. Required for Done/Failed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Retrospective {
    /// Free-form: what was done, with specifics.
    pub done: String,
    /// How well the execution matched the plan.
    pub matches_plan: MatchVerdict,
    /// If not a match, what was different.
    pub deviation: Option<String>,
    /// Should the next step proceed as planned, or do we need to re-plan?
    pub next_step_ok: bool,
    /// Any other notes the LLM wants to flag.
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MatchVerdict {
    Match,
    Partial,
    Diverged,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepOutput {
    pub kind: OutputKind,
    /// File path, command string, or other reference.
    pub path: Option<String>,
    /// Short summary (≤1 line).
    pub summary: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputKind {
    FileCreated,
    FileModified,
    FileDeleted,
    CommandRun,
    TestRun,
    SearchResult,
    Note,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    /// LLM is still constructing the plan.
    Draft,
    /// Waiting for user approval.
    Pending,
    /// User approved, ready to execute.
    Approved,
    /// Currently being executed.
    InProgress,
    /// User paused; can be resumed.
    Paused,
    Done,
    Failed,
    /// User cancelled; no resume.
    Aborted,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::SessionId;

    #[test]
    fn plan_serializes_to_json() {
        let session = SessionId::new();
        let plan = Plan {
            id: PlanId::new(),
            session_id: session,
            title: "Test".into(),
            task: "Test task".into(),
            goal: "Test goal".into(),
            success_criteria: vec![],
            assumptions: vec![],
            unknowns: vec![],
            steps: vec![],
            status: PlanStatus::Draft,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains("Test task"));
    }

    #[test]
    fn step_intent_snake_case() {
        let intent = StepIntent::Investigate;
        let json = serde_json::to_string(&intent).unwrap();
        assert_eq!(json, "\"investigate\"");
    }
}
