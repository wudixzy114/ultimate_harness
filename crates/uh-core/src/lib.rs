//! # uh-core
//!
//! The "stable structure" of Ultimate Harness.
//!
//! Everything that flows between LLM, daemon, and web frontend passes
//! through one of the schema types defined here. These types are the
//! single source of truth — the web frontend mirrors them in TypeScript
//! (see `apps/web/src/types/`), and the LLM is expected to emit JSON
//! that conforms to them.
//!
//! ## Design constraint
//!
//! Any "object" that needs to survive across turns, be displayed in UI,
//! or be modified by the user **must** be a schema type here. Natural
//! language prose is only allowed for content (tool outputs, plan
//! descriptions) — never for state.

pub mod ids;
pub mod schema;
pub mod skill;

pub use ids::*;
pub use schema::*;
pub use skill::*;
