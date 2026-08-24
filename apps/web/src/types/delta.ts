// TypeScript mirror of uh-core::schema::delta

import type { MatchVerdict, StepId, StepIntent, PlanId } from "./plan";

export type DeltaTarget =
  | { plan: PlanId }
  | { step: { plan_id: PlanId; step_id: StepId } };

export interface StepSpec {
  title: string;
  intent: StepIntent;
  what: string;
  how: string | null;
  risk: string | null;
  backout: string | null;
}

export type DeltaAction =
  | { type: "approve_plan" }
  | { type: "reject_plan"; reason: string }
  | { type: "set_goal"; text: string }
  | { type: "add_criterion"; text: string }
  | { type: "remove_criterion"; id: string }
  | { type: "add_assumption"; text: string; critical: boolean }
  | { type: "remove_assumption"; id: string }
  | { type: "add_unknown"; text: string; blocking: boolean }
  | { type: "remove_unknown"; id: string }
  | { type: "start_step" }
  | {
      type: "complete_step";
      actual: string;
      matches_plan: MatchVerdict;
      deviation: string | null;
    }
  | { type: "fail_step"; error: string }
  | { type: "skip_step"; reason: string }
  | { type: "approve_step" }
  | { type: "reject_step"; reason: string }
  | { type: "add_step"; after_step: StepId | null; spec: StepSpec }
  | { type: "remove_step" }
  | { type: "set_step_title"; text: string }
  | { type: "set_step_what"; text: string }
  | { type: "set_step_how"; text: string | null }
  | { type: "set_step_risk"; text: string | null }
  | { type: "set_step_backout"; text: string | null }
  | { type: "set_step_intent"; intent: StepIntent }
  | { type: "reorder_steps"; order: StepId[] }
  | { type: "add_comment"; text: string }
  | { type: "ask_question"; question: string }
  | { type: "pause_plan" }
  | { type: "resume_plan" }
  | { type: "abort_plan"; reason: string };

export interface Delta {
  id: string;
  timestamp: string;
  target: DeltaTarget;
  action: DeltaAction;
  comment: string | null;
}
