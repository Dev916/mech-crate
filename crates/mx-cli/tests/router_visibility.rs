//! `mx router status` must not report a healthy router when the Traefik that
//! answers on the host ports cannot see this Docker daemon's containers
//! (bd:mech-crate-86o).
//!
//! The dual-daemon condition is reproduced without a second daemon: a stub
//! `docker` plays the active daemon (one running container declaring router
//! `shop-api`), and a local HTTP listener plays the Traefik dashboard API that
//! answers on the host. When that API has loaded only someone else's routers,
//! status must say so and fail; when it has loaded `shop-api`, status is ok.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

/// A fake `docker`: the router container is up, and one container on the
/// router network declares the router `shop-api`.
fn stub_docker(dir: &Path) {
    let script = r#"#!/bin/sh
case "$*" in
  *"compose"*"ps -q"*) echo routerid ;;
  "ps -q --filter network=devmesh-traefik") echo c1 ;;
  inspect*) echo '{"traefik.enable":"true","traefik.http.routers.shop-api.rule":"Host(`api.localhost`)","com.docker.compose.project":"shop"}' ;;
  *) exit 0 ;;
esac
"#;
    let path = dir.join("docker");
    std::fs::write(&path, script).expect("setup: write docker stub");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .expect("setup: chmod docker stub");
    }
}

/// Serve `body` as the Traefik `/api/http/routers` answer on a free port in the
/// dashboard range the router accepts, and return that port.
fn fake_dashboard(body: &'static str) -> u16 {
    let (listener, port) = (7700u16..7800)
        .find_map(|p| TcpListener::bind(("127.0.0.1", p)).ok().map(|l| (l, p)))
        .expect("setup: a free port in 7700-7799");
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            let mut buf = [0u8; 2048];
            let _ = stream.read(&mut buf);
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
        }
    });
    port
}

fn status_with(api_body: &'static str) -> assert_cmd::assert::Assert {
    let home = tempfile::tempdir().expect("setup: home");
    let router = home.path().join(".mech-crate/router");
    std::fs::create_dir_all(&router).expect("setup: router dir");
    std::fs::write(router.join("docker-compose.yml"), "services: {}\n").expect("setup: compose");
    let port = fake_dashboard(api_body);
    std::fs::write(router.join(".dashboard-port"), port.to_string()).expect("setup: port file");

    let bin = home.path().join("bin");
    std::fs::create_dir_all(&bin).expect("setup: bin");
    stub_docker(&bin);
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    Command::cargo_bin("mx")
        .expect("mx binary built")
        .env("HOME", home.path())
        .env_remove("MX_HOME")
        .env("PATH", path)
        .args(["router", "status"])
        .assert()
}

#[test]
fn status_fails_when_the_answering_traefik_cannot_see_this_daemon() {
    status_with(r#"[{"name":"other-api@docker"},{"name":"dashboard@internal"}]"#)
        .failure()
        .stdout(contains("Status: running"))
        .stdout(contains("cannot see this Docker daemon"))
        .stdout(contains("shop-api"))
        .stdout(contains("Routing: ok").not());
}

#[test]
fn status_is_ok_when_the_answering_traefik_has_loaded_this_daemons_routers() {
    status_with(r#"[{"name":"shop-api@docker"},{"name":"other-api@docker"}]"#)
        .success()
        .stdout(contains("Routing: ok (1 router(s)"));
}
