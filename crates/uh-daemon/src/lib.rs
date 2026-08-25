//! `uh-daemon` — HTTP / WebSocket server and the agent loop.
//!
//! Architecture (v0.0.3):
//!
//! ```text
//!   axum::Router
//!     ├── GET  /             → serve web/dist
//!     ├── GET  /api/health   → health check
//!     ├── GET  /api/llm      → LLM info
//!     └── GET  /ws           → WebSocket upgrade
//!
//!   On each WS message:
//!     Router.dispatch()  →  AppState method
//!   The agent loop (run_turn) drives Message exchange with the Llm.
//! ```

pub mod config;
pub mod loop_;
pub mod peers;
pub mod router;
pub mod server;
pub mod state;

pub use config::*;
pub use server::serve;
pub use state::*;
