//! Environment utilities
//!
//! Helpers for managing environment variables and PATH.

/// Ensure PATH includes common binary locations
///
/// This is needed because when running as a subprocess or from certain contexts,
/// the PATH may not include directories like /usr/local/bin where Docker is installed.
pub fn ensure_path() -> String {
    let current_path = std::env::var("PATH").unwrap_or_default();

    // Common paths that might contain docker, make, etc.
    let extra_paths = [
        "/usr/local/bin",
        "/opt/homebrew/bin",
        "/usr/bin",
        "/bin",
        "/usr/sbin",
        "/sbin",
    ];

    let mut paths: Vec<&str> = current_path.split(':').collect();

    for extra in extra_paths {
        if !paths.contains(&extra) {
            paths.push(extra);
        }
    }

    paths.join(":")
}

/// Get a PATH that includes the user's home bin directories
pub fn ensure_full_path() -> String {
    let mut path = ensure_path();

    // Add user-specific paths
    if let Ok(home) = std::env::var("HOME") {
        let user_paths = [
            format!("{home}/.local/bin"),
            format!("{home}/bin"),
            format!("{home}/.cargo/bin"),
        ];

        for user_path in user_paths {
            if !path.contains(&user_path) {
                path = format!("{user_path}:{path}");
            }
        }
    }

    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_path_includes_usr_local_bin() {
        let path = ensure_path();
        assert!(path.contains("/usr/local/bin"));
    }

    #[test]
    fn test_ensure_path_includes_opt_homebrew() {
        let path = ensure_path();
        assert!(path.contains("/opt/homebrew/bin"));
    }

    #[test]
    fn test_ensure_full_path_includes_cargo() {
        std::env::set_var("HOME", "/Users/test");
        let path = ensure_full_path();
        assert!(path.contains(".cargo/bin"));
    }
}
