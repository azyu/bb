use std::fs;
use std::io::{BufRead, Write};
use std::path::Path;

use reqwest::Method;
use serde_json::{Value, json};

use crate::client::Client;
use crate::config::{self, Profile};
use crate::context;
use crate::error::CliError;
use crate::render::{self, ErrorEnvelope, ErrorPayload, PrTableRow, WikiPage};
use crate::version;
use crate::{
    ApiRequest, AuthLoginRequest, AuthRequest, AuthStatusRequest, CompletionShell,
    IssueCreateRequest, IssueListRequest, IssueRequest, IssueUpdateRequest, ListOutput,
    PipelineListRequest, PipelineRequest, PipelineRunRequest, PrActivityRequest, PrApproveRequest,
    PrCommentRequest, PrCommentsRequest, PrCreateRequest, PrDeclineRequest, PrDiffRequest,
    PrGetRequest, PrListRequest, PrMergeRequest, PrRemoveRequestChangesRequest, PrRequest,
    PrRequestChangesRequest, PrStatusesRequest, PrUnapproveRequest, PrUpdateRequest,
    RepoListRequest, RepoRequest, Request, WikiGetRequest, WikiListRequest, WikiPutRequest,
    WikiRequest, WriteOutput,
};

pub const STDIN_TOKEN_SENTINEL: &str = "__bb_stdin_token__";

pub fn run<R: BufRead, O: Write, E: Write>(
    request: Request,
    stdin: &mut R,
    stdout: &mut O,
    stderr: &mut E,
    stdout_is_tty: bool,
) -> u8 {
    match dispatch(&request, stdin, stdout, stdout_is_tty) {
        Ok(()) => 0,
        Err(error) => {
            let _ = emit_error(&request, &error, stdout, stderr);
            1
        }
    }
}

fn dispatch<R: BufRead, O: Write>(
    request: &Request,
    stdin: &mut R,
    stdout: &mut O,
    stdout_is_tty: bool,
) -> Result<(), CliError> {
    match request {
        Request::RootHelp => {
            write!(stdout, "{}", render::root_usage())?;
        }
        Request::Version => {
            write!(
                stdout,
                "bb version {}\ncommit: {}\nbuilt: {}\n",
                version::display_version(),
                version::short_commit(),
                version::BUILD_DATE
            )?;
        }
        Request::Completion(shell) => {
            let Some(shell) = shell.as_deref() else {
                write!(stdout, "{}", render::completion_usage())?;
                return Ok(());
            };
            let script = match normalize_completion_shell(shell)? {
                CompletionShell::Bash => render::bash_completion_script(),
                CompletionShell::Zsh => render::zsh_completion_script(),
                CompletionShell::Fish => render::fish_completion_script(),
                CompletionShell::Powershell => render::powershell_completion_script(),
            };
            writeln!(stdout, "{script}")?;
        }
        Request::Auth(auth) => handle_auth(auth, stdin, stdout)?,
        Request::Api(api) => handle_api(api, stdout)?,
        Request::Repo(repo) => handle_repo(repo, stdout)?,
        Request::Pr(pr) => handle_pr(pr, stdout, stdout_is_tty)?,
        Request::Pipeline(pipeline) => handle_pipeline(pipeline, stdout)?,
        Request::Issue(issue) => handle_issue(issue, stdout)?,
        Request::Wiki(wiki) => handle_wiki(wiki, stdout)?,
    }

    Ok(())
}

fn emit_error<O: Write, E: Write>(
    request: &Request,
    error: &CliError,
    stdout: &mut O,
    stderr: &mut E,
) -> Result<(), CliError> {
    if wants_json_errors(request) {
        return render::print_json(
            stdout,
            &ErrorEnvelope {
                error: ErrorPayload {
                    code: error.code(),
                    message: error.message(),
                },
            },
        );
    }
    writeln!(stderr, "{}", error.message()).map_err(CliError::from)
}

fn wants_json_errors(request: &Request) -> bool {
    match request {
        Request::Api(_) => true,
        Request::Repo(RepoRequest::List(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::List(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Create(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Merge(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Get(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Update(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Approve(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Unapprove(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::RequestChanges(req)) => {
            req.output.trim().eq_ignore_ascii_case("json")
        }
        Request::Pr(PrRequest::RemoveRequestChanges(req)) => {
            req.output.trim().eq_ignore_ascii_case("json")
        }
        Request::Pr(PrRequest::Decline(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Comment(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Comments(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Diff(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Statuses(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Activity(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pipeline(PipelineRequest::List(req)) => {
            req.output.trim().eq_ignore_ascii_case("json")
        }
        Request::Pipeline(PipelineRequest::Run(req)) => {
            req.output.trim().eq_ignore_ascii_case("json")
        }
        Request::Issue(IssueRequest::List(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Issue(IssueRequest::Create(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Issue(IssueRequest::Update(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Wiki(WikiRequest::List(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Wiki(WikiRequest::Get(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Wiki(WikiRequest::Put(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        _ => false,
    }
}

fn handle_auth<R: BufRead, O: Write>(
    request: &AuthRequest,
    stdin: &mut R,
    stdout: &mut O,
) -> Result<(), CliError> {
    match request {
        AuthRequest::Help => write!(stdout, "{}", render::auth_usage()).map_err(CliError::from),
        AuthRequest::Login(request) => handle_auth_login(request, stdin, stdout),
        AuthRequest::Status(request) => handle_auth_status(request, stdout),
        AuthRequest::Logout(request) => handle_auth_logout(request, stdout),
    }
}

fn handle_auth_login<R: BufRead, O: Write>(
    request: &AuthLoginRequest,
    stdin: &mut R,
    stdout: &mut O,
) -> Result<(), CliError> {
    let token = resolve_login_token(request, stdin)?;
    let username = request
        .username
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            std::env::var("BITBUCKET_USERNAME")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_default();

    let mut config = config::load()?;
    config.set_profile_with_auth(
        &request.profile,
        &username,
        &token,
        request.base_url.as_deref(),
    );
    config::save(&config)?;

    writeln!(stdout, "authenticated profile {:?}", request.profile)?;
    if username.is_empty() {
        writeln!(stdout, "auth mode: bearer token")?;
    } else {
        writeln!(stdout, "auth mode: basic ({username})")?;
    }
    Ok(())
}

fn handle_auth_status<O: Write>(
    request: &AuthStatusRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let config = config::load()?;
    let (profile, name) = config.active_profile(request.profile.as_deref())?;

    writeln!(stdout, "Profile: {name}")?;
    writeln!(stdout, "Base URL: {}", profile.base_url)?;
    if profile.username.trim().is_empty() {
        writeln!(stdout, "Auth: bearer token")?;
    } else {
        writeln!(stdout, "Auth: basic ({})", profile.username.trim())?;
    }
    if profile.token.trim().is_empty() {
        writeln!(stdout, "Token: not configured")?;
    } else {
        writeln!(stdout, "Token: configured")?;
    }
    Ok(())
}

fn handle_auth_logout<O: Write>(
    request: &crate::AuthLogoutRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let mut config = config::load()?;
    if request
        .profile
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
        && config.current.trim().is_empty()
    {
        return Err(CliError::NotLoggedIn);
    }

    let (removed, ok) = config.remove_profile(request.profile.as_deref());
    if !ok {
        return if removed.trim().is_empty() {
            Err(CliError::NotLoggedIn)
        } else {
            Err(CliError::Config(format!("profile {:?} not found", removed)))
        };
    }

    config::save(&config)?;
    writeln!(stdout, "logged out profile {:?}", removed)?;
    if !config.current.trim().is_empty() {
        writeln!(stdout, "active profile: {:?}", config.current)?;
    }
    Ok(())
}

fn handle_api<O: Write>(request: &ApiRequest, stdout: &mut O) -> Result<(), CliError> {
    let endpoint = request
        .endpoint
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| CliError::InvalidInput("usage: bb api [flags] <endpoint>".to_string()))?;
    let client = client_from_profile(request.profile.as_deref())?;
    let query = collect_query([
        ("q", request.q.as_deref()),
        ("sort", request.sort.as_deref()),
        ("fields", request.fields.as_deref()),
    ]);

    if request.paginate {
        let values = client.get_all_values(endpoint, &query)?;
        return render::print_json(stdout, &values);
    }

    let method = request.method.trim().to_uppercase();
    let value = client.request_value(
        Method::from_bytes(method.as_bytes())
            .map_err(|error| CliError::InvalidInput(format!("invalid HTTP method: {error}")))?,
        endpoint,
        &query,
        None,
    )?;
    render::print_json(stdout, &value)
}

fn handle_repo<O: Write>(request: &RepoRequest, stdout: &mut O) -> Result<(), CliError> {
    match request {
        RepoRequest::Help => write!(stdout, "{}", render::repo_usage()).map_err(CliError::from),
        RepoRequest::List(request) => handle_repo_list(request, stdout),
    }
}

fn handle_repo_list<O: Write>(request: &RepoListRequest, stdout: &mut O) -> Result<(), CliError> {
    let output = parse_list_output(&request.output)?;
    let (workspace, _) = context::resolve_repo_target(request.workspace.as_deref(), None, false)?;
    let client = client_from_profile(request.profile.as_deref())?;
    let query = collect_query([
        ("q", request.q.as_deref()),
        ("sort", request.sort.as_deref()),
        ("fields", request.fields.as_deref()),
    ]);
    let path = format!("/repositories/{workspace}");

    let values = if request.all {
        client.get_all_values(&path, &query)?
    } else {
        client.get_page(&path, &query)?.0
    };

    match output {
        ListOutput::Json => render::print_json(stdout, &values),
        ListOutput::Table => {
            write!(stdout, "{}", render::render_repo_table(&values)).map_err(CliError::from)
        }
    }
}

fn handle_pr<O: Write>(
    request: &PrRequest,
    stdout: &mut O,
    stdout_is_tty: bool,
) -> Result<(), CliError> {
    match request {
        PrRequest::Help => write!(stdout, "{}", render::pr_usage()).map_err(CliError::from),
        PrRequest::List(request) => handle_pr_list(request, stdout, stdout_is_tty),
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
        PrRequest::Comments(request) => handle_pr_comments(request, stdout),
        PrRequest::Diff(request) => handle_pr_diff(request, stdout),
        PrRequest::Statuses(request) => handle_pr_statuses(request, stdout),
        PrRequest::Activity(request) => handle_pr_activity(request, stdout),
    }
}

fn handle_pr_list<O: Write>(
    request: &PrListRequest,
    stdout: &mut O,
    stdout_is_tty: bool,
) -> Result<(), CliError> {
    let output = parse_list_output(&request.output)?;
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

    if output == ListOutput::Json {
        let values = if request.all {
            client.get_all_values(&path, &query)?
        } else {
            client.get_page(&path, &query)?.0
        };
        return render::print_json(stdout, &values);
    }

    let (values, total_count) = if request.all {
        (client.get_all_values(&path, &query)?, None)
    } else {
        let (values, total) = client.get_page(&path, &query)?;
        (values, total.map(|value| value as usize))
    };
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
    let id = parse_numeric_id(request.id.as_deref(), "--id is required")?;
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
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_numeric_id(request.id.as_deref(), "--id is required")?;
    let client = client_from_profile(request.profile.as_deref())?;
    let value = client.request_value(
        Method::GET,
        &format!("/repositories/{workspace}/{repo}/pullrequests/{id}"),
        &collect_query([("fields", request.fields.as_deref())]),
        None,
    )?;

    match output {
        WriteOutput::Json => render::print_json(stdout, &value),
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
    let id = parse_numeric_id(request.id.as_deref(), "--id is required")?;
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
    let id = parse_numeric_id(request.id.as_deref(), "--id is required")?;
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
    let id = parse_numeric_id(request.id.as_deref(), "--id is required")?;
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
    let id = parse_numeric_id(request.id.as_deref(), "--id is required")?;
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
    let id = parse_numeric_id(request.id.as_deref(), "--id is required")?;
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
    let id = parse_numeric_id(request.id.as_deref(), "--id is required")?;
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
    let id = parse_numeric_id(request.id.as_deref(), "--id is required")?;
    let content = required_string("--content is required", request.content.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let value = client.request_value(
        Method::POST,
        &format!("/repositories/{workspace}/{repo}/pullrequests/{id}/comments"),
        &[],
        Some(json!({
            "content": {
                "raw": content,
            }
        })),
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

fn handle_pr_comments<O: Write>(
    request: &PrCommentsRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let output = parse_list_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_numeric_id(request.id.as_deref(), "--id is required")?;
    let client = client_from_profile(request.profile.as_deref())?;
    let path = format!("/repositories/{workspace}/{repo}/pullrequests/{id}/comments");
    let query = collect_query([
        ("q", request.q.as_deref()),
        ("sort", request.sort.as_deref()),
        ("fields", request.fields.as_deref()),
    ]);
    let values = fetch_values(&client, &path, &query, request.all)?;

    match output {
        ListOutput::Json => render::print_json(stdout, &values),
        ListOutput::Table => {
            write!(stdout, "{}", render::render_pr_comments_table(&values)).map_err(CliError::from)
        }
    }
}

fn handle_pr_diff<O: Write>(request: &PrDiffRequest, stdout: &mut O) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_numeric_id(request.id.as_deref(), "--id is required")?;
    let client = client_from_profile(request.profile.as_deref())?;
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

fn handle_pr_statuses<O: Write>(
    request: &PrStatusesRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let output = parse_list_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_numeric_id(request.id.as_deref(), "--id is required")?;
    let client = client_from_profile(request.profile.as_deref())?;
    let path = format!("/repositories/{workspace}/{repo}/pullrequests/{id}/statuses");
    let query = collect_query([
        ("q", request.q.as_deref()),
        ("sort", request.sort.as_deref()),
        ("fields", request.fields.as_deref()),
    ]);
    let values = fetch_values(&client, &path, &query, request.all)?;

    match output {
        ListOutput::Json => render::print_json(stdout, &values),
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
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let id = parse_numeric_id(request.id.as_deref(), "--id is required")?;
    let client = client_from_profile(request.profile.as_deref())?;
    let path = format!("/repositories/{workspace}/{repo}/pullrequests/{id}/activity");
    let query = collect_query([
        ("q", request.q.as_deref()),
        ("sort", request.sort.as_deref()),
        ("fields", request.fields.as_deref()),
    ]);
    let values = fetch_values(&client, &path, &query, request.all)?;

    match output {
        ListOutput::Json => render::print_json(stdout, &values),
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

fn handle_pipeline<O: Write>(request: &PipelineRequest, stdout: &mut O) -> Result<(), CliError> {
    match request {
        PipelineRequest::Help => {
            write!(stdout, "{}", render::pipeline_usage()).map_err(CliError::from)
        }
        PipelineRequest::List(request) => handle_pipeline_list(request, stdout),
        PipelineRequest::Run(request) => handle_pipeline_run(request, stdout),
    }
}

fn handle_pipeline_list<O: Write>(
    request: &PipelineListRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let output = parse_list_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let client = client_from_profile(request.profile.as_deref())?;
    let query = collect_query([
        ("sort", request.sort.as_deref()),
        ("fields", request.fields.as_deref()),
    ]);
    let path = format!("/repositories/{workspace}/{repo}/pipelines");
    let values = if request.all {
        client.get_all_values(&path, &query)?
    } else {
        client.get_page(&path, &query)?.0
    };

    match output {
        ListOutput::Json => render::print_json(stdout, &values),
        ListOutput::Table => {
            write!(stdout, "{}", render::render_pipeline_table(&values)).map_err(CliError::from)
        }
    }
}

fn handle_pipeline_run<O: Write>(
    request: &PipelineRunRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let branch = required_string("--branch is required", request.branch.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let body = json!({
        "target": {
            "type": "pipeline_ref_target",
            "ref_type": "branch",
            "ref_name": branch,
        }
    });
    let value = client.request_value(
        Method::POST,
        &format!("/repositories/{workspace}/{repo}/pipelines"),
        &[],
        Some(body),
    )?;
    match output {
        WriteOutput::Json => render::print_json(stdout, &value),
        WriteOutput::Text => {
            writeln!(
                stdout,
                "Triggered pipeline {}",
                render::string_field(&value, &["uuid"]).unwrap_or("-")
            )?;
            writeln!(stdout, "State: {}", render::pipeline_state_label(&value))?;
            if let Some(reference) = render::string_field(&value, &["target", "ref_name"]) {
                if !reference.trim().is_empty() {
                    writeln!(stdout, "Ref: {reference}")?;
                }
            }
            Ok(())
        }
    }
}

fn handle_issue<O: Write>(request: &IssueRequest, stdout: &mut O) -> Result<(), CliError> {
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

fn handle_wiki<O: Write>(request: &WikiRequest, stdout: &mut O) -> Result<(), CliError> {
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

fn resolve_login_token<R: BufRead>(
    request: &AuthLoginRequest,
    stdin: &mut R,
) -> Result<String, CliError> {
    let token = request
        .token
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if let Some(token) = token {
        if token == STDIN_TOKEN_SENTINEL {
            return read_token_from_stdin(stdin);
        }
        return Ok(token.to_string());
    }
    if request.with_token {
        return read_token_from_stdin(stdin);
    }
    if let Ok(token) = std::env::var("BITBUCKET_TOKEN") {
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }
    Err(CliError::InvalidInput(
        "token is required: use --token <value>, --with-token, or BITBUCKET_TOKEN".to_string(),
    ))
}

fn read_token_from_stdin<R: BufRead>(stdin: &mut R) -> Result<String, CliError> {
    let mut buffer = String::new();
    let bytes = stdin.read_line(&mut buffer)?;
    if bytes == 0 {
        return Err(CliError::InvalidInput(
            "no token provided on stdin".to_string(),
        ));
    }
    let token = buffer.trim().to_string();
    if token.is_empty() {
        return Err(CliError::InvalidInput(
            "no token provided on stdin".to_string(),
        ));
    }
    Ok(token)
}

fn client_from_profile(profile_name: Option<&str>) -> Result<Client, CliError> {
    let profile = profile_from_config(profile_name)?;
    Client::from_profile(&profile)
}

fn profile_from_config(profile_name: Option<&str>) -> Result<Profile, CliError> {
    let config = config::load()?;
    let (profile, _) = config.active_profile(profile_name)?;
    if profile.token.trim().is_empty() {
        return Err(CliError::Config(
            "profile has no token configured".to_string(),
        ));
    }
    Ok(profile)
}

fn parse_list_output(value: &str) -> Result<ListOutput, CliError> {
    match value.trim().to_lowercase().as_str() {
        "table" => Ok(ListOutput::Table),
        "json" => Ok(ListOutput::Json),
        other => Err(CliError::UnsupportedOutput(format!(
            "unsupported output format: {other}"
        ))),
    }
}

fn parse_write_output(value: &str) -> Result<WriteOutput, CliError> {
    match value.trim().to_lowercase().as_str() {
        "text" => Ok(WriteOutput::Text),
        "json" => Ok(WriteOutput::Json),
        other => Err(CliError::UnsupportedOutput(format!(
            "unsupported output format: {other}"
        ))),
    }
}

fn normalize_completion_shell(value: &str) -> Result<CompletionShell, CliError> {
    match value.trim().to_lowercase().as_str() {
        "bash" => Ok(CompletionShell::Bash),
        "zsh" => Ok(CompletionShell::Zsh),
        "fish" => Ok(CompletionShell::Fish),
        "powershell" => Ok(CompletionShell::Powershell),
        other => Err(CliError::InvalidInput(format!(
            "unsupported shell: {other}"
        ))),
    }
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

fn parse_numeric_id(value: Option<&str>, missing_message: &str) -> Result<String, CliError> {
    let value = required_string(missing_message, value)?;
    value
        .parse::<u64>()
        .map(|_| value.to_string())
        .map_err(|_| CliError::InvalidInput(format!("--id must be a number: {value}")))
}

fn collect_query<const N: usize>(pairs: [(&str, Option<&str>); N]) -> Vec<(String, String)> {
    pairs
        .into_iter()
        .filter_map(|(key, value)| {
            optional_trimmed(value).map(|value| (key.to_string(), value.to_string()))
        })
        .collect()
}

fn required_string<'a>(message: &str, value: Option<&'a str>) -> Result<&'a str, CliError> {
    optional_trimmed(value).ok_or_else(|| CliError::InvalidInput(message.to_string()))
}

fn optional_trimmed(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn set_optional_string(target: &mut Value, key: &str, value: Option<&str>) {
    if let Some(value) = optional_trimmed(value) {
        target[key] = Value::String(value.to_string());
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
