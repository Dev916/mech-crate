//! Recipe management
//!
//! Handles parsing, installation, and caching of MechCrate recipes.

mod installer;
mod parser;
pub mod validate;

pub use installer::{InstallResult, RecipeInstaller};
pub use parser::{
    ChmodAction, CreateFile, FileMapping, InitApp, PlaceholderDef, PostInstall, PostInstallAction,
    Recipe, RecipeOption, RecipeService, RenameAction, RunAction,
};
