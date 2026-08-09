use std::io::Write;

use crate::context;
use crate::error::CliError;
use crate::render;
use crate::{ListOutput, RepoListRequest, RepoRequest};

use super::support::{
    client_from_profile, collect_query, parse_json_fields, parse_list_output, print_json_list,
};

const REPO_LIST_JSON_FIELDS: &[&str] = &[
    "description",
    "full_name",
    "is_private",
    "links",
    "mainbranch",
    "name",
    "project",
    "scm",
    "slug",
    "updated_on",
    "uuid",
];

pub(super) fn handle_repo<O: Write>(request: &RepoRequest, stdout: &mut O) -> Result<(), CliError> {
    match request {
        RepoRequest::Help => write!(stdout, "{}", render::repo_usage()).map_err(CliError::from),
        RepoRequest::List(request) => handle_repo_list(request, stdout),
    }
}

fn handle_repo_list<O: Write>(request: &RepoListRequest, stdout: &mut O) -> Result<(), CliError> {
    let output = parse_list_output(&request.output)?;
    let json_fields = parse_json_fields(
        request.json_fields.as_deref(),
        output == ListOutput::Json,
        "bb repo list",
        REPO_LIST_JSON_FIELDS,
    )?;
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
        ListOutput::Json => print_json_list(stdout, &values, json_fields.as_deref()),
        ListOutput::Table => {
            write!(stdout, "{}", render::render_repo_table(&values)).map_err(CliError::from)
        }
    }
}
