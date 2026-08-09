use std::io::Write;

use reqwest::Method;
use serde_json::{Value, json};

use crate::client::Client;
use crate::context;
use crate::error::CliError;
use crate::render;
use crate::{
    ListOutput, PipelineGetRequest, PipelineListRequest, PipelineLogRequest, PipelineRequest,
    PipelineRunRequest, PipelineStepsRequest, WriteOutput,
};

use super::support::{
    client_from_profile, collect_query, parse_json_fields, parse_list_output, parse_write_output,
    print_json_list, print_json_object, required_string,
};

enum PipelineSelector {
    Uuid(String, String),
    Build(u64),
}

const PIPELINE_SELECTOR_NAME: &str = "pipeline selector";

const PIPELINE_JSON_FIELDS: &[&str] = &[
    "build_number",
    "completed_on",
    "created_on",
    "creator",
    "links",
    "state",
    "target",
    "trigger",
    "uuid",
];

const PIPELINE_STEPS_JSON_FIELDS: &[&str] = &[
    "completed_on",
    "image",
    "links",
    "name",
    "run_number",
    "script_commands",
    "setup_commands",
    "started_on",
    "state",
    "step",
    "trigger",
    "uuid",
];

pub(super) fn handle_pipeline<O: Write>(
    request: &PipelineRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    match request {
        PipelineRequest::Help => {
            write!(stdout, "{}", render::pipeline_usage()).map_err(CliError::from)
        }
        PipelineRequest::List(request) => handle_pipeline_list(request, stdout),
        PipelineRequest::Get(request) => handle_pipeline_get(request, stdout),
        PipelineRequest::Steps(request) => handle_pipeline_steps(request, stdout),
        PipelineRequest::Log(request) => handle_pipeline_log(request, stdout),
        PipelineRequest::Run(request) => handle_pipeline_run(request, stdout),
    }
}

fn handle_pipeline_list<O: Write>(
    request: &PipelineListRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let output = parse_list_output(&request.output)?;
    let json_fields = parse_json_fields(
        request.json_fields.as_deref(),
        output == ListOutput::Json,
        "bb pipeline list",
        PIPELINE_JSON_FIELDS,
    )?;
    let branch = request
        .branch
        .as_deref()
        .map(|branch| required_string("--branch must not be empty", Some(branch)))
        .transpose()?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let client = client_from_profile(request.profile.as_deref())?;
    let query = collect_query([
        ("target.branch", branch),
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
        ListOutput::Json => print_json_list(stdout, &values, json_fields.as_deref()),
        ListOutput::Table => {
            write!(stdout, "{}", render::render_pipeline_table(&values)).map_err(CliError::from)
        }
    }
}

fn handle_pipeline_get<O: Write>(
    request: &PipelineGetRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let json_fields = parse_json_fields(
        request.json_fields.as_deref(),
        output == WriteOutput::Json,
        "bb pipeline get",
        PIPELINE_JSON_FIELDS,
    )?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let selector = validate_pipeline_selector(
        request.uuid.as_deref(),
        request.build.as_deref(),
        request.positional_selector.as_deref(),
    )?;
    let client = client_from_profile(request.profile.as_deref())?;
    let (_, pipeline_uuid) = resolve_pipeline_selector(&client, &workspace, &repo, &selector)?;
    let query = collect_query([("fields", request.fields.as_deref())]);
    let value = client.request_value(
        Method::GET,
        &format!("/repositories/{workspace}/{repo}/pipelines/{pipeline_uuid}"),
        &query,
        None,
    )?;

    match output {
        WriteOutput::Json => print_json_object(stdout, &value, json_fields.as_deref()),
        WriteOutput::Text => write_pipeline_summary(stdout, "Pipeline", &value),
    }
}

fn handle_pipeline_steps<O: Write>(
    request: &PipelineStepsRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let output = parse_list_output(&request.output)?;
    let json_fields = parse_json_fields(
        request.json_fields.as_deref(),
        output == ListOutput::Json,
        "bb pipeline steps",
        PIPELINE_STEPS_JSON_FIELDS,
    )?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let selector = validate_pipeline_selector(
        request.uuid.as_deref(),
        request.build.as_deref(),
        request.positional_selector.as_deref(),
    )?;
    let client = client_from_profile(request.profile.as_deref())?;
    let (_, pipeline_uuid) = resolve_pipeline_selector(&client, &workspace, &repo, &selector)?;
    let query = collect_query([
        ("sort", request.sort.as_deref()),
        ("fields", request.fields.as_deref()),
    ]);
    let path = format!("/repositories/{workspace}/{repo}/pipelines/{pipeline_uuid}/steps");
    let values = if request.all {
        client.get_all_values(&path, &query)?
    } else {
        client.get_page(&path, &query)?.0
    };

    match output {
        ListOutput::Json => print_json_list(stdout, &values, json_fields.as_deref()),
        ListOutput::Table => write!(stdout, "{}", render::render_pipeline_steps_table(&values))
            .map_err(CliError::from),
    }
}

fn handle_pipeline_log<O: Write>(
    request: &PipelineLogRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let output = parse_write_output(&request.output)?;
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let selector = validate_pipeline_selector(
        request.uuid.as_deref(),
        request.build.as_deref(),
        request.positional_selector.as_deref(),
    )?;
    let (step_display_uuid, step_uuid) = normalize_uuid_arg("--step", request.step.as_deref())?;
    let client = client_from_profile(request.profile.as_deref())?;
    let (pipeline_display_uuid, pipeline_uuid) =
        resolve_pipeline_selector(&client, &workspace, &repo, &selector)?;
    let log = client.request_text(
        Method::GET,
        &format!(
            "/repositories/{workspace}/{repo}/pipelines/{pipeline_uuid}/steps/{step_uuid}/log"
        ),
        &[],
    )?;

    match output {
        WriteOutput::Text => write!(stdout, "{log}").map_err(CliError::from),
        WriteOutput::Json => render::print_json(
            stdout,
            &json!({
                "pipeline_uuid": pipeline_display_uuid,
                "step_uuid": step_display_uuid,
                "log": log,
            }),
        ),
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
        WriteOutput::Text => write_pipeline_summary(stdout, "Triggered pipeline", &value),
    }
}

fn write_pipeline_summary<O: Write>(
    stdout: &mut O,
    heading: &str,
    value: &Value,
) -> Result<(), CliError> {
    writeln!(
        stdout,
        "{heading} {}",
        render::string_field(value, &["uuid"]).unwrap_or("-")
    )?;
    writeln!(stdout, "State: {}", render::pipeline_state_label(value))?;
    if let Some(reference) = render::string_field(value, &["target", "ref_name"]) {
        if !reference.trim().is_empty() {
            writeln!(stdout, "Ref: {reference}")?;
        }
    }
    if let Some(build_number) = render::int_field(value, &["build_number"]) {
        if build_number > 0 {
            writeln!(stdout, "Build: {build_number}")?;
        }
    }
    if let Some(url) = render::string_field(value, &["links", "html", "href"]) {
        if !url.trim().is_empty() {
            writeln!(stdout, "URL: {url}")?;
        }
    }
    Ok(())
}
fn normalize_uuid_arg(flag_name: &str, value: Option<&str>) -> Result<(String, String), CliError> {
    let value = required_string(&format!("{flag_name} is required"), value)?;
    let trimmed = value.trim();
    let lowercase = trimmed.to_ascii_lowercase();
    let has_encoded_open = lowercase.starts_with("%7b");
    let has_encoded_close = lowercase.ends_with("%7d");
    if has_encoded_open ^ has_encoded_close {
        return Err(CliError::InvalidInput(format!(
            "{flag_name} must be a Bitbucket UUID"
        )));
    }
    if has_encoded_open && has_encoded_close && trimmed.len() > 6 {
        let inner = &trimmed[3..trimmed.len() - 3];
        validate_uuid_arg(flag_name, inner)?;
        return Ok((format!("{{{inner}}}"), trimmed.to_string()));
    }

    let has_open_brace = trimmed.starts_with('{');
    let has_close_brace = trimmed.ends_with('}');
    if has_open_brace ^ has_close_brace {
        return Err(CliError::InvalidInput(format!(
            "{flag_name} must be a Bitbucket UUID"
        )));
    }

    let inner = trimmed.strip_prefix('{').unwrap_or(trimmed);
    let inner = inner.strip_suffix('}').unwrap_or(inner);
    validate_uuid_arg(flag_name, inner)?;
    Ok((format!("{{{inner}}}"), format!("%7B{inner}%7D")))
}

fn validate_pipeline_selector(
    uuid: Option<&str>,
    build: Option<&str>,
    positional_selector: Option<&str>,
) -> Result<PipelineSelector, CliError> {
    match (uuid, build, positional_selector) {
        (Some(uuid), None, None) => {
            let (display_uuid, encoded_uuid) = normalize_uuid_arg("--uuid", Some(uuid))?;
            Ok(PipelineSelector::Uuid(display_uuid, encoded_uuid))
        }
        (None, Some(build), None) => Ok(PipelineSelector::Build(parse_pipeline_build_arg(
            "--build", build,
        )?)),
        (None, None, Some(selector)) => {
            let selector = selector.trim();
            if selector.starts_with('{') {
                let (display_uuid, encoded_uuid) =
                    normalize_uuid_arg(PIPELINE_SELECTOR_NAME, Some(selector))?;
                Ok(PipelineSelector::Uuid(display_uuid, encoded_uuid))
            } else {
                Ok(PipelineSelector::Build(parse_pipeline_build_arg(
                    PIPELINE_SELECTOR_NAME,
                    selector,
                )?))
            }
        }
        (None, None, None) => Err(CliError::InvalidInput(
            "pipeline identifier is required: pass a build number or {uuid} positionally, or --uuid/--build"
                .to_string(),
        )),
        _ => Err(CliError::InvalidInput(
            "pass exactly one of the positional selector, --uuid, or --build".to_string(),
        )),
    }
}

fn resolve_pipeline_selector(
    client: &Client,
    workspace: &str,
    repo: &str,
    selector: &PipelineSelector,
) -> Result<(String, String), CliError> {
    match selector {
        PipelineSelector::Uuid(display_uuid, encoded_uuid) => {
            Ok((display_uuid.clone(), encoded_uuid.clone()))
        }
        PipelineSelector::Build(build) => {
            resolve_pipeline_build_lookup(client, workspace, repo, *build)
        }
    }
}

fn parse_pipeline_build_arg(arg_name: &str, build: &str) -> Result<u64, CliError> {
    let build = required_string(&format!("{arg_name} is required"), Some(build))?;
    let build = build
        .parse::<u64>()
        .map_err(|_| CliError::InvalidInput(format!("{arg_name} must be a positive integer")))?;
    if build == 0 {
        return Err(CliError::InvalidInput(format!(
            "{arg_name} must be a positive integer"
        )));
    }
    Ok(build)
}

fn resolve_pipeline_build_lookup(
    client: &Client,
    workspace: &str,
    repo: &str,
    build: u64,
) -> Result<(String, String), CliError> {
    let value = client.request_value(
        Method::GET,
        &format!("/repositories/{workspace}/{repo}/pipelines/{build}"),
        &[],
        None,
    )?;
    let returned_build = value
        .get("build_number")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            CliError::Internal("pipeline lookup response missing build_number".to_string())
        })?;
    if returned_build != build {
        return Err(CliError::Internal(format!(
            "pipeline lookup returned build {returned_build}, expected {build}"
        )));
    }
    let uuid = value
        .get("uuid")
        .and_then(Value::as_str)
        .ok_or_else(|| CliError::Internal("pipeline lookup response missing uuid".to_string()))?;
    normalize_uuid_arg("--build", Some(uuid))
}

fn validate_uuid_arg(flag_name: &str, value: &str) -> Result<(), CliError> {
    let value = value.trim();
    if value.is_empty()
        || value
            .chars()
            .any(|ch| ch.is_whitespace() || matches!(ch, '/' | '?' | '#' | '{' | '}'))
    {
        return Err(CliError::InvalidInput(format!(
            "{flag_name} must be a Bitbucket UUID"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_pipeline_selector_reads_positional_build_number() {
        let selector = validate_pipeline_selector(None, None, Some("14588"))
            .expect("positional build number should be accepted");

        assert!(matches!(selector, PipelineSelector::Build(14588)));
    }

    #[test]
    fn validate_pipeline_selector_reads_positional_uuid() {
        let selector = validate_pipeline_selector(None, None, Some("{pipe-123}"))
            .expect("positional uuid should be accepted");

        let PipelineSelector::Uuid(display_uuid, encoded_uuid) = selector else {
            panic!("expected uuid selector");
        };
        assert_eq!(display_uuid, "{pipe-123}");
        assert_eq!(encoded_uuid, "%7Bpipe-123%7D");
    }

    #[test]
    fn validate_pipeline_selector_rejects_non_numeric_positional() {
        let Err(error) = validate_pipeline_selector(None, None, Some("abc")) else {
            panic!("positional selector should be rejected");
        };

        assert_eq!(error.code(), "invalid_input");
        assert_eq!(
            error.message(),
            "pipeline selector must be a positive integer"
        );
    }

    #[test]
    fn validate_pipeline_selector_rejects_zero_positional() {
        let Err(error) = validate_pipeline_selector(None, None, Some("0")) else {
            panic!("positional selector should be rejected");
        };

        assert_eq!(error.code(), "invalid_input");
        assert_eq!(
            error.message(),
            "pipeline selector must be a positive integer"
        );
    }

    #[test]
    fn validate_pipeline_selector_rejects_positional_with_flags() {
        for (uuid, build) in [(Some("{pipe-123}"), None), (None, Some("17"))] {
            let Err(error) = validate_pipeline_selector(uuid, build, Some("17")) else {
                panic!("positional selector with a flag should be rejected");
            };

            assert_eq!(error.code(), "invalid_input");
            assert_eq!(
                error.message(),
                "pass exactly one of the positional selector, --uuid, or --build"
            );
        }
    }

    #[test]
    fn validate_pipeline_selector_requires_an_identifier() {
        let Err(error) = validate_pipeline_selector(None, None, None) else {
            panic!("missing identifier should be rejected");
        };

        assert_eq!(error.code(), "invalid_input");
        assert_eq!(
            error.message(),
            "pipeline identifier is required: pass a build number or {uuid} positionally, or --uuid/--build"
        );
    }
}
