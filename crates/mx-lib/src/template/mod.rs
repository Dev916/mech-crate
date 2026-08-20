//! Template processing and interpolation
//!
//! Two layers, deliberately separate:
//!
//! * [`expand_placeholders`] — the narrow `{{NAME}}` substitution the recipe
//!   installer uses. It never interprets foreign template syntax, so recipe app
//!   sources (Blade, Vue, Zola) survive installation byte-for-byte.
//! * [`TemplateEngine`] — full Tera rendering with MechCrate's custom filters,
//!   for inputs that are genuinely templates.

mod engine;
mod placeholder;

pub use engine::TemplateEngine;
pub use placeholder::expand_placeholders;
