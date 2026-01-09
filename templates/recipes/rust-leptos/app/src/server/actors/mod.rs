//! Actor Model Implementation
//!
//! Actors for stateful entities following the Actor Model principles:
//! - Each actor has private state
//! - Communication via messages only
//! - One message processed at a time (no data races)
//! - Supervision for fault tolerance
//!
//! See: appendix-actor-model.md

mod session;
mod supervisor;

pub use session::{SessionActor, SessionManager};
pub use supervisor::SupervisorActor;
