//! Traefik router management
//!
//! Manages the global Traefik router for local development.

use std::net::TcpListener;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::config::MechCrateConfig;
use crate::docker::Compose;
use crate::error::{Error, Result};
use crate::paths;

/// Router state file names
const DASHBOARD_PORT_FILE: &str = ".dashboard-port";
const NETWORK_NAME: &str = "devmesh-traefik";

/// Dashboard port allocation range (matches shell implementation and docker-compose template)
const DASHBOARD_PORT_START: u16 = 7680;
const DASHBOARD_PORT_END: u16 = 7799;

/// Router manager
#[derive(Debug)]
pub struct Router {
    config: MechCrateConfig,
}

impl Router {
    /// Create a new router manager
    pub fn new(config: MechCrateConfig) -> Self {
        Self { config }
    }

    /// Get the router installation directory
    pub fn install_dir(&self) -> PathBuf {
        self.config.router_dir()
    }

    /// Check if router is installed
    pub fn is_installed(&self) -> bool {
        self.install_dir().join("docker-compose.yml").exists()
    }

    /// Get the Docker network name
    pub fn network_name(&self) -> &'static str {
        NETWORK_NAME
    }

    /// Ensure the Docker network exists
    pub fn ensure_network(&self) -> Result<()> {
        if !crate::docker::Docker::network_exists(NETWORK_NAME) {
            crate::docker::Docker::create_network(NETWORK_NAME)?;
        }
        Ok(())
    }

    /// Find a free port in the given range
    pub fn find_free_port(&self, start: u16, end: u16) -> Result<u16> {
        for port in start..=end {
            if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)) {
                drop(listener);
                return Ok(port);
            }
        }
        Err(Error::Other(format!(
            "No free port found in range {}-{}",
            start, end
        )))
    }

    /// Get or allocate the dashboard port
    pub fn dashboard_port(&self) -> Result<u16> {
        let port_file = self.install_dir().join(DASHBOARD_PORT_FILE);

        if port_file.exists() {
            let content = std::fs::read_to_string(&port_file)?;
            if let Ok(port) = content.trim().parse::<u16>() {
                // Validate cached port is within the expected range
                if (DASHBOARD_PORT_START..=DASHBOARD_PORT_END).contains(&port) {
                    return Ok(port);
                }
                // Stale port outside expected range -- re-allocate
                tracing::warn!(
                    "Cached dashboard port {} is outside range {}-{}, re-allocating",
                    port,
                    DASHBOARD_PORT_START,
                    DASHBOARD_PORT_END
                );
            }
        }

        // Allocate a new port in the correct range
        let port = self.find_free_port(DASHBOARD_PORT_START, DASHBOARD_PORT_END)?;
        std::fs::write(&port_file, port.to_string())?;
        Ok(port)
    }

    /// Install the router from templates
    pub fn install(&self) -> Result<()> {
        // Get source templates directory
        let templates_dir = paths::templates_dir()?;
        let router_template = templates_dir.join("router");

        if !router_template.exists() {
            return Err(Error::Config(format!(
                "Router template not found at {}. Run 'mx init' first.",
                router_template.display()
            )));
        }

        let install_dir = self.install_dir();

        // Create install directory
        std::fs::create_dir_all(&install_dir)?;

        // Copy all files from template
        self.copy_dir(&router_template, &install_dir)?;

        // Set proper permissions on acme.json
        let acme_json = install_dir.join("letsencrypt").join("acme.json");
        if acme_json.exists() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&acme_json, std::fs::Permissions::from_mode(0o600))?;
            }
        }

        // Try to create the network (non-fatal if Docker isn't available)
        if let Err(e) = self.ensure_network() {
            tracing::warn!(
                "Could not create Docker network: {}. Will retry on start.",
                e
            );
        }

        Ok(())
    }

    /// Copy directory recursively
    fn copy_dir(&self, from: &Path, to: &Path) -> Result<()> {
        for entry in WalkDir::new(from) {
            let entry = entry.map_err(|e| Error::Io(e.into()))?;
            let relative = entry
                .path()
                .strip_prefix(from)
                .map_err(|e| Error::Other(e.to_string()))?;
            let dest = to.join(relative);

            if entry.file_type().is_dir() {
                std::fs::create_dir_all(&dest)?;
            } else if entry.file_type().is_file() {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(entry.path(), &dest)?;
            }
        }
        Ok(())
    }

    /// Check if router is running
    pub fn is_running(&self) -> bool {
        if !self.is_installed() {
            return false;
        }

        // Important: we must query the same compose *project* name we use for `up`/`down`.
        // Otherwise `docker compose ps` may look at a different project (derived from CWD),
        // and incorrectly report the router as stopped even when it's running.
        let compose = Compose::new(self.install_dir())
            .with_file("docker-compose.yml")
            .with_project_name("mx-router");

        let output = compose.run(&["ps", "-q"]);

        output
            .map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty())
            .unwrap_or(false)
    }

    /// Start the router
    pub fn start(&self) -> Result<()> {
        if !self.is_installed() {
            return Err(Error::Other(
                "Router not installed. Run 'mx router install' first.".into(),
            ));
        }

        self.ensure_network()?;
        let port = self.dashboard_port()?;

        let compose = Compose::new(self.install_dir())
            .with_file("docker-compose.yml")
            .with_project_name("mx-router")
            .with_env("MX_ROUTER_DASHBOARD_PORT", port.to_string());

        let output = compose.run(&["up", "-d"])?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!(
                "Failed to start router: {}",
                stderr
            )));
        }

        tracing::info!("Router started. Dashboard: http://localhost:{}", port);
        Ok(())
    }

    /// Stop the router
    pub fn stop(&self) -> Result<()> {
        if !self.is_installed() {
            return Ok(());
        }

        let compose = Compose::new(self.install_dir())
            .with_file("docker-compose.yml")
            .with_project_name("mx-router");

        let output = compose.run(&["down"])?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!(
                "Failed to stop router: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Get router status information
    pub fn status(&self) -> Result<RouterStatus> {
        Ok(RouterStatus {
            installed: self.is_installed(),
            running: self.is_running(),
            network: NETWORK_NAME.to_string(),
            dashboard_port: self.dashboard_port().ok(),
            install_dir: self.install_dir(),
        })
    }

    /// Get logs
    pub fn logs(&self, follow: bool) -> Result<std::process::Output> {
        if !self.is_installed() {
            return Err(Error::Other("Router not installed".into()));
        }

        let compose = Compose::new(self.install_dir())
            .with_file("docker-compose.yml")
            .with_project_name("mx-router");

        compose.logs(None, follow)
    }
}

/// Router status information
#[derive(Debug)]
pub struct RouterStatus {
    pub installed: bool,
    pub running: bool,
    pub network: String,
    pub dashboard_port: Option<u16>,
    pub install_dir: PathBuf,
}

impl Default for Router {
    fn default() -> Self {
        Self::new(MechCrateConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::StubBin;
    use std::sync::{Mutex, MutexGuard};

    /// Tests here mutate process-global env (`PATH` so the stub `docker` wins,
    /// `HOME` so config paths resolve into a tempdir). The suite runs under
    /// cargo-nextest, which is process-per-test, so each of these mutations is
    /// already isolated from every other test. The mutex below only matters if
    /// someone runs the same tests through plain `cargo test` (threads in one
    /// process) — it keeps them serialized rather than interleaved.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn env_guard() -> MutexGuard<'static, ()> {
        ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner())
    }

    /// A router rooted at `root` (i.e. `root/router` is the install dir),
    /// bypassing home resolution entirely.
    fn router_at(root: &Path) -> Router {
        Router::new(MechCrateConfig {
            root: root.to_path_buf(),
        })
    }

    /// A router whose install dir exists and contains `docker-compose.yml`,
    /// i.e. `is_installed() == true`.
    fn installed_router(root: &Path) -> Router {
        let router = router_at(root);
        std::fs::create_dir_all(router.install_dir()).unwrap();
        std::fs::write(
            router.install_dir().join("docker-compose.yml"),
            "services: {}\n",
        )
        .unwrap();
        router
    }

    // ── dashboard port allocation ──────────────────────────────────────────

    #[test]
    fn dashboard_port_allocates_within_the_documented_range() {
        let tmp = tempfile::tempdir().unwrap();
        let router = installed_router(tmp.path());

        let port = router.dashboard_port().unwrap();

        assert!(
            (DASHBOARD_PORT_START..=DASHBOARD_PORT_END).contains(&port),
            "allocated port {port} outside {DASHBOARD_PORT_START}-{DASHBOARD_PORT_END}"
        );
        let persisted = std::fs::read_to_string(router.install_dir().join(DASHBOARD_PORT_FILE))
            .expect("allocation must persist the port");
        assert_eq!(persisted.trim().parse::<u16>().unwrap(), port);
    }

    #[test]
    fn dashboard_port_reuses_a_cached_in_range_value() {
        let tmp = tempfile::tempdir().unwrap();
        let router = installed_router(tmp.path());
        std::fs::write(router.install_dir().join(DASHBOARD_PORT_FILE), "7777").unwrap();

        assert_eq!(router.dashboard_port().unwrap(), 7777);
    }

    #[test]
    fn dashboard_port_reallocates_when_the_cached_value_is_out_of_range() {
        let tmp = tempfile::tempdir().unwrap();
        let router = installed_router(tmp.path());
        let port_file = router.install_dir().join(DASHBOARD_PORT_FILE);
        std::fs::write(&port_file, "9999").unwrap();

        let port = router.dashboard_port().unwrap();

        assert_ne!(port, 9999, "stale out-of-range port must not be reused");
        assert!(
            (DASHBOARD_PORT_START..=DASHBOARD_PORT_END).contains(&port),
            "re-allocated port {port} outside {DASHBOARD_PORT_START}-{DASHBOARD_PORT_END}"
        );
        assert_eq!(
            std::fs::read_to_string(&port_file)
                .unwrap()
                .trim()
                .parse::<u16>()
                .unwrap(),
            port,
            "re-allocation must overwrite the stale cache"
        );
    }

    #[test]
    fn dashboard_port_reallocates_for_below_range_and_unparseable_caches() {
        let tmp = tempfile::tempdir().unwrap();
        let router = installed_router(tmp.path());
        let port_file = router.install_dir().join(DASHBOARD_PORT_FILE);

        for stale in ["80", "7679", "7800", "not-a-port", ""] {
            std::fs::write(&port_file, stale).unwrap();
            let port = router.dashboard_port().unwrap();
            assert!(
                (DASHBOARD_PORT_START..=DASHBOARD_PORT_END).contains(&port),
                "cache {stale:?} yielded out-of-range port {port}"
            );
        }
    }

    #[test]
    fn find_free_port_errors_when_the_range_is_unusable() {
        let tmp = tempfile::tempdir().unwrap();
        let router = router_at(tmp.path());

        // Port 0 is "any free port" to bind(), so a 0..=0 range always
        // succeeds; use a range we hold open instead.
        let held = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let taken = held.local_addr().unwrap().port();

        let err = router.find_free_port(taken, taken).unwrap_err();
        assert!(
            err.to_string().contains("No free port"),
            "unexpected error: {err}"
        );
    }

    // ── install detection (overridden home) ────────────────────────────────

    #[test]
    fn is_installed_keys_on_docker_compose_yml_under_the_overridden_home() {
        let home = tempfile::tempdir().unwrap();
        let _guard = env_guard();
        std::env::set_var("HOME", home.path());

        let config = MechCrateConfig::new().expect("home-derived config");
        assert_eq!(
            config.root,
            home.path().join(".mech-crate"),
            "HOME override must drive config root"
        );

        let router = Router::new(config);
        assert_eq!(router.install_dir(), home.path().join(".mech-crate/router"));
        assert!(!router.is_installed(), "empty home is not installed");

        std::fs::create_dir_all(router.install_dir()).unwrap();
        std::fs::write(router.install_dir().join("traefik.yml"), "x\n").unwrap();
        assert!(
            !router.is_installed(),
            "other files in the router dir must not count as installed"
        );

        std::fs::write(
            router.install_dir().join("docker-compose.yml"),
            "services: {}\n",
        )
        .unwrap();
        assert!(
            router.is_installed(),
            "docker-compose.yml is the install marker"
        );
    }

    // ── docker invocation contracts (stub-bin; no real daemon) ─────────────

    #[test]
    fn ensure_network_only_probes_when_the_network_already_exists() {
        let sb = StubBin::new();
        sb.stub("docker", 0, "[]");
        let _guard = env_guard();
        std::env::set_var("PATH", sb.path_env());

        let tmp = tempfile::tempdir().unwrap();
        router_at(tmp.path()).ensure_network().unwrap();

        let calls = sb.invocations("docker");
        assert_eq!(calls, vec![format!("network inspect {NETWORK_NAME}")]);
    }

    #[test]
    fn ensure_network_creates_the_network_when_the_probe_fails() {
        let sb = StubBin::new();
        // Non-zero: `network inspect` reports "missing", and the follow-up
        // `network create` reports failure — both invocations still recorded.
        sb.stub("docker", 1, "");
        let _guard = env_guard();
        std::env::set_var("PATH", sb.path_env());

        let tmp = tempfile::tempdir().unwrap();
        let err = router_at(tmp.path()).ensure_network().unwrap_err();
        assert!(matches!(err, Error::CommandFailed(_)), "got {err:?}");

        let calls = sb.invocations("docker");
        assert_eq!(
            calls,
            vec![
                format!("network inspect {NETWORK_NAME}"),
                format!("network create {NETWORK_NAME}"),
            ]
        );
    }

    #[test]
    fn start_ensures_the_network_then_composes_up_detached() {
        let sb = StubBin::new();
        sb.stub("docker", 0, "");
        let _guard = env_guard();
        std::env::set_var("PATH", sb.path_env());

        let tmp = tempfile::tempdir().unwrap();
        let router = installed_router(tmp.path());

        router.start().unwrap();

        let calls = sb.invocations("docker");
        assert_eq!(calls.len(), 2, "unexpected docker traffic: {calls:?}");
        assert_eq!(calls[0], format!("network inspect {NETWORK_NAME}"));
        assert_eq!(
            calls[1], "compose -f docker-compose.yml -p mx-router up -d",
            "compose invocation must pin file + project name and detach"
        );
    }

    #[test]
    fn stop_composes_down_against_the_same_project() {
        let sb = StubBin::new();
        sb.stub("docker", 0, "");
        let _guard = env_guard();
        std::env::set_var("PATH", sb.path_env());

        let tmp = tempfile::tempdir().unwrap();
        installed_router(tmp.path()).stop().unwrap();

        assert_eq!(
            sb.invocations("docker"),
            vec!["compose -f docker-compose.yml -p mx-router down".to_string()]
        );
    }

    #[test]
    fn start_without_an_install_errors_and_never_touches_docker() {
        let sb = StubBin::new();
        sb.stub("docker", 0, "");
        let _guard = env_guard();
        std::env::set_var("PATH", sb.path_env());

        let tmp = tempfile::tempdir().unwrap();
        let err = router_at(tmp.path()).start().unwrap_err();

        assert!(err.to_string().contains("Router not installed"), "{err}");
        assert!(
            sb.invocations("docker").is_empty(),
            "uninstalled start must not shell out"
        );
    }

    #[test]
    fn start_surfaces_a_failing_compose_up() {
        let sb = StubBin::new();
        sb.stub("docker", 7, "");
        let _guard = env_guard();
        std::env::set_var("PATH", sb.path_env());

        let tmp = tempfile::tempdir().unwrap();
        let err = installed_router(tmp.path()).start().unwrap_err();

        // ensure_network fails first when the network probe fails, so the
        // failure surfaces as a CommandFailed either way.
        assert!(matches!(err, Error::CommandFailed(_)), "got {err:?}");
    }

    #[test]
    fn status_reports_install_state_without_a_real_daemon() {
        let sb = StubBin::new();
        // `compose ps -q` prints nothing → not running.
        sb.stub("docker", 0, "");
        let _guard = env_guard();
        std::env::set_var("PATH", sb.path_env());

        let tmp = tempfile::tempdir().unwrap();
        let status = installed_router(tmp.path()).status().unwrap();

        assert!(status.installed);
        assert!(!status.running);
        assert_eq!(status.network, NETWORK_NAME);
        let port = status.dashboard_port.expect("port allocated");
        assert!((DASHBOARD_PORT_START..=DASHBOARD_PORT_END).contains(&port));
        assert!(status.install_dir.ends_with("router"));
    }
}
