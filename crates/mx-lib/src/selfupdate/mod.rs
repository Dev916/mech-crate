//! Self-update engine for the mx client.
//!
//! Pure core (no IO): [`version`], [`target`], [`kind`], [`plan`].
//! Effectful shell: [`index`], [`fetch`] (GitHub + download, 4vp.4);
//! [`layout`], [`verify`], [`refresh`] (install layout + post-update, 4vp.5).
//!
//! Design: `docs/superpowers/specs/2026-09-02-mx-self-update-design.md` §3.4.

pub mod fetch;
pub mod index;
pub mod kind;
pub mod layout;
pub mod plan;
pub mod refresh;
pub mod target;
pub mod verify;
pub mod version;

pub use kind::{detect, InstallKind};
pub use plan::{plan, UpdatePlan, BREW_UPGRADE};
pub use target::{asset_name, bundle_dir_name, checksum_name, Triple};
pub use version::{current, is_newer, parse, Version};
