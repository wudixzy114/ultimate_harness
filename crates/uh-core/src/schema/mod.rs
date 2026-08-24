//! Core schema types — the "stable structure" of Ultimate Harness.
//!
//! Every object that flows between LLM, daemon, and web frontend is
//! defined here. These types are the single source of truth.
//!
//! ## Invariants
//!
//! 1. **Working Memory is never inside the cacheable prefix.** See
//!    [`context::ContextLayer::is_cacheable`] — L4 (`WorkingMemory`) and
//!    L5 (`StepMemory`) MUST be `false`. Putting them inside the
//!    cacheable prefix would break prompt caching on every modification.
//! 2. **All plans and steps carry retrospectives.** When a step ends,
//!    it must include a `Retrospective` so the user can see what
//!    actually happened vs. what was planned.
//! 3. **All Deltas are explicit.** User interactions with the system
//!    are represented as a `Delta` — no implicit state changes.

pub mod context;
pub mod delta;
pub mod memory;
pub mod plan;

pub use context::*;
pub use delta::*;
pub use memory::*;
pub use plan::*;
