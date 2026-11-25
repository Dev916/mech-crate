//! Does the Traefik answering on the host see this Docker daemon's containers?
//!
//! `mx router status` used to report "running" from `docker compose ps` alone.
//! That is true of the router container on the active Docker context and says
//! nothing about which process owns host port 80. With two daemons on one
//! workstation (Docker Desktop and OrbStack), each can run a container named
//! `mx-router`; one of them holds the host ports, and if it is the other
//! daemon's, every `<svc>.localhost` request reaches a Traefik whose docker
//! provider cannot see the stacks started here and answers 404
//! (bd:mech-crate-86o).
//!
//! The check compares two views. From the active daemon: the router names its
//! running, traefik-enabled containers declare. From the Traefik that answers
//! on the host dashboard port: the routers its docker provider has loaded. A
//! router this daemon declares and that Traefik has never heard of is the
//! signature of a router that belongs to another daemon.
//!
//! Everything here is pure; the CLI gathers the two inputs.

use std::collections::BTreeSet;

/// The verdict of comparing the two views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouterView {
    /// Every router this daemon declares is loaded by the answering Traefik.
    Consistent { routers: usize },
    /// Nothing on this daemon declares a router, so there is nothing to compare.
    Unverified,
    /// The answering Traefik has loaded none of the routers this daemon
    /// declares: it is almost certainly another daemon's router.
    Blind { missing: Vec<String> },
    /// Some declared routers are missing. Usually a container still starting or
    /// a label Traefik rejected; not the dual-daemon signature.
    Partial { missing: Vec<String>, seen: usize },
    /// The dashboard API on the host port did not answer.
    ApiUnreachable { detail: String },
}

impl RouterView {
    /// Whether `mx router status` must fail: the host ports are answered by a
    /// Traefik that cannot route this daemon's containers, or by nothing.
    /// `Partial` is a warning, since a container may simply still be starting.
    pub fn is_fault(&self) -> bool {
        matches!(
            self,
            RouterView::Blind { .. } | RouterView::ApiUnreachable { .. }
        )
    }

    /// Whether `mx router status` should present the router as healthy.
    pub fn is_healthy(&self) -> bool {
        matches!(self, RouterView::Consistent { .. } | RouterView::Unverified)
    }
}

/// Router names declared by one container's labels: every key shaped
/// `traefik.http.routers.<name>.rule`, on containers with `traefik.enable=true`.
pub fn declared_routers<'a, I>(labels: I) -> Vec<String>
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let labels: Vec<(&str, &str)> = labels.into_iter().collect();
    let enabled = labels
        .iter()
        .any(|(k, v)| *k == "traefik.enable" && v.trim() == "true");
    if !enabled {
        return Vec::new();
    }
    let mut names: BTreeSet<String> = BTreeSet::new();
    for (key, _) in &labels {
        if let Some(rest) = key.strip_prefix("traefik.http.routers.") {
            if let Some(name) = rest.strip_suffix(".rule") {
                if !name.is_empty() {
                    names.insert(name.to_string());
                }
            }
        }
    }
    names.into_iter().collect()
}

/// Names of the routers the answering Traefik loaded from its docker provider.
/// The API reports them as `<name>@docker`; file and internal routers are not
/// this daemon's containers and are ignored.
pub fn docker_provider_routers<'a, I>(api_names: I) -> Vec<String>
where
    I: IntoIterator<Item = &'a str>,
{
    api_names
        .into_iter()
        .filter_map(|n| n.strip_suffix("@docker"))
        .map(str::to_string)
        .collect()
}

/// Compare what this daemon declares with what the answering Traefik loaded.
pub fn diagnose(declared: &[String], api: Result<Vec<String>, String>) -> RouterView {
    let loaded = match api {
        Ok(names) => names.into_iter().collect::<BTreeSet<String>>(),
        Err(detail) => return RouterView::ApiUnreachable { detail },
    };
    let declared: BTreeSet<&String> = declared.iter().collect();
    if declared.is_empty() {
        return RouterView::Unverified;
    }
    let missing: Vec<String> = declared
        .iter()
        .filter(|n| !loaded.contains(n.as_str()))
        .map(|n| n.to_string())
        .collect();
    let seen = declared.len() - missing.len();
    if missing.is_empty() {
        RouterView::Consistent {
            routers: declared.len(),
        }
    } else if seen == 0 {
        RouterView::Blind { missing }
    } else {
        RouterView::Partial { missing, seen }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_router_that_sees_none_of_this_daemons_containers_is_blind() {
        // The 86o reproduction: OrbStack runs `shop-api`, the Traefik on :80
        // belongs to Docker Desktop and has loaded only its own routers.
        let view = diagnose(
            &names(&["shop-api", "shop-site"]),
            Ok(names(&["other-api", "legacy-web"])),
        );
        assert_eq!(
            view,
            RouterView::Blind {
                missing: names(&["shop-api", "shop-site"])
            }
        );
        assert!(!view.is_healthy());
        assert!(view.is_fault());
    }

    #[test]
    fn a_router_that_has_loaded_every_declared_router_is_consistent() {
        let view = diagnose(
            &names(&["shop-api"]),
            Ok(names(&["shop-api", "someone-elses"])),
        );
        assert_eq!(view, RouterView::Consistent { routers: 1 });
        assert!(view.is_healthy());
    }

    #[test]
    fn some_missing_is_partial_not_the_dual_daemon_verdict() {
        let view = diagnose(&names(&["shop-api", "shop-site"]), Ok(names(&["shop-api"])));
        assert_eq!(
            view,
            RouterView::Partial {
                missing: names(&["shop-site"]),
                seen: 1
            }
        );
        assert!(!view.is_healthy());
        assert!(
            !view.is_fault(),
            "a starting container must not fail the command"
        );
    }

    #[test]
    fn nothing_declared_cannot_be_verified_and_is_not_an_alarm() {
        let view = diagnose(&[], Ok(names(&["anything"])));
        assert_eq!(view, RouterView::Unverified);
        assert!(view.is_healthy());
    }

    #[test]
    fn an_unanswering_dashboard_is_reported_as_such() {
        let view = diagnose(&names(&["shop-api"]), Err("connection refused".into()));
        assert_eq!(
            view,
            RouterView::ApiUnreachable {
                detail: "connection refused".into()
            }
        );
        assert!(!view.is_healthy());
    }

    #[test]
    fn declared_routers_reads_only_enabled_containers_rule_keys() {
        let enabled = [
            ("traefik.enable", "true"),
            (
                "traefik.http.routers.shop-api.rule",
                "Host(`api.localhost`)",
            ),
            ("traefik.http.routers.shop-api.entrypoints", "web"),
            (
                "traefik.http.services.shop-api.loadbalancer.server.port",
                "80",
            ),
            ("com.docker.compose.project", "shop"),
        ];
        assert_eq!(declared_routers(enabled), names(&["shop-api"]));

        let disabled = [("traefik.http.routers.shop-api.rule", "Host(`x`)")];
        assert!(declared_routers(disabled).is_empty());
    }

    #[test]
    fn only_docker_provider_routers_count_as_loaded() {
        let api = ["shop-api@docker", "dashboard@internal", "eco7@file"];
        assert_eq!(docker_provider_routers(api), names(&["shop-api"]));
    }
}
