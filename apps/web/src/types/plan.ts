// TypeScript mirror of uh-core::schema::plan
// These types MUST stay in sync with the Rust schema.
// (In v0.1+ we may switch to ts-rs for auto-generation.)

export type PlanId = string;
export type StepId = string;
export type SessionId = string;

export type StepIntent =
  | "investigate"
  | "implement"
  | "verify"
  | "ask_user"
  | "review"
  | "communicate";

export type StepStatus =
  | "pending"
  | "in_progress"
  | "done"
  | "failed"
  | "skipped"
  | "blocked";

export type PlanStatus =
  | "draft"
  | "pending"
  | "approved"
  | "in_progress"
  | "paused"
  | "done"
  | "failed"
  | "aborted";

export type MatchVerdict = "match" | "partial" | "diverged";

export type OutputKind =
  | "file_created"
  | "file_modified"
  | "file_deleted"
  | "command_run"
  | "test_run"
  | "search_result"
  | "note";

export interface Criterion {
  id: string;
  text: string;
  verified: boolean;
}

export interface Assumption {
  id: string;
  text: string;
  critical: boolean;
}

export interface Unknown {
  id: string;
  text: string;
  blocking: boolean;
}

export interface StepOutput {
  kind: OutputKind;
  path: string | null;
  summary: string;
}

export interface Retrospective {
  done: string;
  matches_plan: MatchVerdict;
  deviation: string | null;
  next_step_ok: boolean;
  notes: string | null;
}

export interface Step {
  id: StepId;
  plan_id: PlanId;
  order: number;
  title: string;
  intent: StepIntent;
  what: string;
  how: string | null;
  risk: string | null;
  backout: string | null;
  depends_on: StepId[];
  status: StepStatus;
  retrospective: Retrospective | null;
  outputs: StepOutput[];
}

export interface Plan {
  id: PlanId;
  session_id: SessionId;
  title: string;
  task: string;
  goal: string;
  success_criteria: Criterion[];
  assumptions: Assumption[];
  unknowns: Unknown[];
  steps: Step[];
  status: PlanStatus;
  created_at: string;
  updated_at: string;
}
