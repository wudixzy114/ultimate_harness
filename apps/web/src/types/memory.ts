// TypeScript mirror of uh-core::schema::memory

import type { PlanId, StepId } from "./plan";
import type { Retrospective } from "./plan";

export type HypothesisStatus = "active" | "confirmed" | "rejected" | "pending";

export interface Hypothesis {
  id: string;
  text: string;
  confidence: number;
  evidence: string[];
  status: HypothesisStatus;
}

export interface Question {
  id: string;
  text: string;
  blocking: boolean;
}

export interface Decision {
  id: string;
  text: string;
  reason: string;
  alternatives: string[];
  made_at: string;
}

export interface PinnedFact {
  id: string;
  text: string;
  source: string;
  pinned_at: string;
}

export interface WorkingMemory {
  current_plan_id: PlanId | null;
  current_step_id: StepId | null;
  hypotheses: Hypothesis[];
  open_questions: Question[];
  decisions: Decision[];
  pinned_facts: PinnedFact[];
  last_retrospective: Retrospective | null;
}
