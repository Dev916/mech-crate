//! Domain Services
//!
//! Pure business logic functions.
//! No IO or side effects - those are handled by ports/adapters.
//! See: appendix-algebraic-effects-optics.md

mod user_service;

pub use user_service::*;
