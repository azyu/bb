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
    assert!(stdout.contains("Quick start:"));
    assert!(stdout.contains("bb pr create --workspace acme --repo widgets"));
    assert!(stdout.contains("bb pr comments 123 --workspace acme --repo widgets"));
    assert!(stdout.contains("Add --output json"));
}

#[test]
fn root_help_flag_matches_no_arg_help() {
    let no_arg = bb_command().output().expect("command should run");
    assert!(no_arg.status.success());

    let help_flag = bb_command()
        .arg("--help")
        .output()
        .expect("command should run");
    assert!(help_flag.status.success());

    let no_arg_stdout = String::from_utf8(no_arg.stdout).expect("stdout should be utf-8");
    let help_flag_stdout = String::from_utf8(help_flag.stdout).expect("stdout should be utf-8");
    assert_eq!(help_flag_stdout, no_arg_stdout);
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
fn repo_list_json_fields_projects_requested_keys() {
    let server = MockServer::start();
    let repos = server.mock(|when, then| {
        when.method(GET).path("/2.0/repositories/acme");
        then.json_body(json!({
            "values": [
                {
                    "slug": "one",
                    "full_name": "acme/one",
                    "name": "one",
                    "uuid": "{repo-1}"
                }
            ]
        }));
    });

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "repo",
            "list",
            "--workspace",
            "acme",
            "--output",
            "json",
            "--json-fields",
            "slug,full_name",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body[0]["slug"], "one");
    assert_eq!(body[0]["full_name"], "acme/one");
    assert!(body[0].get("name").is_none());
    repos.assert();
}

#[test]
fn pipeline_help_lists_debugging_commands() {
    let output = bb_command()
        .args(["pipeline", "--help"])
        .output()
        .expect("command should run");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("get"));
    assert!(stdout.contains("steps"));
    assert!(stdout.contains("log"));
}

#[test]
fn pipeline_get_json_reads_config_and_calls_server() {
    let server = MockServer::start();
    let pipeline = server.mock(|when, then| {
        when.method(GET)
            .path("/2.0/repositories/acme/widgets/pipelines/%7Bpipe-123%7D");
        then.json_body(json!({
            "uuid": "{pipe-123}",
            "build_number": 17,
            "state": { "name": "COMPLETED", "result": { "name": "FAILED" } },
            "target": { "ref_name": "feature/widgets" }
        }));
    });

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "pipeline",
            "get",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--uuid",
            "{pipe-123}",
            "--output",
            "json",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body["uuid"], "{pipe-123}");
    assert_eq!(body["build_number"], 17);
    pipeline.assert();
}

#[test]
fn pipeline_get_json_fields_projects_requested_keys() {
    let server = MockServer::start();
    let pipeline = server.mock(|when, then| {
        when.method(GET)
            .path("/2.0/repositories/acme/widgets/pipelines/%7Bpipe-123%7D");
        then.json_body(json!({
            "uuid": "{pipe-123}",
            "build_number": 17,
            "state": { "name": "COMPLETED" },
            "target": { "ref_name": "feature/widgets" }
        }));
    });

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "pipeline",
            "get",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--uuid",
            "{pipe-123}",
            "--output",
            "json",
            "--json-fields",
            "uuid,state",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body["uuid"], "{pipe-123}");
    assert_eq!(body["state"]["name"], "COMPLETED");
    assert!(body.get("build_number").is_none());
    pipeline.assert();
}

#[test]
fn pipeline_steps_json_reads_config_and_calls_server() {
    let server = MockServer::start();
    let steps = server.mock(|when, then| {
        when.method(GET)
            .path("/2.0/repositories/acme/widgets/pipelines/%7Bpipe-123%7D/steps");
        then.json_body(json!({
            "values": [
                {
                    "uuid": "{step-1}",
                    "name": "build",
                    "state": { "name": "COMPLETED", "result": { "name": "FAILED" } }
                }
            ]
        }));
    });

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "pipeline",
            "steps",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--uuid",
            "{pipe-123}",
            "--output",
            "json",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body[0]["uuid"], "{step-1}");
    assert_eq!(body[0]["name"], "build");
    steps.assert();
}

#[test]
fn pipeline_log_text_reads_config_and_calls_server() {
    let server = MockServer::start();
    let log = server.mock(|when, then| {
        when.method(GET)
            .path("/2.0/repositories/acme/widgets/pipelines/%7Bpipe-123%7D/steps/%7Bstep-1%7D/log");
        then.body("build failed\n");
    });

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "pipeline",
            "log",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--uuid",
            "{pipe-123}",
            "--step",
            "{step-1}",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(stdout, "build failed\n");
    log.assert();
}

#[test]
fn pipeline_get_rejects_unbalanced_uuid_braces() {
    let output = bb_command()
        .args([
            "pipeline",
            "get",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--uuid",
            "{pipe-123",
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
    assert_eq!(body["error"]["message"], "--uuid must be a Bitbucket UUID");
}

#[test]
fn pipeline_log_rejects_unbalanced_step_braces() {
    let output = bb_command()
        .args([
            "pipeline",
            "log",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--uuid",
            "{pipe-123}",
            "--step",
            "step-1}",
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
    assert_eq!(body["error"]["message"], "--step must be a Bitbucket UUID");
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
    assert!(stdout.contains("steps"));
    assert!(stdout.contains("log"));
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
fn pr_comments_help_includes_positional_id() {
    let output = bb_command()
        .args(["pr", "comments", "--help"])
        .output()
        .expect("command should run");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Usage: bb pr comments"));
    assert!(stdout.contains("[ID]"));
    assert!(stdout.contains("--id <ID>"));
    assert!(stdout.contains("--comment-id <COMMENT_ID>"));
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
            "42",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
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
fn pr_get_json_fields_projects_requested_keys() {
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
            "42",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--output",
            "json",
            "--json-fields",
            "id,title,state",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body["id"], 42);
    assert_eq!(body["title"], "Add widget support");
    assert_eq!(body["state"], "OPEN");
    assert!(body.get("source").is_none());
    pr.assert();
}

#[test]
fn pr_comments_positional_id_reads_config_and_calls_server() {
    let server = MockServer::start();
    let comments = server.mock(|when, then| {
        when.method(GET)
            .path("/2.0/repositories/acme/widgets/pullrequests/42/comments");
        then.json_body(json!({
            "values": [
                {
                    "id": 7,
                    "content": { "raw": "needs changes" }
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
            "comments",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
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
    assert_eq!(body[0]["id"], 7);
    assert_eq!(body[0]["content"]["raw"], "needs changes");
    comments.assert();
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
fn pr_comment_get_json_reads_config_and_calls_server() {
    let server = MockServer::start();
    let comment = server.mock(|when, then| {
        when.method(GET)
            .path("/2.0/repositories/acme/widgets/pullrequests/42/comments/7");
        then.json_body(json!({
            "id": 7,
            "content": {
                "raw": "needs changes"
            },
            "user": {
                "display_name": "codex"
            }
        }));
    });

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "pr",
            "comments",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "42",
            "--comment-id",
            "7",
            "--output",
            "json",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body["id"], 7);
    assert_eq!(body["content"]["raw"], "needs changes");
    comment.assert();
}

#[test]
fn pr_comment_get_rejects_list_flags() {
    let output = bb_command()
        .args([
            "pr",
            "comments",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "42",
            "--comment-id",
            "7",
            "--all",
        ])
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("--comment-id cannot be combined with --all, --q, or --sort"));
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
fn pr_statuses_json_fields_include_timestamps() {
    let server = MockServer::start();
    let statuses = server.mock(|when, then| {
        when.method(GET)
            .path("/2.0/repositories/acme/widgets/pullrequests/42/statuses");
        then.json_body(json!({
            "values": [
                {
                    "key": "build",
                    "state": "SUCCESSFUL",
                    "created_on": "2026-03-10T00:00:00Z",
                    "updated_on": "2026-03-10T01:00:00Z"
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
            "--json-fields",
            "key,created_on,updated_on",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body[0]["key"], "build");
    assert_eq!(body[0]["created_on"], "2026-03-10T00:00:00Z");
    assert_eq!(body[0]["updated_on"], "2026-03-10T01:00:00Z");
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

#[test]
fn pr_comments_invalid_positional_id_emits_json_error() {
    let output = bb_command()
        .args([
            "pr",
            "comments",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "abc",
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
    assert_eq!(
        body["error"]["message"],
        "pull request id must be a number: abc"
    );
}

#[test]
fn pr_get_json_fields_requires_json_output() {
    let output = bb_command()
        .args([
            "pr",
            "get",
            "42",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--json-fields",
            "id,title",
        ])
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("--json-fields requires --output json"));
}

#[test]
fn pr_get_invalid_json_field_emits_json_error() {
    let output = bb_command()
        .args([
            "pr",
            "get",
            "42",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--output",
            "json",
            "--json-fields",
            "nope",
        ])
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body["error"]["code"], "invalid_input");
    assert_eq!(
        body["error"]["message"],
        "unknown --json-fields value for bb pr get: nope (allowed: author, close_source_branch, comment_count, created_on, description, destination, draft, id, links, participants, reason, reviewers, source, state, summary, task_count, title, updated_on)"
    );
}
