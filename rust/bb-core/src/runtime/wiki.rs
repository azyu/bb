use std::fs;
use std::io::Write;
use std::path::Path;

use crate::config::Profile;
use crate::context;
use crate::error::CliError;
use crate::render::{self, WikiPage};
use crate::{
    ListOutput, WikiGetRequest, WikiListRequest, WikiPutRequest, WikiRequest, WriteOutput,
};
use serde_json::json;

use super::support::{
    optional_trimmed, parse_list_output, parse_write_output, profile_from_config,
};

pub(super) fn handle_wiki<O: Write>(request: &WikiRequest, stdout: &mut O) -> Result<(), CliError> {
    match request {
        WikiRequest::Help => write!(stdout, "{}", render::wiki_usage()).map_err(CliError::from),
        WikiRequest::List(request) => handle_wiki_list(request, stdout),
        WikiRequest::Get(request) => handle_wiki_get(request, stdout),
        WikiRequest::Put(request) => handle_wiki_put(request, stdout),
    }
}

fn handle_wiki_list<O: Write>(request: &WikiListRequest, stdout: &mut O) -> Result<(), CliError> {
    let output = parse_list_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let profile = profile_from_config(request.profile.as_deref())?;
    let repo_dir = context::clone_wiki_to_temp(&profile, &workspace, &repo)
        .map_err(|error| CliError::Git(redact_token(&error.message(), &profile.token)))?;
    let rows = list_wiki_pages(repo_dir.path())?;

    match output {
        ListOutput::Json => render::print_json(stdout, &rows),
        ListOutput::Table => {
            write!(stdout, "{}", render::render_wiki_table(&rows)).map_err(CliError::from)
        }
    }
}

fn handle_wiki_get<O: Write>(request: &WikiGetRequest, stdout: &mut O) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let page = context::normalize_wiki_page_path(
        request
            .page
            .as_deref()
            .ok_or_else(|| CliError::InvalidInput("--page is required".to_string()))?,
    )?;
    let profile = profile_from_config(request.profile.as_deref())?;
    let repo_dir = context::clone_wiki_to_temp(&profile, &workspace, &repo)
        .map_err(|error| CliError::Git(redact_token(&error.message(), &profile.token)))?;
    let abs_path = repo_dir.path().join(Path::new(&page));
    let content = fs::read_to_string(&abs_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            CliError::Io(format!("wiki page not found: {page}"))
        } else {
            CliError::Io(format!("read wiki page: {error}"))
        }
    })?;

    match output {
        WriteOutput::Text => write!(stdout, "{content}").map_err(CliError::from),
        WriteOutput::Json => {
            render::print_json(stdout, &json!({ "page": page, "content": content }))
        }
    }
}

fn handle_wiki_put<O: Write>(request: &WikiPutRequest, stdout: &mut O) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let page = context::normalize_wiki_page_path(
        request
            .page
            .as_deref()
            .ok_or_else(|| CliError::InvalidInput("--page is required".to_string()))?,
    )?;

    let content = match (
        optional_trimmed(request.content.as_deref()),
        optional_trimmed(request.file.as_deref()),
    ) {
        (Some(_), Some(_)) => {
            return Err(CliError::InvalidInput(
                "use only one of --content or --file".to_string(),
            ));
        }
        (None, None) => {
            return Err(CliError::InvalidInput(
                "either --content or --file is required".to_string(),
            ));
        }
        (Some(content), None) => content.to_string(),
        (None, Some(path)) => fs::read_to_string(path)
            .map_err(|error| CliError::Io(format!("read --file: {error}")))?,
    };

    let profile = profile_from_config(request.profile.as_deref())?;
    let repo_dir = context::clone_wiki_to_temp(&profile, &workspace, &repo)
        .map_err(|error| CliError::Git(redact_token(&error.message(), &profile.token)))?;
    let abs_path = repo_dir.path().join(Path::new(&page));
    if let Some(parent) = abs_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| CliError::Io(format!("create wiki page directory: {error}")))?;
    }
    fs::write(&abs_path, content)
        .map_err(|error| CliError::Io(format!("write wiki page: {error}")))?;

    context::run_git(Some(repo_dir.path()), ["add", "--", page.as_str()])?;
    let status = context::run_git(
        Some(repo_dir.path()),
        ["status", "--porcelain", "--", page.as_str()],
    )?;
    if status.trim().is_empty() {
        return match output {
            WriteOutput::Json => {
                render::print_json(stdout, &json!({ "page": page, "status": "no_change" }))
            }
            WriteOutput::Text => {
                writeln!(stdout, "No changes for wiki page: {page}").map_err(CliError::from)
            }
        };
    }

    let commit_message = optional_trimmed(request.message.as_deref())
        .map(str::to_string)
        .unwrap_or_else(|| format!("Update wiki page {page}"));
    let (name, email) = commit_identity(&profile);
    context::run_git(
        Some(repo_dir.path()),
        ["config", "user.name", name.as_str()],
    )?;
    context::run_git(
        Some(repo_dir.path()),
        ["config", "user.email", email.as_str()],
    )?;
    context::run_git(
        Some(repo_dir.path()),
        ["commit", "-m", commit_message.as_str()],
    )?;
    context::run_git_with_askpass(
        Some(repo_dir.path()),
        &profile.token,
        ["push", "origin", "HEAD"],
    )
    .map_err(|error| CliError::Git(redact_token(&error.message(), &profile.token)))?;

    match output {
        WriteOutput::Json => {
            render::print_json(stdout, &json!({ "page": page, "status": "updated" }))
        }
        WriteOutput::Text => writeln!(stdout, "Updated wiki page: {page}").map_err(CliError::from),
    }
}
fn list_wiki_pages(root: &Path) -> Result<Vec<WikiPage>, CliError> {
    let mut rows = Vec::new();
    walk_wiki(root, root, &mut rows)?;
    rows.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(rows)
}

fn walk_wiki(root: &Path, dir: &Path, rows: &mut Vec<WikiPage>) -> Result<(), CliError> {
    for entry in
        fs::read_dir(dir).map_err(|error| CliError::Io(format!("list wiki pages: {error}")))?
    {
        let entry = entry.map_err(|error| CliError::Io(format!("list wiki pages: {error}")))?;
        let path = entry.path();
        let file_name = entry.file_name();
        if file_name.to_string_lossy() == ".git" {
            continue;
        }
        let file_type = entry
            .file_type()
            .map_err(|error| CliError::Io(format!("list wiki pages: {error}")))?;
        if file_type.is_dir() {
            walk_wiki(root, &path, rows)?;
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|error| CliError::Io(format!("list wiki pages: {error}")))?;
        let metadata = entry
            .metadata()
            .map_err(|error| CliError::Io(format!("list wiki pages: {error}")))?;
        rows.push(WikiPage {
            path: relative.to_string_lossy().replace('\\', "/"),
            size: metadata.len(),
        });
    }
    Ok(())
}

fn commit_identity(profile: &Profile) -> (String, String) {
    let username = profile.username.trim();
    if let Some((name, _)) = username.split_once('@') {
        if !name.trim().is_empty() {
            return (name.trim().to_string(), username.to_string());
        }
    }
    ("bb-cli".to_string(), "bb-cli@local".to_string())
}

fn redact_token(input: &str, token: &str) -> String {
    let token = token.trim();
    if token.is_empty() {
        input.to_string()
    } else {
        input.replace(token, "***")
    }
}
