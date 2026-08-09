use std::io::Write;

use reqwest::Method;
use serde_json::json;

use crate::context;
use crate::error::CliError;
use crate::render;
use crate::{
    IssueCreateRequest, IssueListRequest, IssueRequest, IssueUpdateRequest, ListOutput, WriteOutput,
};

use super::support::{
    client_from_profile, collect_query, optional_trimmed, parse_list_output, parse_write_output,
    required_string, set_optional_string,
};

pub(super) fn handle_issue<O: Write>(
    request: &IssueRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    match request {
        IssueRequest::Help => write!(stdout, "{}", render::issue_usage()).map_err(CliError::from),
        IssueRequest::List(request) => handle_issue_list(request, stdout),
        IssueRequest::Create(request) => handle_issue_create(request, stdout),
        IssueRequest::Update(request) => handle_issue_update(request, stdout),
    }
}

fn handle_issue_list<O: Write>(request: &IssueListRequest, stdout: &mut O) -> Result<(), CliError> {
    let output = parse_list_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let client = client_from_profile(request.profile.as_deref())?;
    let query = collect_query([
        ("q", request.q.as_deref()),
        ("sort", request.sort.as_deref()),
        ("fields", request.fields.as_deref()),
    ]);
    let path = format!("/repositories/{workspace}/{repo}/issues");
    let values = if request.all {
        client.get_all_values(&path, &query)?
    } else {
        client.get_page(&path, &query)?.0
    };

    match output {
        ListOutput::Json => render::print_json(stdout, &values),
        ListOutput::Table => {
            write!(stdout, "{}", render::render_issue_table(&values)).map_err(CliError::from)
        }
    }
}

fn handle_issue_create<O: Write>(
    request: &IssueCreateRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let title = required_string("--title is required", request.title.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let mut body = json!({ "title": title });
    if let Some(content) = optional_trimmed(request.content.as_deref()) {
        body["content"] = json!({ "raw": content });
    }
    set_optional_string(&mut body, "state", request.state.as_deref());
    set_optional_string(&mut body, "kind", request.kind.as_deref());
    set_optional_string(&mut body, "priority", request.priority.as_deref());

    let value = client.request_value(
        Method::POST,
        &format!("/repositories/{workspace}/{repo}/issues"),
        &[],
        Some(body),
    )?;

    match output {
        WriteOutput::Json => render::print_json(stdout, &value),
        WriteOutput::Text => {
            writeln!(
                stdout,
                "Created issue #{} ({}): {}",
                render::int_field(&value, &["id"]).unwrap_or_default(),
                render::string_field(&value, &["state"]).unwrap_or("-"),
                render::string_field(&value, &["title"]).unwrap_or("-")
            )?;
            if let Some(url) = render::string_field(&value, &["links", "html", "href"]) {
                if !url.trim().is_empty() {
                    writeln!(stdout, "URL: {url}")?;
                }
            }
            Ok(())
        }
    }
}

fn handle_issue_update<O: Write>(
    request: &IssueUpdateRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = request
        .id
        .filter(|value| *value > 0)
        .ok_or_else(|| CliError::InvalidInput("--id is required".to_string()))?;
    let client = client_from_profile(request.profile.as_deref())?;
    let mut body = json!({});
    set_optional_string(&mut body, "title", request.title.as_deref());
    set_optional_string(&mut body, "state", request.state.as_deref());
    set_optional_string(&mut body, "kind", request.kind.as_deref());
    set_optional_string(&mut body, "priority", request.priority.as_deref());
    if let Some(content) = optional_trimmed(request.content.as_deref()) {
        body["content"] = json!({ "raw": content });
    }
    if body
        .as_object()
        .map(|value| value.is_empty())
        .unwrap_or(true)
    {
        return Err(CliError::InvalidInput(
            "at least one field to update is required".to_string(),
        ));
    }

    let value = client.request_value(
        Method::PUT,
        &format!("/repositories/{workspace}/{repo}/issues/{id}"),
        &[],
        Some(body),
    )?;

    match output {
        WriteOutput::Json => render::print_json(stdout, &value),
        WriteOutput::Text => {
            writeln!(
                stdout,
                "Updated issue #{} ({}): {}",
                render::int_field(&value, &["id"]).unwrap_or_default(),
                render::string_field(&value, &["state"]).unwrap_or("-"),
                render::string_field(&value, &["title"]).unwrap_or("-")
            )?;
            if let Some(url) = render::string_field(&value, &["links", "html", "href"]) {
                if !url.trim().is_empty() {
                    writeln!(stdout, "URL: {url}")?;
                }
            }
            Ok(())
        }
    }
}
