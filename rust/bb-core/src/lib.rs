pub mod client;
pub mod config;
pub mod context;
pub mod error;
pub mod render;
pub mod runtime;
pub mod version;

use std::io::{BufRead, Write};

pub use error::CliError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteOutput {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListOutput {
    Table,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    Powershell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrState {
    Open,
    Merged,
    Declined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    MergeCommit,
    Squash,
    FastForward,
}

#[derive(Debug, Clone)]
pub struct AuthLoginRequest {
    pub profile: String,
    pub token: Option<String>,
    pub username: Option<String>,
    pub with_token: bool,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthStatusRequest {
    pub profile: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthLogoutRequest {
    pub profile: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApiRequest {
    pub method: String,
    pub paginate: bool,
    pub profile: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub fields: Option<String>,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RepoListRequest {
    pub workspace: Option<String>,
    pub output: String,
    pub all: bool,
    pub profile: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub fields: Option<String>,
    pub json_fields: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PrListRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub output: String,
    pub all: bool,
    pub profile: Option<String>,
    pub state: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub fields: Option<String>,
    pub json_fields: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PrCreateRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub title: Option<String>,
    pub source: Option<String>,
    pub destination: Option<String>,
    pub description: Option<String>,
    pub close_branch: bool,
    pub profile: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct PrMergeRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub id: Option<String>,
    pub message: Option<String>,
    pub strategy: Option<String>,
    pub close_branch: bool,
    pub profile: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct PrGetRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub id: Option<String>,
    pub profile: Option<String>,
    pub output: String,
    pub fields: Option<String>,
    pub json_fields: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PrUpdateRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub destination: Option<String>,
    pub profile: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct PrApproveRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub id: Option<String>,
    pub profile: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct PrUnapproveRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub id: Option<String>,
    pub profile: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct PrRequestChangesRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub id: Option<String>,
    pub profile: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct PrRemoveRequestChangesRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub id: Option<String>,
    pub profile: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct PrDeclineRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub id: Option<String>,
    pub profile: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct PrCommentRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub id: Option<String>,
    pub content: Option<String>,
    pub profile: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct PrCommentsRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub id: Option<String>,
    pub comment_id: Option<String>,
    pub output: String,
    pub all: bool,
    pub profile: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub fields: Option<String>,
    pub json_fields: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PrDiffRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub id: Option<String>,
    pub profile: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct PrStatusesRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub id: Option<String>,
    pub output: String,
    pub all: bool,
    pub profile: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub fields: Option<String>,
    pub json_fields: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PrActivityRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub id: Option<String>,
    pub output: String,
    pub all: bool,
    pub profile: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub fields: Option<String>,
    pub json_fields: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PipelineListRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub output: String,
    pub all: bool,
    pub profile: Option<String>,
    pub sort: Option<String>,
    pub fields: Option<String>,
    pub json_fields: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PipelineGetRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub uuid: Option<String>,
    pub profile: Option<String>,
    pub output: String,
    pub fields: Option<String>,
    pub json_fields: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PipelineStepsRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub uuid: Option<String>,
    pub output: String,
    pub all: bool,
    pub profile: Option<String>,
    pub sort: Option<String>,
    pub fields: Option<String>,
    pub json_fields: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PipelineLogRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub uuid: Option<String>,
    pub step: Option<String>,
    pub profile: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct PipelineRunRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub branch: Option<String>,
    pub profile: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct IssueListRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub output: String,
    pub all: bool,
    pub profile: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub fields: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IssueCreateRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub state: Option<String>,
    pub kind: Option<String>,
    pub priority: Option<String>,
    pub profile: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct IssueUpdateRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub id: Option<u64>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub state: Option<String>,
    pub kind: Option<String>,
    pub priority: Option<String>,
    pub profile: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct WikiListRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub profile: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct WikiGetRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub page: Option<String>,
    pub profile: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct WikiPutRequest {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub page: Option<String>,
    pub content: Option<String>,
    pub file: Option<String>,
    pub message: Option<String>,
    pub profile: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub enum AuthRequest {
    Help,
    Login(AuthLoginRequest),
    Status(AuthStatusRequest),
    Logout(AuthLogoutRequest),
}

#[derive(Debug, Clone)]
pub enum RepoRequest {
    Help,
    List(RepoListRequest),
}

#[derive(Debug, Clone)]
pub enum PrRequest {
    Help,
    List(PrListRequest),
    Create(PrCreateRequest),
    Merge(PrMergeRequest),
    Get(PrGetRequest),
    Update(PrUpdateRequest),
    Approve(PrApproveRequest),
    Unapprove(PrUnapproveRequest),
    RequestChanges(PrRequestChangesRequest),
    RemoveRequestChanges(PrRemoveRequestChangesRequest),
    Decline(PrDeclineRequest),
    Comment(PrCommentRequest),
    Comments(PrCommentsRequest),
    Diff(PrDiffRequest),
    Statuses(PrStatusesRequest),
    Activity(PrActivityRequest),
}

#[derive(Debug, Clone)]
pub enum PipelineRequest {
    Help,
    List(PipelineListRequest),
    Get(PipelineGetRequest),
    Steps(PipelineStepsRequest),
    Log(PipelineLogRequest),
    Run(PipelineRunRequest),
}

#[derive(Debug, Clone)]
pub enum IssueRequest {
    Help,
    List(IssueListRequest),
    Create(IssueCreateRequest),
    Update(IssueUpdateRequest),
}

#[derive(Debug, Clone)]
pub enum WikiRequest {
    Help,
    List(WikiListRequest),
    Get(WikiGetRequest),
    Put(WikiPutRequest),
}

#[derive(Debug, Clone)]
pub enum Request {
    RootHelp,
    Version,
    Completion(Option<String>),
    Auth(AuthRequest),
    Api(ApiRequest),
    Repo(RepoRequest),
    Pr(PrRequest),
    Pipeline(PipelineRequest),
    Issue(IssueRequest),
    Wiki(WikiRequest),
}

pub fn run<R: BufRead, O: Write, E: Write>(
    request: Request,
    stdin: &mut R,
    stdout: &mut O,
    stderr: &mut E,
    stdout_is_tty: bool,
) -> u8 {
    runtime::run(request, stdin, stdout, stderr, stdout_is_tty)
}
