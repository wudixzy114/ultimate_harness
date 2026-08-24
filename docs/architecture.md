# Ultimate Harness — Architecture

> v0.0.1, snapshot of the current design. Will evolve as we iterate.

## 1. First Principles

### 1.1 Structured iteration is a first-class concern

> **LLM 负责生成内容，结构负责承载状态。**

Any object that needs to (a) survive across turns, (b) be displayed in UI, or
(c) be modified by the user MUST be a schema type in `uh-core`. Natural language
prose is only for content (tool outputs, descriptions) — never for state.

### 1.2 Cache-aware context

Working Memory and Step Memory live in the **variable suffix** of the prompt.
The other three layers (World, Project, Session) live in the **cacheable prefix**.
Putting mutable state inside the cacheable prefix would break prompt caching on
every modification.

### 1.3 Plan-first

The first and only built-in skill (`uh-skill-plan-first`) forces the agent to
emit a structured `Plan` before any non-trivial work, and a `StepPlan` (via
retrospective + delta) before any mutating action. The user always knows what
the AI is about to do.

## 2. Repo Layout

```
ultimate-harness/
├── Cargo.toml                       # workspace root
├── crates/
│   ├── uh-core/                     # schema + traits (no behavior)
│   ├── uh-skill-plan-first/         # the only v0.1 built-in skill
│   └── uh/                          # binary entry point
├── apps/
│   └── web/                         # the only frontend (Vite + React + TS)
└── docs/
```

## 3. The Schema (uh-core/src/schema/)

| Type | Purpose | Lives in |
|------|---------|----------|
| `Plan` | Top-level iteration object: goal, criteria, assumptions, steps | prompt body |
| `Step` | One unit of work: intent, what, how, risk, backout | prompt body |
| `Retrospective` | What actually happened vs. plan (required for Done/Failed) | prompt body |
| `Delta` | User → system: approve, reject, edit, comment, ask | incoming |
| `WorkingMemory` | Mutable scratchpad (hypotheses, decisions, pinned facts) | variable suffix only |
| `ContextSnapshot` | Cacheable/variable split for visualization | UI only |
| `Skill` (trait) | Plugin interface | n/a |
| `SkillEvent` | Lifecycle event emitted into skills | bus |

### 3.1 Invariants enforced by types

1. **`Step::retrospective`** is `Option<Retrospective>`, but the agent runtime
   requires it to be `Some(_)` when status is `Done` or `Failed`. Enforced at
   runtime, not at the type level (Rust can't do this without GATs).
2. **`WorkingMemory`** never appears in `ContextSnapshot::layers` with
   `is_cacheable: true`. Encoded as a docstring + runtime assertion in
   `LayerKind::is_cacheable_by_default`.
3. **All Deltas target a specific Plan or Step** via `DeltaTarget`. No
   "global" deltas.

## 4. Crate Dependency Graph

```
           ┌──────────┐
           │  uh-core │  (schema + traits, no behavior)
           └─────┬────┘
              ┌──┴──┬──────────────┐
              │     │              │
   ┌──────────┴┐  ┌─┴───────────┐ ┌┴─────────────────────────┐
   │ uh-skill- │  │  (future)   │ │ (future)                  │
   │ plan-     │  │  llm crate  │ │ tool-*, sandbox-*, etc.   │
   │ first     │  └─────────────┘ └───────────────────────────┘
   └─────┬─────┘
         │
       ┌─┴────┐
       │  uh  │  (binary: starts daemon, serves web)
       └──────┘
```

## 5. Borrowing Decisions (vs Grok Build & DeepSeek Harness)

| Concept | Source | How we use it |
|---------|--------|---------------|
| OS-native sandbox | Grok Build | Defer to v0.3; same shape (Landlock/Seatbelt/Win32 Job Objects) |
| `Model-Visible ⟺ Logged` invariant | DeepSeek Harness | Adopt as core principle; implement in v0.0.5+ |
| 7-stage tool pipeline | DeepSeek Harness | v0.0.5+; will start with 3 stages, expand to 7 |
| Capability Seam (Definition/Provider/Consumer) | DeepSeek Harness | Adopt pattern; v0.0.5+ for LLM, then for each tool |
| 4 layer event bus (emit/waterfall/parallel/serial) | DeepSeek Harness | Adopt; implement in v0.0.5+ |
| rustls + aws-lc-rs | Grok Build | Confirmed for v0.0.2+ |
| Append-only SessionEvent log | DeepSeek Harness | Adopt; v0.0.5+ |

## 6. What we explicitly DON'T do (zero-cost abstraction)

- No TUI / CLI — only web frontend
- No subagent runtime
- No multi-agent DAG (visualisation only, v0.2+)
- No game-dev skills (deferred indefinitely)
- No SE skill presets (only `plan-first`; others from scratch later)
- No external plugin ABI (skills are workspace crates for v0.1)
- No built-in workflow orchestration (user writes skills)

## 7. Versioning

We use `0.0.X` for incomplete features, `0.X.0` for first usable of a feature,
`X.0.0` for API-stable. Currently at `0.0.1`. See root `README.md` for the
release train.

## 8. Open Questions for the User

1. TypeScript type source of truth: keep manual mirror, or switch to `ts-rs`?
2. Working Memory rendering: keep as a sidebar panel, or fold into PlanPanel?
3. When to introduce WebSocket: ship `v0.0.2` with file-backed IPC first, or
   jump straight to WS?
4. Skill discovery: hard-code in `uh-core`, or load from a `presets/` dir?
