use std::io::Write;

use reqwest::Method;
use serde_json::{Value, json};

use crate::client::Client;
use crate::context;
use crate::error::CliError;
use crate::render::{self, PrTableRow};
use crate::{
    ListOutput, PrActivityRequest, PrApproveRequest, PrCommentRequest, PrCommentUpdateRequest,
    PrCommentsRequest, PrCreateRequest, PrDeclineRequest, PrDiffRequest, PrDiffstatRequest,
    PrGetRequest, PrListRequest, PrMergeRequest, PrRemoveRequestChangesRequest, PrRequest,
    PrRequestChangesRequest, PrStatusesRequest, PrUnapproveRequest, PrUpdateRequest, WriteOutput,
};

use super::support::{
    client_from_profile, collect_query, optional_trimmed, parse_json_fields, parse_list_output,
    parse_write_output, print_json_list, print_json_object, required_string, set_optional_string,
};

const PR_JSON_FIELDS: &[&str] = &[
    "author",
    "close_source_branch",
    "comment_count",
    "created_on",
    "description",
    "destination",
    "draft",
    "id",
    "links",
    "participants",
    "reason",
    "reviewers",
    "source",
    "state",
    "summary",
    "task_count",
    "title",
    "updated_on",
];

const PR_COMMENTS_JSON_FIELDS: &[&str] = &[
    "content",
    "created_on",
    "deleted",
    "id",
    "inline",
    "links",
    "parent",
    "pullrequest",
    "updated_on",
    "user",
];

const PR_DIFFSTAT_JSON_FIELDS: &[&str] = &[
    "lines_added",
    "lines_removed",
    "new",
    "old",
    "status",
    "type",
];
const PR_STATUSES_JSON_FIELDS: &[&str] = &[
    "created_on",
    "description",
    "key",
    "links",
    "name",
    "refname",
    "state",
    "type",
    "updated_on",
    "url",
    "uuid",
];

const PR_ACTIVITY_JSON_FIELDS: &[&str] = &[
    "approval",
    "changes_request",
    "comment",
    "created_on",
    "decline",
    "merge",
    "request_changes",
    "task",
    "type",
    "update",
    "user",
];

pub(super) fn handle_pr<O: Write, E: Write>(
    request: &PrRequest,
    stdout: &mut O,
    stderr: &mut E,
    stdout_is_tty: bool,
    stderr_is_tty: bool,
) -> Result<(), CliError> {
    match request {
        PrRequest::Help => write!(stdout, "{}", render::pr_usage()).map_err(CliError::from),
        PrRequest::List(request) => {
            handle_pr_list(request, stdout, stderr, stdout_is_tty, stderr_is_tty)
        }
        PrRequest::Create(request) => handle_pr_create(request, stdout),
        PrRequest::Merge(request) => handle_pr_merge(request, stdout),
        PrRequest::Get(request) => handle_pr_get(request, stdout),
        PrRequest::Update(request) => handle_pr_update(request, stdout),
        PrRequest::Approve(request) => handle_pr_approve(request, stdout),
        PrRequest::Unapprove(request) => handle_pr_unapprove(request, stdout),
        PrRequest::RequestChanges(request) => handle_pr_request_changes(request, stdout),
        PrRequest::RemoveRequestChanges(request) => {
            handle_pr_remove_request_changes(request, stdout)
        }
        PrRequest::Decline(request) => handle_pr_decline(request, stdout),
        PrRequest::Comment(request) => handle_pr_comment(request, stdout),
        PrRequest::CommentUpdate(request) => handle_pr_comment_update(request, stdout),
        PrRequest::Comments(request) => handle_pr_comments(request, stdout),
        PrRequest::Diff(request) => handle_pr_diff(request, stdout),
        PrRequest::Diffstat(request) => handle_pr_diffstat(request, stdout),
        PrRequest::Statuses(request) => handle_pr_statuses(request, stdout),
        PrRequest::Activity(request) => handle_pr_activity(request, stdout),
    }
}

fn handle_pr_list<O: Write, E: Write>(
    request: &PrListRequest,
    stdout: &mut O,
    stderr: &mut E,
    stdout_is_tty: bool,
    stderr_is_tty: bool,
) -> Result<(), CliError> {
    let output = parse_list_output(&request.output)?;
    let json_fields = parse_json_fields(
        request.json_fields.as_deref(),
        output == ListOutput::Json,
        "bb pr list",
        PR_JSON_FIELDS,
    )?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let state = normalize_pr_state(request.state.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let query = collect_query([
        ("state", state.as_deref()),
        ("q", request.q.as_deref()),
        ("sort", request.sort.as_deref()),
        ("fields", request.fields.as_deref()),
    ]);
    let path = format!("/repositories/{workspace}/{repo}/pullrequests");

    let (values, total_count) = if request.all {
        (
            fetch_all_prs(&client, &path, &query, stderr, stderr_is_tty)?,
            None,
        )
    } else if let Some(limit) = request.limit {
        (client.get_values_up_to(&path, &query, limit)?, None)
    } else {
        let (values, total) = client.get_page(&path, &query)?;
        (values, total.map(|value| value as usize))
    };

    if output == ListOutput::Json {
        return print_json_list(stdout, &values, json_fields.as_deref());
    }
    let rows = values
        .iter()
        .map(|value| PrTableRow {
            id: render::int_field(value, &["id"]).unwrap_or_default() as u64,
            title: render::string_field(value, &["title"])
                .unwrap_or("-")
                .to_string(),
            branch: render::string_field(value, &["source", "branch", "name"])
                .unwrap_or("-")
                .to_string(),
            created_on: render::string_field(value, &["created_on"])
                .unwrap_or_default()
                .to_string(),
        })
        .collect::<Vec<_>>();
    let state_label = describe_pr_state_label(state.as_deref());
    write!(
        stdout,
        "{}",
        render::render_pr_table(
            &rows,
            &workspace,
            &repo,
            state_label,
            total_count,
            render::should_use_color(stdout_is_tty)
        )
    )
    .map_err(CliError::from)
}
fn fetch_all_prs<E: Write>(
    client: &Client,
    path: &str,
    query: &[(String, String)],
    stderr: &mut E,
    show_progress: bool,
) -> Result<Vec<Value>, CliError> {
    if !show_progress {
        return client.get_all_values(path, query);
    }

    let mut wrote_progress = false;
    let result = client.get_all_values_with_progress(path, query, |fetched, total| {
        wrote_progress = true;
        write_pr_progress(stderr, fetched, total)?;
        stderr.flush().map_err(CliError::from)
    });
    if wrote_progress {
        writeln!(stderr)?;
    }
    result
}

fn write_pr_progress<E: Write>(
    stderr: &mut E,
    fetched: usize,
    total: Option<u64>,
) -> Result<(), CliError> {
    match total {
        Some(total) => write!(stderr, "\rFetched {fetched}/{total} pull requests"),
        None => write!(stderr, "\rFetched {fetched} pull requests"),
    }
    .map_err(CliError::from)
}

fn handle_pr_create<O: Write>(request: &PrCreateRequest, stdout: &mut O) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let title = required_string("--title is required", request.title.as_deref())?;
    let source = required_string("--source is required", request.source.as_deref())?;
    let destination = required_string("--destination is required", request.destination.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let mut body = json!({
        "title": title,
        "source": { "branch": { "name": source } },
        "destination": { "branch": { "name": destination } },
    });
    if let Some(description) = optional_trimmed(request.description.as_deref()) {
        body["description"] = Value::String(description.to_string());
    }
    if request.close_branch {
        body["close_source_branch"] = Value::Bool(true);
    }

    let value = client.request_value(
        Method::POST,
        &format!("/repositories/{workspace}/{repo}/pullrequests"),
        &[],
        Some(body),
    )?;

    match output {
        WriteOutput::Json => render::print_json(stdout, &value),
        WriteOutput::Text => {
            writeln!(
                stdout,
                "Created PR #{} ({}): {}",
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

fn handle_pr_merge<O: Write>(request: &PrMergeRequest, stdout: &mut O) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_pr_numeric_id(request.id.as_deref())?;
    let strategy = normalize_merge_strategy(request.strategy.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let mut body = json!({});
    if let Some(message) = optional_trimmed(request.message.as_deref()) {
        body["message"] = Value::String(message.to_string());
    }
    if let Some(strategy) = strategy.as_deref() {
        body["merge_strategy"] = Value::String(strategy.to_string());
    }
    if request.close_branch {
        body["close_source_branch"] = Value::Bool(true);
    }

    let value = client.request_value(
        Method::POST,
        &format!("/repositories/{workspace}/{repo}/pullrequests/{id}/merge"),
        &[],
        Some(body),
    )?;

    match output {
        WriteOutput::Json => render::print_json(stdout, &value),
        WriteOutput::Text => {
            writeln!(
                stdout,
                "Merged PR #{} ({}): {}",
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

fn handle_pr_get<O: Write>(request: &PrGetRequest, stdout: &mut O) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let json_fields = parse_json_fields(
        request.json_fields.as_deref(),
        output == WriteOutput::Json,
        "bb pr get",
        PR_JSON_FIELDS,
    )?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_pr_numeric_id(request.id.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let value = client.request_value(
        Method::GET,
        &format!("/repositories/{workspace}/{repo}/pullrequests/{id}"),
        &collect_query([("fields", request.fields.as_deref())]),
        None,
    )?;

    match output {
        WriteOutput::Json => print_json_object(stdout, &value, json_fields.as_deref()),
        WriteOutput::Text => {
            writeln!(
                stdout,
                "PR #{} ({})",
                render::int_field(&value, &["id"]).unwrap_or_default(),
                render::string_field(&value, &["state"]).unwrap_or("-")
            )?;
            writeln!(
                stdout,
                "Title: {}",
                render::string_field(&value, &["title"]).unwrap_or("-")
            )?;
            writeln!(
                stdout,
                "Source: {}",
                render::string_field(&value, &["source", "branch", "name"]).unwrap_or("-")
            )?;
            writeln!(
                stdout,
                "Destination: {}",
                render::string_field(&value, &["destination", "branch", "name"]).unwrap_or("-")
            )?;
            if let Some(author) = render::string_field(&value, &["author", "display_name"]) {
                if !author.trim().is_empty() {
                    writeln!(stdout, "Author: {author}")?;
                }
            }
            if let Some(description) = render::string_field(&value, &["description"]) {
                if !description.trim().is_empty() {
                    writeln!(stdout, "Description: {description}")?;
                }
            }
            if let Some(url) = render::string_field(&value, &["links", "html", "href"]) {
                if !url.trim().is_empty() {
                    writeln!(stdout, "URL: {url}")?;
                }
            }
            Ok(())
        }
    }
}

fn handle_pr_update<O: Write>(request: &PrUpdateRequest, stdout: &mut O) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_pr_numeric_id(request.id.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let mut body = json!({});

    set_optional_string(&mut body, "title", request.title.as_deref());
    set_optional_string(&mut body, "description", request.description.as_deref());
    if let Some(source) = optional_trimmed(request.source.as_deref()) {
        body["source"] = json!({ "branch": { "name": source } });
    }
    if let Some(destination) = optional_trimmed(request.destination.as_deref()) {
        body["destination"] = json!({ "branch": { "name": destination } });
    }

    if body
        .as_object()
        .map(|value| value.is_empty())
        .unwrap_or(false)
    {
        return Err(CliError::InvalidInput(
            "at least one of --title, --description, --source, --destination is required"
                .to_string(),
        ));
    }

    let value = client.request_value(
        Method::PUT,
        &format!("/repositories/{workspace}/{repo}/pullrequests/{id}"),
        &[],
        Some(body),
    )?;

    match output {
        WriteOutput::Json => render::print_json(stdout, &value),
        WriteOutput::Text => write_pr_response_text(stdout, "Updated", &value),
    }
}

fn handle_pr_approve<O: Write>(request: &PrApproveRequest, stdout: &mut O) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_pr_numeric_id(request.id.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let value = client.request_value(
        Method::POST,
        &format!("/repositories/{workspace}/{repo}/pullrequests/{id}/approve"),
        &[],
        None,
    )?;

    match output {
        WriteOutput::Json => render::print_json(stdout, &value),
        WriteOutput::Text => write_pr_participant_action_text(stdout, "Approved", &id, &value),
    }
}

fn handle_pr_unapprove<O: Write>(
    request: &PrUnapproveRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_pr_numeric_id(request.id.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    client.request_text(
        Method::DELETE,
        &format!("/repositories/{workspace}/{repo}/pullrequests/{id}/approve"),
        &[],
    )?;
    write_pr_no_content_action(stdout, output, &id, "Removed approval from")?;
    Ok(())
}

fn handle_pr_request_changes<O: Write>(
    request: &PrRequestChangesRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_pr_numeric_id(request.id.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let value = client.request_value(
        Method::POST,
        &format!("/repositories/{workspace}/{repo}/pullrequests/{id}/request-changes"),
        &[],
        None,
    )?;

    match output {
        WriteOutput::Json => render::print_json(stdout, &value),
        WriteOutput::Text => {
            write_pr_participant_action_text(stdout, "Requested changes on", &id, &value)
        }
    }
}

fn handle_pr_remove_request_changes<O: Write>(
    request: &PrRemoveRequestChangesRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_pr_numeric_id(request.id.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    client.request_text(
        Method::DELETE,
        &format!("/repositories/{workspace}/{repo}/pullrequests/{id}/request-changes"),
        &[],
    )?;
    write_pr_no_content_action(stdout, output, &id, "Removed change request from")?;
    Ok(())
}

fn handle_pr_decline<O: Write>(request: &PrDeclineRequest, stdout: &mut O) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_pr_numeric_id(request.id.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let value = client.request_value(
        Method::POST,
        &format!("/repositories/{workspace}/{repo}/pullrequests/{id}/decline"),
        &[],
        None,
    )?;

    match output {
        WriteOutput::Json => render::print_json(stdout, &value),
        WriteOutput::Text => write_pr_response_text(stdout, "Declined", &value),
    }
}

fn handle_pr_comment<O: Write>(request: &PrCommentRequest, stdout: &mut O) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_pr_numeric_id(request.id.as_deref())?;
    let content = required_string("--content is required", request.content.as_deref())?;
    let parent = parse_parent_comment_id(request.parent.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let mut body = json!({
        "content": {
            "raw": content,
        }
    });
    if let Some(parent) = parent {
        body["parent"] = json!({ "id": parent });
    }
    let value = client.request_value(
        Method::POST,
        &format!("/repositories/{workspace}/{repo}/pullrequests/{id}/comments"),
        &[],
        Some(body),
    )?;

    match output {
        WriteOutput::Json => render::print_json(stdout, &value),
        WriteOutput::Text => {
            writeln!(
                stdout,
                "Created comment #{} on PR #{}",
                render::int_field(&value, &["id"]).unwrap_or_default(),
                id
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

fn handle_pr_comment_update<O: Write>(
    request: &PrCommentUpdateRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let pr_id = parse_pr_numeric_id(request.id.as_deref())?;
    let comment_id = parse_comment_numeric_id(request.comment_id.as_deref())?;
    let content = required_string("--content is required", request.content.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let body = json!({
        "content": {
            "raw": content,
        }
    });
    let value = client.request_value(
        Method::PUT,
        &format!("/repositories/{workspace}/{repo}/pullrequests/{pr_id}/comments/{comment_id}"),
        &[],
        Some(body),
    )?;

    match output {
        WriteOutput::Json => render::print_json(stdout, &value),
        WriteOutput::Text => {
            writeln!(stdout, "Updated comment #{comment_id} on PR #{pr_id}")?;
            if let Some(url) = render::string_field(&value, &["links", "html", "href"]) {
                if !url.trim().is_empty() {
                    writeln!(stdout, "URL: {url}")?;
                }
            }
            Ok(())
        }
    }
}

fn handle_pr_comments<O: Write>(
    request: &PrCommentsRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    validate_pr_comment_lookup_options(request)?;
    let output = parse_list_output(&request.output)?;
    let json_fields = parse_json_fields(
        request.json_fields.as_deref(),
        output == ListOutput::Json,
        "bb pr comments",
        PR_COMMENTS_JSON_FIELDS,
    )?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let pr_id = parse_pr_numeric_id(request.id.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let query = collect_query([("fields", request.fields.as_deref())]);

    if let Some(comment_id) = request.comment_id.as_deref() {
        let comment_id = parse_comment_numeric_id(Some(comment_id))?;
        let value = client.request_value(
            Method::GET,
            &format!("/repositories/{workspace}/{repo}/pullrequests/{pr_id}/comments/{comment_id}"),
            &query,
            None,
        )?;

        return match output {
            ListOutput::Json => print_json_object(stdout, &value, json_fields.as_deref()),
            ListOutput::Table => write!(stdout, "{}", render::render_pr_comment_detail(&value))
                .map_err(CliError::from),
        };
    }

    let path = format!("/repositories/{workspace}/{repo}/pullrequests/{pr_id}/comments");
    let list_query = collect_query([
        ("q", request.q.as_deref()),
        ("sort", request.sort.as_deref()),
        ("fields", request.fields.as_deref()),
    ]);
    let values = fetch_values(&client, &path, &list_query, request.all)?;

    match output {
        ListOutput::Json => print_json_list(stdout, &values, json_fields.as_deref()),
        ListOutput::Table => {
            write!(stdout, "{}", render::render_pr_comments_table(&values)).map_err(CliError::from)
        }
    }
}

fn handle_pr_diff<O: Write>(request: &PrDiffRequest, stdout: &mut O) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_pr_numeric_id(request.id.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    if request.name_only {
        let values = client.get_all_values(
            &format!("/repositories/{workspace}/{repo}/pullrequests/{id}/diffstat"),
            &[],
        )?;
        return match output {
            WriteOutput::Json => {
                let paths = values
                    .iter()
                    .filter_map(render::pr_diffstat_path)
                    .collect::<Vec<_>>();
                render::print_json(stdout, &paths)
            }
            WriteOutput::Text => {
                for value in &values {
                    if let Some(path) = render::pr_diffstat_path(value) {
                        writeln!(stdout, "{path}")?;
                    }
                }
                Ok(())
            }
        };
    }
    let diff = client.request_text(
        Method::GET,
        &format!("/repositories/{workspace}/{repo}/pullrequests/{id}/diff"),
        &[],
    )?;

    match output {
        WriteOutput::Json => render::print_json(stdout, &json!({ "diff": diff })),
        WriteOutput::Text => {
            write!(stdout, "{diff}")?;
            if !diff.ends_with('\n') {
                writeln!(stdout)?;
            }
            Ok(())
        }
    }
}

fn handle_pr_diffstat<O: Write>(
    request: &PrDiffstatRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let output = parse_list_output(&request.output)?;
    let json_fields = parse_json_fields(
        request.json_fields.as_deref(),
        output == ListOutput::Json,
        "bb pr diffstat",
        PR_DIFFSTAT_JSON_FIELDS,
    )?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_pr_numeric_id(request.id.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let path = format!("/repositories/{workspace}/{repo}/pullrequests/{id}/diffstat");
    let query = collect_query([
        ("q", request.q.as_deref()),
        ("sort", request.sort.as_deref()),
        ("fields", request.fields.as_deref()),
    ]);
    let values = fetch_values(&client, &path, &query, request.all)?;

    match output {
        ListOutput::Json => print_json_list(stdout, &values, json_fields.as_deref()),
        ListOutput::Table => {
            write!(stdout, "{}", render::render_pr_diffstat_table(&values)).map_err(CliError::from)
        }
    }
}

fn handle_pr_statuses<O: Write>(
    request: &PrStatusesRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let output = parse_list_output(&request.output)?;
    let json_fields = parse_json_fields(
        request.json_fields.as_deref(),
        output == ListOutput::Json,
        "bb pr statuses",
        PR_STATUSES_JSON_FIELDS,
    )?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_pr_numeric_id(request.id.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let path = format!("/repositories/{workspace}/{repo}/pullrequests/{id}/statuses");
    let query = collect_query([
        ("q", request.q.as_deref()),
        ("sort", request.sort.as_deref()),
        ("fields", request.fields.as_deref()),
    ]);
    let values = fetch_values(&client, &path, &query, request.all)?;

    match output {
        ListOutput::Json => print_json_list(stdout, &values, json_fields.as_deref()),
        ListOutput::Table => {
            write!(stdout, "{}", render::render_pr_statuses_table(&values)).map_err(CliError::from)
        }
    }
}

fn handle_pr_activity<O: Write>(
    request: &PrActivityRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let output = parse_list_output(&request.output)?;
    let json_fields = parse_json_fields(
        request.json_fields.as_deref(),
        output == ListOutput::Json,
        "bb pr activity",
        PR_ACTIVITY_JSON_FIELDS,
    )?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_pr_numeric_id(request.id.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let path = format!("/repositories/{workspace}/{repo}/pullrequests/{id}/activity");
    let query = collect_query([
        ("q", request.q.as_deref()),
        ("sort", request.sort.as_deref()),
        ("fields", request.fields.as_deref()),
    ]);
    let values = fetch_values(&client, &path, &query, request.all)?;

    match output {
        ListOutput::Json => print_json_list(stdout, &values, json_fields.as_deref()),
        ListOutput::Table => {
            write!(stdout, "{}", render::render_pr_activity_table(&values)).map_err(CliError::from)
        }
    }
}

fn fetch_values(
    client: &Client,
    path: &str,
    query: &[(String, String)],
    all: bool,
) -> Result<Vec<Value>, CliError> {
    if all {
        client.get_all_values(path, query)
    } else {
        Ok(client.get_page(path, query)?.0)
    }
}

fn write_pr_response_text<O: Write>(
    stdout: &mut O,
    action: &str,
    value: &Value,
) -> Result<(), CliError> {
    writeln!(
        stdout,
        "{action} PR #{} ({}): {}",
        render::int_field(value, &["id"]).unwrap_or_default(),
        render::string_field(value, &["state"]).unwrap_or("-"),
        render::string_field(value, &["title"]).unwrap_or("-")
    )?;
    if let Some(url) = render::string_field(value, &["links", "html", "href"]) {
        if !url.trim().is_empty() {
            writeln!(stdout, "URL: {url}")?;
        }
    }
    Ok(())
}

fn write_pr_participant_action_text<O: Write>(
    stdout: &mut O,
    action: &str,
    pr_id: &str,
    value: &Value,
) -> Result<(), CliError> {
    let actor = render::string_field(value, &["user", "display_name"])
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!(" by {value}"))
        .unwrap_or_default();
    writeln!(stdout, "{action} PR #{pr_id}{actor}")?;
    Ok(())
}

fn write_pr_no_content_action<O: Write>(
    stdout: &mut O,
    output: WriteOutput,
    pr_id: &str,
    action: &str,
) -> Result<(), CliError> {
    match output {
        WriteOutput::Text => writeln!(stdout, "{action} PR #{pr_id}").map_err(CliError::from),
        WriteOutput::Json => render::print_json(
            stdout,
            &json!({
                "id": pr_id.parse::<u64>().unwrap_or_default(),
                "action": action,
                "ok": true,
            }),
        ),
    }
}
fn parse_parent_comment_id(parent: Option<&str>) -> Result<Option<u64>, CliError> {
    let Some(parent) = optional_trimmed(parent) else {
        return Ok(None);
    };

    let parent = parent
        .parse::<u64>()
        .map_err(|_| CliError::InvalidInput("--parent must be a numeric comment ID".to_string()))?;
    Ok(Some(parent))
}
fn normalize_pr_state(value: Option<&str>) -> Result<Option<String>, CliError> {
    let Some(value) = optional_trimmed(value) else {
        return Ok(None);
    };
    let value = value.to_uppercase();
    match value.as_str() {
        "OPEN" | "MERGED" | "DECLINED" => Ok(Some(value)),
        _ => Err(CliError::InvalidInput(
            "--state must be one of OPEN, MERGED, DECLINED".to_string(),
        )),
    }
}

fn describe_pr_state_label(state: Option<&str>) -> &'static str {
    match state.unwrap_or_default() {
        "OPEN" => "open pull requests",
        "MERGED" => "merged pull requests",
        "DECLINED" => "declined pull requests",
        _ => "pull requests",
    }
}

fn normalize_merge_strategy(value: Option<&str>) -> Result<Option<String>, CliError> {
    let Some(value) = optional_trimmed(value) else {
        return Ok(None);
    };
    let value = value.to_lowercase();
    match value.as_str() {
        "merge_commit" | "squash" | "fast_forward" => Ok(Some(value)),
        _ => Err(CliError::InvalidInput(
            "--strategy must be one of merge_commit, squash, fast_forward".to_string(),
        )),
    }
}

fn parse_pr_numeric_id(value: Option<&str>) -> Result<String, CliError> {
    let value = required_string("pull request id is required: pass <id> or --id", value)?;
    value
        .parse::<u64>()
        .map(|_| value.to_string())
        .map_err(|_| CliError::InvalidInput(format!("pull request id must be a number: {value}")))
}

fn parse_comment_numeric_id(value: Option<&str>) -> Result<String, CliError> {
    let value = required_string("comment id is required: pass --comment-id", value)?;
    value
        .parse::<u64>()
        .map(|_| value.to_string())
        .map_err(|_| {
            CliError::InvalidInput("--comment-id must be a numeric comment ID".to_string())
        })
}

fn validate_pr_comment_lookup_options(request: &PrCommentsRequest) -> Result<(), CliError> {
    if request.comment_id.is_some()
        && (request.all || request.q.is_some() || request.sort.is_some())
    {
        return Err(CliError::InvalidInput(
            "--comment-id cannot be combined with --all, --q, or --sort".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::config::Profile;
    use httpmock::Method::GET;
    use httpmock::MockServer;

    use serde_json::json;

    use super::*;

    #[test]
    fn write_pr_progress_uses_total_when_available() {
        let mut stderr = Vec::new();
        write_pr_progress(&mut stderr, 25, Some(100)).expect("progress should render");
        assert_eq!(stderr, b"\rFetched 25/100 pull requests");
    }

    #[test]
    fn fetch_all_prs_writes_each_page_when_progress_is_enabled() {
        let server = MockServer::start();
        let page2 = server.mock(|when, then| {
            when.method(GET).path("/page2");
            then.json_body(json!({"values":[{"id":2}],"size":2}));
        });
        let page1 = server.mock(|when, then| {
            when.method(GET)
                .path("/pullrequests")
                .query_param("pagelen", "100");
            then.json_body(json!({
                "values":[{"id":1}],
                "size":2,
                "next": format!("{}/page2", server.base_url())
            }));
        });
        let client = Client::from_profile(&Profile {
            base_url: server.base_url(),
            token: "token".to_string(),
            username: String::new(),
        })
        .unwrap();
        let mut stderr = Vec::new();

        let values = fetch_all_prs(&client, "/pullrequests", &[], &mut stderr, true)
            .expect("paginated fetch should succeed");

        assert_eq!(values.len(), 2);
        assert_eq!(
            stderr,
            b"\rFetched 1/2 pull requests\rFetched 2/2 pull requests\n"
        );
        page1.assert();
        page2.assert();
    }
}
