use bb_core::{
    AuthRequest, PipelineRequest, PrDeclineRequest, PrRequest, PrRequestChangesRequest,
    PrStatusesRequest, PrUpdateRequest, Request,
};

use super::*;

#[test]
fn global_repository_selector_populates_repo_target() {
    let request = parse_from(["bb", "pr", "get", "42", "-R", "acme/widgets"])
        .expect("repository selector should parse");
    let Request::Pr(PrRequest::Get(request)) = request else {
        panic!("expected pr get");
    };
    assert_eq!(request.workspace.as_deref(), Some("acme"));
    assert_eq!(request.repo.as_deref(), Some("widgets"));
}

#[test]
fn global_repository_selector_rejects_invalid_or_conflicting_inputs() {
    assert!(parse_from(["bb", "pr", "get", "42", "-R", "invalid"]).is_err());
    assert!(
        parse_from([
            "bb",
            "pr",
            "get",
            "42",
            "-R",
            "acme/widgets",
            "--workspace",
            "other",
        ])
        .is_err()
    );
    assert!(parse_from(["bb", "api", "-R", "acme/widgets", "user"]).is_err());
}

#[test]
fn bare_token_is_normalized_to_stdin_sentinel() {
    let request = parse_from(["bb", "auth", "login", "--token"]).expect("parse should succeed");
    let Request::Auth(AuthRequest::Login(request)) = request else {
        panic!("expected auth login");
    };
    assert_eq!(request.token.as_deref(), Some(STDIN_TOKEN_SENTINEL));
}

#[test]
fn root_without_command_maps_to_root_help() {
    let request = parse_from(["bb"]).expect("parse should succeed");
    assert!(matches!(request, Request::RootHelp));
}

#[test]
fn root_help_flag_maps_to_root_help() {
    let request = parse_from(["bb", "--help"]).expect("parse should succeed");
    assert!(matches!(request, Request::RootHelp));
}

#[test]
fn version_flag_maps_to_version_request() {
    let request = parse_from(["bb", "--version"]).expect("parse should succeed");
    assert!(matches!(request, Request::Version));
}

#[test]
fn api_maps_input_flag() {
    let request = parse_from([
        "bb",
        "api",
        "--method",
        "POST",
        "--input",
        "-",
        "repositories/acme/widgets/pullrequests/42/comments",
    ])
    .expect("parse should succeed");
    let Request::Api(request) = request else {
        panic!("expected api request");
    };
    assert_eq!(request.method, "POST");
    assert_eq!(request.input.as_deref(), Some("-"));
    assert_eq!(
        request.endpoint.as_deref(),
        Some("repositories/acme/widgets/pullrequests/42/comments")
    );
}

#[test]
fn pr_list_maps_positive_limit_and_rejects_conflicts() {
    let request =
        parse_from(["bb", "pr", "list", "--limit", "25"]).expect("positive limit should parse");
    let Request::Pr(PrRequest::List(request)) = request else {
        panic!("expected pr list");
    };
    assert_eq!(request.limit, Some(25));

    assert!(parse_from(["bb", "pr", "list", "--limit", "0"]).is_err());
    assert!(parse_from(["bb", "pr", "list", "--limit", "25", "--all"]).is_err());
}

#[test]
fn pr_get_maps_to_get_request() {
    let request = parse_from(["bb", "pr", "get", "--id", "42"]).expect("parse should succeed");
    let Request::Pr(PrRequest::Get(request)) = request else {
        panic!("expected pr get");
    };
    assert_eq!(request.id.as_deref(), Some("42"));
    assert_eq!(request.output, "text");
}

#[test]
fn pr_get_maps_positional_id_to_get_request() {
    let request = parse_from(["bb", "pr", "get", "42"]).expect("parse should succeed");
    let Request::Pr(PrRequest::Get(request)) = request else {
        panic!("expected pr get");
    };
    assert_eq!(request.id.as_deref(), Some("42"));
}

#[test]
fn pr_view_alias_maps_to_get_request() {
    let request = parse_from(["bb", "pr", "view", "42"]).expect("parse should succeed");
    let Request::Pr(PrRequest::Get(request)) = request else {
        panic!("expected pr get");
    };
    assert_eq!(request.id.as_deref(), Some("42"));
}

#[test]
fn pr_get_maps_json_fields() {
    let request = parse_from([
        "bb",
        "pr",
        "get",
        "42",
        "--output",
        "json",
        "--json-fields",
        "id,title",
    ])
    .expect("parse should succeed");
    let Request::Pr(PrRequest::Get(request)) = request else {
        panic!("expected pr get");
    };
    assert_eq!(request.json_fields.as_deref(), Some("id,title"));
}

#[test]
fn pr_edit_alias_maps_to_update_request() {
    let request = parse_from(["bb", "pr", "edit", "--id", "42"]).expect("parse should succeed");
    assert!(matches!(
        request,
        Request::Pr(PrRequest::Update(PrUpdateRequest { id: Some(id), .. })) if id == "42"
    ));
}

#[test]
fn pr_request_changes_maps_to_request_changes_request() {
    let request =
        parse_from(["bb", "pr", "request-changes", "--id", "42"]).expect("parse should succeed");
    assert!(matches!(
        request,
        Request::Pr(PrRequest::RequestChanges(PrRequestChangesRequest {
            id: Some(id),
            ..
        })) if id == "42"
    ));
}

#[test]
fn pr_close_alias_maps_to_decline_request() {
    let request = parse_from(["bb", "pr", "close", "--id", "42"]).expect("parse should succeed");
    assert!(matches!(
        request,
        Request::Pr(PrRequest::Decline(PrDeclineRequest { id: Some(id), .. })) if id == "42"
    ));
}

#[test]
fn pr_checks_alias_maps_to_statuses_request() {
    let request = parse_from(["bb", "pr", "checks", "--id", "42"]).expect("parse should succeed");
    assert!(matches!(
        request,
        Request::Pr(PrRequest::Statuses(PrStatusesRequest { id: Some(id), .. })) if id == "42"
    ));
}

#[test]
fn pr_create_maps_body_alias_to_description() {
    let request = parse_from([
        "bb",
        "pr",
        "create",
        "--title",
        "title",
        "--source",
        "feature",
        "--destination",
        "main",
        "--body",
        "description",
    ])
    .expect("parse should succeed");
    let Request::Pr(PrRequest::Create(request)) = request else {
        panic!("expected pr create");
    };
    assert_eq!(request.description.as_deref(), Some("description"));
}

#[test]
fn pr_comment_maps_content_and_output() {
    let request = parse_from([
        "bb",
        "pr",
        "comment",
        "--id",
        "42",
        "--body",
        "needs changes",
        "--output",
        "json",
    ])
    .expect("parse should succeed");
    let Request::Pr(PrRequest::Comment(request)) = request else {
        panic!("expected pr comment");
    };
    assert_eq!(request.id.as_deref(), Some("42"));
    assert_eq!(request.content.as_deref(), Some("needs changes"));
    assert_eq!(request.output, "json");
}

#[test]
fn pr_comment_update_maps_comment_id_content_and_output() {
    let request = parse_from([
        "bb",
        "pr",
        "comment-update",
        "42",
        "--comment-id",
        "7",
        "--content",
        "updated body",
        "--output",
        "json",
    ])
    .expect("parse should succeed");
    let Request::Pr(PrRequest::CommentUpdate(request)) = request else {
        panic!("expected pr comment update");
    };
    assert_eq!(request.id.as_deref(), Some("42"));
    assert_eq!(request.comment_id.as_deref(), Some("7"));
    assert_eq!(request.content.as_deref(), Some("updated body"));
    assert_eq!(request.output, "json");
}

#[test]
fn pr_comments_rejects_positional_and_flag_id_together() {
    let error =
        parse_from(["bb", "pr", "comments", "42", "--id", "43"]).expect_err("parse should fail");
    assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
}

#[test]
fn pr_comments_maps_comment_id() {
    let request = parse_from([
        "bb",
        "pr",
        "comments",
        "42",
        "--comment-id",
        "7",
        "--output",
        "json",
    ])
    .expect("parse should succeed");
    let Request::Pr(PrRequest::Comments(request)) = request else {
        panic!("expected pr comments");
    };
    assert_eq!(request.id.as_deref(), Some("42"));
    assert_eq!(request.comment_id.as_deref(), Some("7"));
    assert_eq!(request.output, "json");
}

#[test]
fn pipeline_get_maps_uuid_and_output() {
    let request = parse_from([
        "bb",
        "pipeline",
        "get",
        "--uuid",
        "{1234}",
        "--output",
        "json",
        "--json-fields",
        "uuid,state",
    ])
    .expect("parse should succeed");
    let Request::Pipeline(PipelineRequest::Get(request)) = request else {
        panic!("expected pipeline get");
    };
    assert_eq!(request.uuid.as_deref(), Some("{1234}"));
    assert_eq!(request.output, "json");
    assert_eq!(request.json_fields.as_deref(), Some("uuid,state"));
}

#[test]
fn pipeline_get_maps_build_and_output() {
    let request = parse_from(["bb", "pipeline", "get", "--build", "17", "--output", "json"])
        .expect("parse should succeed");
    let Request::Pipeline(PipelineRequest::Get(request)) = request else {
        panic!("expected pipeline get");
    };
    assert_eq!(request.build.as_deref(), Some("17"));
    assert_eq!(request.output, "json");
}

#[test]
fn pipeline_log_maps_step_and_output() {
    let request = parse_from([
        "bb", "pipeline", "log", "--uuid", "{pipe}", "--step", "{step}",
    ])
    .expect("parse should succeed");
    let Request::Pipeline(PipelineRequest::Log(request)) = request else {
        panic!("expected pipeline log");
    };
    assert_eq!(request.uuid.as_deref(), Some("{pipe}"));
    assert_eq!(request.step.as_deref(), Some("{step}"));
    assert_eq!(request.output, "text");
}

#[test]
fn pipeline_get_maps_positional_selector() {
    let request = parse_from(["bb", "pipeline", "get", "14588", "--output", "json"])
        .expect("parse should succeed");
    let Request::Pipeline(PipelineRequest::Get(request)) = request else {
        panic!("expected pipeline get");
    };
    assert_eq!(request.positional_selector.as_deref(), Some("14588"));
    assert_eq!(request.uuid, None);
    assert_eq!(request.build, None);
}

#[test]
fn pipeline_steps_maps_positional_selector() {
    let request = parse_from(["bb", "pipeline", "steps", "14588", "--output", "json"])
        .expect("parse should succeed");
    let Request::Pipeline(PipelineRequest::Steps(request)) = request else {
        panic!("expected pipeline steps");
    };
    assert_eq!(request.positional_selector.as_deref(), Some("14588"));
}

#[test]
fn pipeline_log_maps_positional_selector() {
    let request = parse_from(["bb", "pipeline", "log", "14588", "--step", "{step}"])
        .expect("parse should succeed");
    let Request::Pipeline(PipelineRequest::Log(request)) = request else {
        panic!("expected pipeline log");
    };
    assert_eq!(request.positional_selector.as_deref(), Some("14588"));
    assert_eq!(request.step.as_deref(), Some("{step}"));
}

#[test]
fn pipeline_get_rejects_uuid_and_build_together() {
    let error = parse_from(["bb", "pipeline", "get", "--uuid", "{1234}", "--build", "17"])
        .expect_err("parse should fail");
    assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
}
