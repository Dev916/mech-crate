//! Recipe management
//!
//! Handles parsing, installation, and caching of MechCrate recipes.

mod parser;
mod installer;

pub use parser::{
    Recipe, RecipeOption, RecipeService, FileMapping, PostInstall, PostInstallAction,
    PlaceholderDef, InitApp, CreateFile, RenameAction, ChmodAction, RunAction,
};
pub use installer::{RecipeInstaller, InstallResult};
