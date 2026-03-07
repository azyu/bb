use std::fs;
use std::process::Command;

use httpmock::Method::{DELETE, GET};
use httpmock::MockServer;
use serde_json::json;
use tempfile::tempdir;

fn bb_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_bb"))
}

fn write_config(config_path: &std::path::Path, base_url: &str) {
    fs::write(
        config_path,
        format!(
            "{{\n  \"current\": \"default\",\n  \"profiles\": {{\n    \"default\": {{\n      \"base_url\": \"{base_url}\",\n      \"token\": \"token-123\",\n      \"username\": \"\"\n    }}\n  }}\n}}\n"
        ),
    )
    .unwrap();
}

#[test]
fn root_help_prints_commands() {
    let output = bb_command().output().expect("command should run");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("bb - Bitbucket CLI"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("auth"));
    assert!(stdout.contains("completion"));
}

#[test]
fn version_prints_metadata() {
    let output = bb_command()
        .arg("version")
        .output()
        .expect("command should run");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains(&format!("bb version {}", env!("CARGO_PKG_VERSION"))));
    assert!(stdout.contains("commit:"));
    assert!(stdout.contains("built:"));
}

#[test]
fn auth_status_without_login_writes_error_to_stderr() {
    let temp = tempdir().unwrap();
    let output = bb_command()
        .args(["auth", "status"])
        .env("BB_CONFIG_PATH", temp.path().join("config.json"))
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("not logged in"));
}

#[test]
fn repo_list_json_reads_config_and_calls_server() {
    let server = MockServer::start();
    let repos = server.mock(|when, then| {
        when.method(GET).path("/2.0/repositories/acme");
        then.json_body(json!({
            "values": [
                {"slug": "one", "full_name": "acme/one"}
            ]
        }));
    });

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args(["repo", "list", "--workspace", "acme", "--output", "json"])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body[0]["full_name"], "acme/one");
    repos.assert();
}

#[test]
fn completion_bash_prints_script() {
    let output = bb_command()
        .args(["completion", "bash"])
        .output()
        .expect("command should run");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("complete -F _bb_complete bb"));
    assert!(stdout.contains("request-changes"));
    assert!(stdout.contains("remove-request-changes"));
}

#[test]
fn pr_help_lists_api_aligned_commands() {
    let output = bb_command()
        .args(["pr", "--help"])
        .output()
        .expect("command should run");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("get"));
    assert!(stdout.contains("update"));
    assert!(stdout.contains("request-changes"));
    assert!(stdout.contains("remove-request-changes"));
    assert!(stdout.contains("statuses"));
    assert!(stdout.contains("activity"));
}

#[test]
fn pr_get_json_reads_config_and_calls_server() {
    let server = MockServer::start();
    let pr = server.mock(|when, then| {
        when.method(GET)
            .path("/2.0/repositories/acme/widgets/pullrequests/42");
        then.json_body(json!({
            "id": 42,
            "state": "OPEN",
            "title": "Add widget support",
            "source": { "branch": { "name": "feature/widgets" } },
            "destination": { "branch": { "name": "main" } }
        }));
    });

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "pr",
            "get",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--id",
            "42",
            "--output",
            "json",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body["title"], "Add widget support");
    pr.assert();
}

#[test]
fn pr_diff_text_reads_config_and_calls_server() {
    let server = MockServer::start();
    let diff = server.mock(|when, then| {
        when.method(GET)
            .path("/2.0/repositories/acme/widgets/pullrequests/42/diff");
        then.body("diff --git a/src/lib.rs b/src/lib.rs\n");
    });

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "pr",
            "diff",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--id",
            "42",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(stdout, "diff --git a/src/lib.rs b/src/lib.rs\n");
    diff.assert();
}

#[test]
fn pr_unapprove_json_emits_synthetic_envelope() {
    let server = MockServer::start();
    let unapprove = server.mock(|when, then| {
        when.method(DELETE)
            .path("/2.0/repositories/acme/widgets/pullrequests/42/approve");
        then.status(204);
    });

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "pr",
            "unapprove",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--id",
            "42",
            "--output",
            "json",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body["id"], 42);
    assert_eq!(body["action"], "Removed approval from");
    assert_eq!(body["ok"], true);
    unapprove.assert();
}

#[test]
fn pr_statuses_json_reads_config_and_calls_server() {
    let server = MockServer::start();
    let statuses = server.mock(|when, then| {
        when.method(GET)
            .path("/2.0/repositories/acme/widgets/pullrequests/42/statuses");
        then.json_body(json!({
            "values": [
                {
                    "key": "build",
                    "state": "SUCCESSFUL",
                    "name": "CI"
                }
            ]
        }));
    });

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "pr",
            "statuses",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--id",
            "42",
            "--output",
            "json",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body[0]["key"], "build");
    assert_eq!(body[0]["state"], "SUCCESSFUL");
    statuses.assert();
}

#[test]
fn pr_comment_missing_content_emits_json_error() {
    let output = bb_command()
        .args([
            "pr",
            "comment",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--id",
            "42",
            "--output",
            "json",
        ])
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body["error"]["code"], "invalid_input");
    assert_eq!(body["error"]["message"], "--content is required");
}
