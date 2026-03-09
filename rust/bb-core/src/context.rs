use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

use crate::config::Profile;
use crate::error::CliError;

pub fn resolve_repo_target(
    workspace_value: Option<&str>,
    repo_value: Option<&str>,
    require_repo: bool,
) -> Result<(String, String), CliError> {
    let mut workspace = workspace_value.unwrap_or_default().trim().to_string();
    let mut repo = repo_value.unwrap_or_default().trim().to_string();

    if (workspace.is_empty() || (require_repo && repo.is_empty()))
        && let Ok((inferred_workspace, inferred_repo)) = infer_bitbucket_repo_from_git(None)
    {
        if workspace.is_empty() {
            workspace = inferred_workspace;
        }
        if repo.is_empty() {
            repo = inferred_repo;
        }
    }

    if workspace.is_empty() {
        return Err(CliError::InvalidInput(
            "--workspace is required".to_string(),
        ));
    }
    if require_repo && repo.is_empty() {
        return Err(CliError::InvalidInput("--repo is required".to_string()));
    }

    Ok((workspace, repo))
}

pub fn infer_bitbucket_repo_from_git(dir: Option<&Path>) -> Result<(String, String), CliError> {
    let remote = run_git(dir, ["config", "--get", "remote.origin.url"])?;
    let remote = remote.trim();
    if remote.is_empty() {
        return Err(CliError::Git("remote.origin.url not set".to_string()));
    }

    parse_bitbucket_remote(remote)
        .ok_or_else(|| CliError::Git("origin remote is not a Bitbucket repository".to_string()))
}

pub fn parse_bitbucket_remote(remote: &str) -> Option<(String, String)> {
    let trimmed = remote.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.contains("://") {
        let url = reqwest::Url::parse(trimmed).ok()?;
        if url.host_str()?.eq_ignore_ascii_case("bitbucket.org") {
            return parse_bitbucket_path(url.path());
        }
        return None;
    }

    let (mut host, path) = trimmed.split_once(':')?;
    if let Some((_, rest)) = host.rsplit_once('@') {
        host = rest;
    }
    if !host.trim().eq_ignore_ascii_case("bitbucket.org") {
        return None;
    }
    parse_bitbucket_path(path)
}

pub fn parse_bitbucket_path(raw_path: &str) -> Option<(String, String)> {
    let trimmed = raw_path
        .trim()
        .trim_start_matches('/')
        .trim_end_matches('/');
    let mut parts = trimmed.split('/');
    let workspace = parts.next()?.trim();
    let repo = parts.next()?.trim().trim_end_matches(".git");
    if parts.next().is_some() || workspace.is_empty() || repo.is_empty() {
        return None;
    }
    Some((workspace.to_string(), repo.to_string()))
}

pub fn run_git<I, S>(dir: Option<&Path>, args: I) -> Result<String, CliError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    run_git_inner(dir, None, args)
}

pub fn run_git_with_env<I, S>(
    dir: Option<&Path>,
    envs: &[(&str, &str)],
    args: I,
) -> Result<String, CliError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    run_git_inner(dir, Some(envs), args)
}

fn run_git_inner<I, S>(
    dir: Option<&Path>,
    envs: Option<&[(&str, &str)]>,
    args: I,
) -> Result<String, CliError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new("git");
    command.arg("-c").arg("credential.helper=");
    command.args(args);
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    if let Some(envs) = envs {
        for (key, value) in envs {
            command.env(key, value);
        }
    }

    let output = command
        .output()
        .map_err(|error| CliError::Git(error.to_string()))?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let message = if stderr.is_empty() { stdout } else { stderr };
    Err(CliError::Git(message))
}

pub fn normalize_wiki_page_path(page: &str) -> Result<String, CliError> {
    let trimmed = page.trim();
    if trimmed.is_empty() {
        return Err(CliError::InvalidInput("--page is required".to_string()));
    }

    let clean = Path::new(trimmed);
    if clean.is_absolute() {
        return Err(CliError::InvalidInput("invalid --page value".to_string()));
    }

    let normalized = clean.components().collect::<PathBuf>();
    let normalized = normalized.to_string_lossy().replace('\\', "/");
    if normalized == "." || normalized == ".." || normalized.starts_with("../") {
        return Err(CliError::InvalidInput("invalid --page value".to_string()));
    }

    Ok(normalized)
}

pub fn resolve_wiki_auth_user(profile_username: &str) -> String {
    let username = profile_username.trim();
    if username.is_empty() {
        return "x-token-auth".to_string();
    }
    if username.contains('@') {
        return "x-bitbucket-api-token-auth".to_string();
    }
    username.to_string()
}

pub fn build_wiki_remote_url(
    profile: &Profile,
    workspace: &str,
    repo: &str,
) -> Result<String, CliError> {
    if profile.token.trim().is_empty() {
        return Err(CliError::Config(
            "profile has no token configured".to_string(),
        ));
    }

    let mut host = "bitbucket.org".to_string();
    if let Ok(url) = reqwest::Url::parse(profile.base_url.trim())
        && let Some(parsed_host) = url.host_str()
    {
        if parsed_host.eq_ignore_ascii_case("api.bitbucket.org") {
            host = "bitbucket.org".to_string();
        } else {
            host = parsed_host.to_string();
        }
    }

    let user = resolve_wiki_auth_user(&profile.username);
    Ok(format!("https://{user}@{host}/{workspace}/{repo}.git/wiki"))
}

pub fn clone_wiki_to_temp(
    profile: &Profile,
    workspace: &str,
    repo: &str,
) -> Result<TempDir, CliError> {
    let remote = build_wiki_remote_url(profile, workspace, repo)?;
    let tempdir = tempfile::tempdir().map_err(|error| CliError::Io(error.to_string()))?;
    let args = vec![
        OsString::from("clone"),
        OsString::from("--depth"),
        OsString::from("1"),
        OsString::from(&remote),
        tempdir.path().as_os_str().to_os_string(),
    ];
    run_git_with_askpass(None, &profile.token, args)?;
    Ok(tempdir)
}

pub fn run_git_with_askpass<I, S>(
    dir: Option<&Path>,
    token: &str,
    args: I,
) -> Result<String, CliError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut script = tempfile::NamedTempFile::new()
        .map_err(|error| CliError::Io(format!("create askpass script: {error}")))?;
    script
        .write_all(
            format!(
                "#!/bin/sh\nprintf '%s\\n' '{}'\n",
                shell_escape_single_quote(token)
            )
            .as_bytes(),
        )
        .map_err(|error| CliError::Io(format!("write askpass script: {error}")))?;
    let mut permissions = script
        .as_file()
        .metadata()
        .map_err(|error| CliError::Io(error.to_string()))?
        .permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o700);
    }
    fs::set_permissions(script.path(), permissions)
        .map_err(|error| CliError::Io(error.to_string()))?;
    let path = script.path().to_string_lossy().to_string();
    let envs = [("GIT_ASKPASS", path.as_str()), ("GIT_TERMINAL_PROMPT", "0")];
    let result = run_git_with_env(dir, &envs, args);
    drop(script);
    result
}

pub fn ensure_git_worktree(dir: Option<&Path>) -> Result<(), CliError> {
    let output = run_git(dir, ["rev-parse", "--is-inside-work-tree"])?;
    if output.trim() == "true" {
        Ok(())
    } else {
        Err(CliError::Git(
            "current directory is not a git worktree".to_string(),
        ))
    }
}

pub fn validate_git_branch_name(dir: Option<&Path>, branch: &str) -> Result<(), CliError> {
    let trimmed = branch.trim();
    if trimmed.is_empty() {
        return Err(CliError::InvalidInput("--branch is required".to_string()));
    }
    run_git(dir, ["check-ref-format", "--branch", trimmed]).map(|_| ())
}

pub fn resolve_git_revision(
    dir: Option<&Path>,
    revision: &str,
) -> Result<Option<String>, CliError> {
    let mut command = Command::new("git");
    command.arg("-c").arg("credential.helper=");
    command.args(["rev-parse", "--verify", "--quiet", revision]);
    if let Some(dir) = dir {
        command.current_dir(dir);
    }

    let output = command
        .output()
        .map_err(|error| CliError::Git(error.to_string()))?;
    if !output.status.success() {
        return Ok(None);
    }

    let revision = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if revision.is_empty() {
        Ok(None)
    } else {
        Ok(Some(revision))
    }
}

fn shell_escape_single_quote(value: &str) -> String {
    value.replace('\'', "'\\''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_remote_supports_https_and_ssh() {
        assert_eq!(
            parse_bitbucket_remote("https://bitbucket.org/acme/app.git"),
            Some(("acme".to_string(), "app".to_string()))
        );
        assert_eq!(
            parse_bitbucket_remote("git@bitbucket.org:acme/app.git"),
            Some(("acme".to_string(), "app".to_string()))
        );
        assert_eq!(
            parse_bitbucket_remote("https://github.com/acme/app.git"),
            None
        );
    }

    #[test]
    fn wiki_auth_user_follows_profile_type() {
        assert_eq!(
            resolve_wiki_auth_user("dev@example.com"),
            "x-bitbucket-api-token-auth"
        );
        assert_eq!(resolve_wiki_auth_user(""), "x-token-auth");
        assert_eq!(resolve_wiki_auth_user("workspace-bot"), "workspace-bot");
    }
}
