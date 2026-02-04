//! Integration tests for the mx CLI

use std::process::Command;

/// Get the mx binary path
fn mx_binary() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove deps
    path.push("mx");
    path
}

#[test]
fn test_mx_help() {
    let output = Command::new(mx_binary())
        .arg("--help")
        .output()
        .expect("Failed to execute mx --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MechCrate CLI"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("new"));
    assert!(stdout.contains("recipes"));
}

#[test]
fn test_mx_version() {
    let output = Command::new(mx_binary())
        .arg("--version")
        .output()
        .expect("Failed to execute mx --version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("mx"));
}

#[test]
fn test_mx_recipes_help() {
    let output = Command::new(mx_binary())
        .args(["recipes", "--help"])
        .output()
        .expect("Failed to execute mx recipes --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"));
    assert!(stdout.contains("info"));
}

#[test]
fn test_mx_router_help() {
    let output = Command::new(mx_binary())
        .args(["router", "--help"])
        .output()
        .expect("Failed to execute mx router --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("install"));
    assert!(stdout.contains("up"));
    assert!(stdout.contains("down"));
}

#[test]
fn test_mx_mcp_help() {
    let output = Command::new(mx_binary())
        .args(["mcp", "--help"])
        .output()
        .expect("Failed to execute mx mcp --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("build"));
    assert!(stdout.contains("start"));
    assert!(stdout.contains("status"));
}

#[test]
fn test_mx_infra_help() {
    let output = Command::new(mx_binary())
        .args(["infra", "--help"])
        .output()
        .expect("Failed to execute mx infra --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("setup"));
    assert!(stdout.contains("list"));
}

#[test]
fn test_mx_upgrade_help() {
    let output = Command::new(mx_binary())
        .args(["upgrade", "--help"])
        .output()
        .expect("Failed to execute mx upgrade --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--diff"));
    assert!(stdout.contains("--yes"));
    assert!(stdout.contains("--dry-run"));
}

#[test]
fn test_mx_init_help() {
    let output = Command::new(mx_binary())
        .args(["init", "--help"])
        .output()
        .expect("Failed to execute mx init --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--force"));
    assert!(stdout.contains("--update"));
}

#[test]
fn test_mx_doctor_runs() {
    let output = Command::new(mx_binary())
        .arg("doctor")
        .output()
        .expect("Failed to execute mx doctor");

    // Doctor should run (may fail if not initialized, but should run)
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MechCrate Health Check") || stdout.contains("not initialized"));
}
