//! Path resolution for MechCrate
//!
//! Handles dynamic resolution of the MechCrate root directory and templates.
//! Priority order:
//! 1. MECH_CRATE_ROOT environment variable (for development/override)
//! 2. ~/.mech-crate/ (standard installation location)
//! 3. Relative to the executable (for portable/dev builds)

use std::path::PathBuf;

use crate::error::{Error, Result};

/// Standard installation directory name
pub const INSTALL_DIR_NAME: &str = ".mech-crate";

/// Get the MechCrate home directory (~/.mech-crate/)
pub fn home_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|h| h.join(INSTALL_DIR_NAME))
        .ok_or_else(|| Error::Config("Could not determine home directory".into()))
}

/// Get the templates directory
/// 
/// Resolution order:
/// 1. MECH_CRATE_ROOT/templates (if env var set)
/// 2. ~/.mech-crate/templates (standard install)
/// 3. Relative to executable (dev/portable mode)
pub fn templates_dir() -> Result<PathBuf> {
    // 1. Check MECH_CRATE_ROOT env var (development override)
    if let Ok(root) = std::env::var("MECH_CRATE_ROOT") {
        let templates = PathBuf::from(&root).join("templates");
        if templates.exists() {
            return Ok(templates);
        }
    }

    // 2. Check ~/.mech-crate/templates (standard install)
    if let Ok(home) = home_dir() {
        let templates = home.join("templates");
        if templates.exists() {
            return Ok(templates);
        }
    }

    // 3. Check relative to executable (portable/dev mode)
    if let Ok(exe) = std::env::current_exe() {
        let mut current = exe.parent();
        while let Some(dir) = current {
            let templates = dir.join("templates");
            if templates.exists() {
                return Ok(templates);
            }
            current = dir.parent();
        }
    }

    Err(Error::Config(
        "Templates not found. Run 'mx init' to install MechCrate.".into()
    ))
}

/// Get the recipes directory
pub fn recipes_dir() -> Result<PathBuf> {
    templates_dir().map(|t| t.join("recipes"))
}

/// Check if MechCrate is initialized
pub fn is_initialized() -> bool {
    if let Ok(home) = home_dir() {
        home.join("templates").join("recipes").exists()
    } else {
        false
    }
}

/// Get the source templates directory (for `mx init`)
/// 
/// This finds the templates in the original installation source:
/// - From MECH_CRATE_ROOT if set
/// - Relative to the executable
pub fn source_templates_dir() -> Result<PathBuf> {
    // Check MECH_CRATE_ROOT first (development)
    if let Ok(root) = std::env::var("MECH_CRATE_ROOT") {
        let templates = PathBuf::from(&root).join("templates");
        if templates.exists() {
            return Ok(templates);
        }
    }

    // Check relative to executable
    if let Ok(exe) = std::env::current_exe() {
        let mut current = exe.parent();
        while let Some(dir) = current {
            let templates = dir.join("templates");
            if templates.exists() {
                return Ok(templates);
            }
            current = dir.parent();
        }
    }

    Err(Error::Config(
        "Source templates not found. Cannot initialize.".into()
    ))
}

/// Get the MechCrate root directory (source repo or install location)
///
/// Resolution order:
/// 1. MECH_CRATE_ROOT environment variable
/// 2. Walk up from executable to find a directory containing `scripts/`
pub fn mech_crate_root() -> Result<PathBuf> {
    // 1. Check MECH_CRATE_ROOT env var
    if let Ok(root) = std::env::var("MECH_CRATE_ROOT") {
        let root = PathBuf::from(&root);
        if root.exists() {
            return Ok(root);
        }
    }

    // 2. Walk up from executable (resolve symlinks first)
    if let Ok(exe) = std::env::current_exe() {
        let exe = exe.canonicalize().unwrap_or(exe);
        let mut current = exe.parent();
        while let Some(dir) = current {
            if dir.join("scripts").is_dir() {
                return Ok(dir.to_path_buf());
            }
            current = dir.parent();
        }
    }

    Err(Error::Config(
        "MechCrate root not found. Set MECH_CRATE_ROOT or run from a MechCrate installation.".into(),
    ))
}

/// Get a subdirectory within the MechCrate home
pub fn subdir(name: &str) -> Result<PathBuf> {
    home_dir().map(|h| h.join(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_dir_ends_with_mech_crate() {
        let home = home_dir().unwrap();
        assert!(home.ends_with(".mech-crate"));
    }

    #[test]
    fn test_subdir_appends_correctly() {
        let config = subdir("config").unwrap();
        assert!(config.ends_with(".mech-crate/config"));
    }

    #[test]
    fn test_install_dir_name_constant() {
        assert_eq!(INSTALL_DIR_NAME, ".mech-crate");
    }

    #[test]
    fn test_templates_dir_with_env_var() {
        // When MECH_CRATE_ROOT is set and has templates, that should be used
        let original = std::env::var("MECH_CRATE_ROOT").ok();
        
        // Create a temp dir structure
        let temp_dir = tempfile::tempdir().unwrap();
        let templates = temp_dir.path().join("templates");
        std::fs::create_dir_all(&templates).unwrap();
        
        std::env::set_var("MECH_CRATE_ROOT", temp_dir.path());
        
        let result = templates_dir();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), templates);
        
        // Restore original
        match original {
            Some(val) => std::env::set_var("MECH_CRATE_ROOT", val),
            None => std::env::remove_var("MECH_CRATE_ROOT"),
        }
    }
}
