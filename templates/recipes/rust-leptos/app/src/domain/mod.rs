//! Domain Layer
//!
//! Contains:
//! - `models/` - Domain entities and value objects
//! - `services/` - Business logic (pure functions)
//! - `ports/` - Interfaces for external dependencies
//!
//! This layer is shared between client and server.
//! No IO allowed here - effects are expressed via ports.
//! See: appendix-business-logic-placement.md

pub mod models;
pub mod services;

#[cfg(feature = "ssr")]
pub mod ports;
