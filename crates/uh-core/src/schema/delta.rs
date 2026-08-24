//! Delta types — user interactions with Plans and Steps.
//!
//! Every change a user makes to a Plan goes through a `Delta`. This
//! makes the audit trail complete and lets the LLM react to minimal,
//! structured inputs instead of re-reading the whole plan each turn.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ids::{PlanId, StepId};
use crate::schema::plan::{MatchVerdict, StepIntent};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Delta {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub target: DeltaTarget,
    pub action: DeltaAction,
    /// Optional user comment attached to this delta.
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeltaTarget {
    Plan(PlanId),
    Step { plan_id: PlanId, step_id: StepId },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeltaAction {
    // ─── Plan-level ─────────────────────────────────────────────
    ApprovePlan,
    RejectPlan {
        reason: String,
    },
    SetGoal {
        text: String,
    },
    AddCriterion {
        text: String,
    },
    RemoveCriterion {
        id: Uuid,
    },
    AddAssumption {
        text: String,
        critical: bool,
    },
    RemoveAssumption {
        id: Uuid,
    },
    AddUnknown {
        text: String,
        blocking: bool,
    },
    RemoveUnknown {
        id: Uuid,
    },

    // ─── Step lifecycle ─────────────────────────────────────────
    StartStep,
    CompleteStep {
        actual: String,
        matches_plan: MatchVerdict,
        deviation: Option<String>,
    },
    FailStep {
        error: String,
    },
    SkipStep {
        reason: String,
    },
    ApproveStep,
    RejectStep {
        reason: String,
    },

    // ─── Step editing ───────────────────────────────────────────
    AddStep {
        after_step: Option<StepId>,
        spec: StepSpec,
    },
    RemoveStep,
    SetStepTitle {
        text: String,
    },
    SetStepWhat {
        text: String,
    },
    SetStepHow {
        text: Option<String>,
    },
    SetStepRisk {
        text: Option<String>,
    },
    SetStepBackout {
        text: Option<String>,
    },
    SetStepIntent {
        intent: StepIntent,
    },
    ReorderSteps {
        order: Vec<StepId>,
    },

    // ─── Free-form ──────────────────────────────────────────────
    AddComment {
        text: String,
    },
    AskQuestion {
        question: String,
    },

    // ─── Flow control ───────────────────────────────────────────
    PausePlan,
    ResumePlan,
    AbortPlan {
        reason: String,
    },
}

/// Template used when adding a new step. Doesn't carry an ID —
/// the system assigns one when the step is created.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepSpec {
    pub title: String,
    pub intent: StepIntent,
    pub what: String,
    pub how: Option<String>,
    pub risk: Option<String>,
    pub backout: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_action_is_tagged() {
        let action = DeltaAction::SetGoal {
            text: "new goal".into(),
        };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("\"type\":\"set_goal\""));
    }
}
