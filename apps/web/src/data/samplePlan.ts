// Sample data that mirrors what `uh` binary prints in v0.0.1.
// In v0.0.3+ this will be replaced by WebSocket-fed real data from the daemon.

import type { Plan } from "../types/plan";
import type { ContextSnapshot } from "../types/context";
import type { WorkingMemory } from "../types/memory";

const now = new Date().toISOString();
const planId = "b44d8a13-3441-45d5-926d-ef4004808510";
const sessionId = "87879051-f46a-4c99-86f3-46644a85412b";

export const SAMPLE_PLAN: Plan = {
  id: planId,
  session_id: sessionId,
  title: "Migrate auth from sessions to JWT with token rotation",
  task: "Migrate auth from sessions to JWT with token rotation",
  goal:
    "Replace session-based auth with JWT + rotation, behind a flag, with 7d backward compat.",
  success_criteria: [
    {
      id: "154d7206-0d82-4ac1-9346-f3d8d08d69ce",
      text: "All existing tests pass with the new auth path",
      verified: false,
    },
    {
      id: "1485ca29-e783-4d7f-b679-12fc92b0cae2",
      text: "No breaking change for current session users within 7d window",
      verified: false,
    },
  ],
  assumptions: [
    {
      id: "1950c40b-6ab7-4c43-a86e-7560c6b5b557",
      text: "Refresh tokens are acceptable per the product spec",
      critical: true,
    },
  ],
  unknowns: [
    {
      id: "dd81e8f7-a3f5-4b75-b416-868464f40ac1",
      text: "Which JWT library to use (jsonwebtoken vs jose vs custom)?",
      blocking: false,
    },
  ],
  steps: [
    {
      id: "64167a41-e137-4383-86f9-a4a151397d76",
      plan_id: planId,
      order: 0,
      title: "Investigate current auth flow",
      intent: "investigate",
      what: "Read auth middleware, session storage, and login routes.",
      how: "Use file read + grep to map all session-related code.",
      risk: "May miss session usages in non-obvious places (e.g. SSR).",
      backout: null,
      depends_on: [],
      status: "pending",
      retrospective: null,
      outputs: [],
    },
    {
      id: "b19a21d5-ed30-40a5-beda-1e590d1d9bac",
      plan_id: planId,
      order: 1,
      title: "Design JWT + rotation interface",
      intent: "implement",
      what: "Add `jwtVerify` helper and `/auth/refresh` with rotating tokens.",
      how: null,
      risk: "Token rotation timing race; mitigate with single-use refresh tokens.",
      backout: "Behind a feature flag; flip back to session middleware.",
      depends_on: [],
      status: "pending",
      retrospective: null,
      outputs: [],
    },
    {
      id: "f685b3c5-efad-491f-b8a6-3e721ffc1b48",
      plan_id: planId,
      order: 2,
      title: "Verify with full test suite",
      intent: "verify",
      what: "Run all auth-related and integration tests.",
      how: null,
      risk: "Tests may be flaky; rerun before reporting failure.",
      backout: null,
      depends_on: [],
      status: "pending",
      retrospective: null,
      outputs: [],
    },
  ],
  status: "pending",
  created_at: now,
  updated_at: now,
};

export const SAMPLE_CONTEXT: ContextSnapshot = {
  session_id: sessionId,
  timestamp: now,
  model: {
    provider: "anthropic",
    model: "claude-opus-4",
    supports_caching: true,
    cache_breakpoints: 4,
  },
  cache_boundary: {
    prefix_tokens: 42000,
    prefix_hash: "blake3:8f3a...",
    provider_cache_supported: true,
    cache_hits_estimated: 17,
  },
  layers: [
    { layer: "world_memory", tokens: 8000, token_budget: 16000, item_count: 23, is_cacheable: true },
    { layer: "project_memory", tokens: 24000, token_budget: 40000, item_count: 156, is_cacheable: true },
    { layer: "session_memory", tokens: 10000, token_budget: 20000, item_count: 8, is_cacheable: true },
    { layer: "working_memory", tokens: 1200, token_budget: 2000, item_count: 5, is_cacheable: false },
    { layer: "step_memory", tokens: 1800, token_budget: 4000, item_count: 1, is_cacheable: false },
  ],
  total_input_tokens: 45000,
  max_context_window: 200000,
};

export const SAMPLE_WORKING_MEMORY: WorkingMemory = {
  current_plan_id: planId,
  current_step_id: null,
  hypotheses: [
    {
      id: "h1",
      text: "Existing session middleware can be replaced wholesale behind a flag",
      confidence: 0.7,
      evidence: ["Only 3 auth routes touched session state", "No Redis dependency yet"],
      status: "active",
    },
    {
      id: "h2",
      text: "Need a token-rotation race-condition mitigation",
      confidence: 0.85,
      evidence: ["Similar issue documented in OAuth 2.0 BCP"],
      status: "active",
    },
  ],
  open_questions: [
    {
      id: "q1",
      text: "Confirm with product: 7d backward compat window or 30d?",
      blocking: false,
    },
  ],
  decisions: [
    {
      id: "d1",
      text: "Use `jsonwebtoken` over `jose`",
      reason: "Smaller bundle, well-audited, async support is enough",
      alternatives: ["jose (more features, larger)", "custom HS256 (too much surface)"],
      made_at: now,
    },
  ],
  pinned_facts: [
    {
      id: "f1",
      text: "Auth middleware lives in src/middleware/auth.ts",
      source: "Step 1 output",
      pinned_at: now,
    },
  ],
  last_retrospective: null,
};
