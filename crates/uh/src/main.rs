//! Ultimate Harness — v0.0.1 entry point.
//!
//! Currently: prints a sample Plan demonstrating the structured
//! iteration principle. The full daemon (with web server, WebSocket,
//! LLM adapter) lands in v0.0.2+.

use anyhow::Result;
use uh_core::ids::{PlanId, SessionId};
use uh_core::schema::plan::{
    Assumption, Criterion, Plan, PlanStatus, Step, StepIntent, StepStatus, Unknown,
};
use uh_skill_plan_first::PlanFirstSkill;
use uh_core::skill::Skill;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info,uh=debug")
        .init();

    tracing::info!("Ultimate Harness v0.0.1");
    tracing::info!("structured iteration is a first-class concern\n");

    // Build a sample Plan to demonstrate the schema.
    let plan = sample_plan();
    let pretty = serde_json::to_string_pretty(&plan)?;
    println!("{pretty}");

    // Show that the plan-first skill loads.
    let skill = PlanFirstSkill::new();
    println!("\n─── loaded skill: {} v{} ───", skill.manifest().name, skill.manifest().version);
    println!("{}", skill.manifest().description);

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn sample_plan() -> Plan {
    use chrono::Utc;

    let session = SessionId::new();
    let plan_id = PlanId::new();
    let now = Utc::now();

    let s1 = Step {
        id: uh_core::ids::StepId::new(),
        plan_id,
        order: 0,
        title: "Investigate current auth flow".into(),
        intent: StepIntent::Investigate,
        what: "Read auth middleware, session storage, and login routes.".into(),
        how: Some("Use file read + grep to map all session-related code.".into()),
        risk: Some("May miss session usages in non-obvious places (e.g. SSR).".into()),
        backout: None,
        depends_on: vec![],
        status: StepStatus::Pending,
        retrospective: None,
        outputs: vec![],
    };

    let s2 = Step {
        id: uh_core::ids::StepId::new(),
        plan_id,
        order: 1,
        title: "Design JWT + rotation interface".into(),
        intent: StepIntent::Implement,
        what: "Add `jwtVerify` helper and `/auth/refresh` with rotating tokens.".into(),
        how: None,
        risk: Some("Token rotation timing race; mitigate with single-use refresh tokens.".into()),
        backout: Some("Behind a feature flag; flip back to session middleware.".into()),
        depends_on: vec![],
        status: StepStatus::Pending,
        retrospective: None,
        outputs: vec![],
    };

    let s3 = Step {
        id: uh_core::ids::StepId::new(),
        plan_id,
        order: 2,
        title: "Verify with full test suite".into(),
        intent: StepIntent::Verify,
        what: "Run all auth-related and integration tests.".into(),
        how: None,
        risk: Some("Tests may be flaky; rerun before reporting failure.".into()),
        backout: None,
        depends_on: vec![],
        status: StepStatus::Pending,
        retrospective: None,
        outputs: vec![],
    };

    Plan {
        id: plan_id,
        session_id: session,
        title: "Migrate auth from sessions to JWT with token rotation".into(),
        task: "Migrate auth from sessions to JWT with token rotation".into(),
        goal: "Replace session-based auth with JWT + rotation, behind a flag, with 7d backward compat.".into(),
        success_criteria: vec![Criterion {
            id: uuid::Uuid::new_v4(),
            text: "All existing tests pass with the new auth path".into(),
            verified: false,
        }, Criterion {
            id: uuid::Uuid::new_v4(),
            text: "No breaking change for current session users within 7d window".into(),
            verified: false,
        }],
        assumptions: vec![Assumption {
            id: uuid::Uuid::new_v4(),
            text: "Refresh tokens are acceptable per the product spec".into(),
            critical: true,
        }],
        unknowns: vec![Unknown {
            id: uuid::Uuid::new_v4(),
            text: "Which JWT library to use (jsonwebtoken vs jose vs custom)?".into(),
            blocking: false,
        }],
        steps: vec![s1, s2, s3],
        status: PlanStatus::Pending,
        created_at: now,
        updated_at: now,
    }
}
