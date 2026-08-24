//! # plan-first
//!
//! The single v0.1 built-in skill. Its job: force the agent to write
//! a brief plan at every meaningful checkpoint, so the user always
//! knows what the AI is about to do.
//!
//! ## Philosophy
//!
//! > 人最难的事情是知道自己需要什么。
//! > AI 最难的事情是知道答案是什么。
//! > 如果我们不知道我们想要什么，也不知道过程是否满足我们的心意，
//! > 那么最后一定得不到我们想要的东西。
//!
//! This skill doesn't help the LLM be smarter. It forces the LLM to
//! be **explicit** about what it's doing, so the human can correct
//! course early. Detailed implementation will land in v0.0.4.

use async_trait::async_trait;
use uh_core::ids::SkillId;
use uh_core::schema::plan::StepIntent;
use uh_core::skill::{Skill, SkillEvent, SkillManifest, SkillOutput};

pub struct PlanFirstSkill {
    manifest: SkillManifest,
}

impl PlanFirstSkill {
    #[must_use]
    pub fn new() -> Self {
        Self {
            manifest: SkillManifest {
                id: SkillId::new(),
                name: "plan-first".into(),
                version: "0.0.1".into(),
                description: "Force the agent to write a brief plan at every step, so the \
                     user always knows what the AI is about to do. The first and only \
                     built-in skill in v0.1."
                    .into(),
                prefers_intents: vec![
                    StepIntent::Investigate,
                    StepIntent::Implement,
                    StepIntent::Verify,
                ],
            },
        }
    }
}

impl Default for PlanFirstSkill {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for PlanFirstSkill {
    fn manifest(&self) -> &SkillManifest {
        &self.manifest
    }

    async fn on_event(&self, _event: SkillEvent) -> SkillOutput {
        // Stub: real implementation lands in v0.0.4.
        // Will:
        // 1. On TaskSubmitted: ask the LLM to produce a Plan (not start coding).
        // 2. On StepStarting: require a Step Plan if intent is Implement / Verify.
        // 3. On StepFinishing: require a Retrospective.
        // 4. Reject any tool call whose plan doesn't have a current step.
        SkillOutput {
            notes: vec!["plan-first v0.0.1: skeleton, no enforcement yet".into()],
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_is_stable() {
        let skill = PlanFirstSkill::new();
        let m = skill.manifest();
        assert_eq!(m.name, "plan-first");
        assert_eq!(m.version, "0.0.1");
    }
}
