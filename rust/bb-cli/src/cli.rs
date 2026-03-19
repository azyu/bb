use std::ffi::OsString;

use bb_core::{
    ApiRequest, AuthLoginRequest, AuthLogoutRequest, AuthRequest, AuthStatusRequest,
    IssueCreateRequest, IssueListRequest, IssueRequest, IssueUpdateRequest, PipelineGetRequest,
    PipelineListRequest, PipelineLogRequest, PipelineRequest, PipelineRunRequest,
    PipelineStepsRequest, PrActivityRequest, PrApproveRequest, PrCommentRequest, PrCommentsRequest,
    PrCreateRequest, PrDeclineRequest, PrDiffRequest, PrGetRequest, PrListRequest, PrMergeRequest,
    PrRemoveRequestChangesRequest, PrRequest, PrRequestChangesRequest, PrStatusesRequest,
    PrUnapproveRequest, PrUpdateRequest, RepoListRequest, RepoRequest, Request, WikiGetRequest,
    WikiListRequest, WikiPutRequest, WikiRequest,
};
use clap::{Args, Parser, Subcommand};

const STDIN_TOKEN_SENTINEL: &str = bb_core::runtime::STDIN_TOKEN_SENTINEL;

#[derive(Debug, Parser)]
#[command(
    name = "bb",
    disable_version_flag = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    #[arg(short = 'v', long = "version", global = true)]
    pub version: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Auth {
        #[command(subcommand)]
        command: Option<AuthCommands>,
    },
    Api(ApiArgs),
    Repo {
        #[command(subcommand)]
        command: Option<RepoCommands>,
    },
    Pr {
        #[command(subcommand)]
        command: Option<PrCommands>,
    },
    Pipeline {
        #[command(subcommand)]
        command: Option<PipelineCommands>,
    },
    Issue {
        #[command(subcommand)]
        command: Option<IssueCommands>,
    },
    Wiki {
        #[command(subcommand)]
        command: Option<WikiCommands>,
    },
    Completion(CompletionArgs),
    Version,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommands {
    Login(AuthLoginArgs),
    Status(AuthStatusArgs),
    Logout(AuthLogoutArgs),
}

#[derive(Debug, Subcommand)]
pub enum RepoCommands {
    List(RepoListArgs),
}

#[derive(Debug, Subcommand)]
pub enum PrCommands {
    List(PrListArgs),
    Create(PrCreateArgs),
    Merge(PrMergeArgs),
    #[command(alias = "view")]
    Get(PrGetArgs),
    #[command(alias = "edit")]
    Update(PrUpdateArgs),
    Approve(PrApproveArgs),
    Unapprove(PrUnapproveArgs),
    RequestChanges(PrRequestChangesArgs),
    RemoveRequestChanges(PrRemoveRequestChangesArgs),
    #[command(alias = "close")]
    Decline(PrDeclineArgs),
    Comment(PrCommentArgs),
    Comments(PrCommentsArgs),
    Diff(PrDiffArgs),
    #[command(alias = "checks")]
    Statuses(PrStatusesArgs),
    Activity(PrActivityArgs),
}

#[derive(Debug, Subcommand)]
pub enum PipelineCommands {
    List(PipelineListArgs),
    Get(PipelineGetArgs),
    Steps(PipelineStepsArgs),
    Log(PipelineLogArgs),
    Run(PipelineRunArgs),
}

#[derive(Debug, Subcommand)]
pub enum IssueCommands {
    List(IssueListArgs),
    Create(IssueCreateArgs),
    Update(IssueUpdateArgs),
}

#[derive(Debug, Subcommand)]
pub enum WikiCommands {
    List(WikiListArgs),
    Get(WikiGetArgs),
    Put(WikiPutArgs),
}

#[derive(Debug, Args)]
pub struct CompletionArgs {
    pub shell: Option<String>,
}

#[derive(Debug, Args)]
pub struct AuthLoginArgs {
    #[arg(long, default_value = "default")]
    pub profile: String,
    #[arg(long)]
    pub token: Option<String>,
    #[arg(long)]
    pub username: Option<String>,
    #[arg(long)]
    pub with_token: bool,
    #[arg(long)]
    pub base_url: Option<String>,
}

#[derive(Debug, Args)]
pub struct AuthStatusArgs {
    #[arg(long)]
    pub profile: Option<String>,
}

#[derive(Debug, Args)]
pub struct AuthLogoutArgs {
    #[arg(long)]
    pub profile: Option<String>,
}

#[derive(Debug, Args)]
pub struct ApiArgs {
    #[arg(long, default_value = "GET")]
    pub method: String,
    #[arg(long)]
    pub paginate: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub q: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub fields: Option<String>,
    pub endpoint: Option<String>,
}

#[derive(Debug, Args)]
pub struct RepoListArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long, default_value = "table")]
    pub output: String,
    #[arg(long)]
    pub all: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub q: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long)]
    pub json_fields: Option<String>,
}

#[derive(Debug, Args)]
pub struct PrListArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value = "table")]
    pub output: String,
    #[arg(long)]
    pub all: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub state: Option<String>,
    #[arg(long)]
    pub q: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long)]
    pub json_fields: Option<String>,
}

#[derive(Debug, Args)]
pub struct PrCreateArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub source: Option<String>,
    #[arg(long)]
    pub destination: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub close_branch: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct PrMergeArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub pr_id: Option<String>,
    #[arg(long)]
    pub message: Option<String>,
    #[arg(long)]
    pub strategy: Option<String>,
    #[arg(long)]
    pub close_branch: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct PrGetArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub pr_id: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long)]
    pub json_fields: Option<String>,
}

#[derive(Debug, Args)]
pub struct PrUpdateArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub pr_id: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub source: Option<String>,
    #[arg(long)]
    pub destination: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct PrApproveArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub pr_id: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct PrUnapproveArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub pr_id: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct PrRequestChangesArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub pr_id: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct PrRemoveRequestChangesArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub pr_id: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct PrDeclineArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub pr_id: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct PrCommentArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub pr_id: Option<String>,
    #[arg(long)]
    pub content: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct PrCommentsArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub pr_id: Option<String>,
    #[arg(long, value_name = "COMMENT_ID")]
    pub comment_id: Option<String>,
    #[arg(long, default_value = "table")]
    pub output: String,
    #[arg(long)]
    pub all: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub q: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long)]
    pub json_fields: Option<String>,
}

#[derive(Debug, Args)]
pub struct PrDiffArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub pr_id: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct PrStatusesArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub pr_id: Option<String>,
    #[arg(long, default_value = "table")]
    pub output: String,
    #[arg(long)]
    pub all: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub q: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long)]
    pub json_fields: Option<String>,
}

#[derive(Debug, Args)]
pub struct PrActivityArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub pr_id: Option<String>,
    #[arg(long, default_value = "table")]
    pub output: String,
    #[arg(long)]
    pub all: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub q: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long)]
    pub json_fields: Option<String>,
}

#[derive(Debug, Args)]
pub struct PipelineListArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value = "table")]
    pub output: String,
    #[arg(long)]
    pub all: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long)]
    pub json_fields: Option<String>,
}

#[derive(Debug, Args)]
pub struct PipelineGetArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, conflicts_with = "build")]
    pub uuid: Option<String>,
    #[arg(long, conflicts_with = "uuid")]
    pub build: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long)]
    pub json_fields: Option<String>,
}

#[derive(Debug, Args)]
pub struct PipelineStepsArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, conflicts_with = "build")]
    pub uuid: Option<String>,
    #[arg(long, conflicts_with = "uuid")]
    pub build: Option<String>,
    #[arg(long, default_value = "table")]
    pub output: String,
    #[arg(long)]
    pub all: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long)]
    pub json_fields: Option<String>,
}

#[derive(Debug, Args)]
pub struct PipelineLogArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, conflicts_with = "build")]
    pub uuid: Option<String>,
    #[arg(long, conflicts_with = "uuid")]
    pub build: Option<String>,
    #[arg(long)]
    pub step: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct PipelineRunArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub branch: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct IssueListArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value = "table")]
    pub output: String,
    #[arg(long)]
    pub all: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub q: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub fields: Option<String>,
}

#[derive(Debug, Args)]
pub struct IssueCreateArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub content: Option<String>,
    #[arg(long)]
    pub state: Option<String>,
    #[arg(long)]
    pub kind: Option<String>,
    #[arg(long)]
    pub priority: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct IssueUpdateArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub id: Option<u64>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub content: Option<String>,
    #[arg(long)]
    pub state: Option<String>,
    #[arg(long)]
    pub kind: Option<String>,
    #[arg(long)]
    pub priority: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct WikiListArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "table")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct WikiGetArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub page: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct WikiPutArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub page: Option<String>,
    #[arg(long)]
    pub content: Option<String>,
    #[arg(long)]
    pub file: Option<String>,
    #[arg(long)]
    pub message: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

pub fn parse_from<I, T>(args: I) -> Result<Request, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let normalized = normalize_args(args.into_iter().map(Into::into).collect());
    if normalized.len() == 2 {
        let arg = normalized[1].to_string_lossy();
        if arg == "help" || arg == "--help" || arg == "-h" {
            return Ok(Request::RootHelp);
        }
    }

    let cli = Cli::try_parse_from(normalized)?;
    Ok(map_request(cli))
}

fn normalize_args(args: Vec<OsString>) -> Vec<OsString> {
    let mut out = Vec::with_capacity(args.len());
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if index > 0 && arg.to_string_lossy() == "--token" {
            let next_is_value = args
                .get(index + 1)
                .map(|value| !value.to_string_lossy().starts_with('-'))
                .unwrap_or(false);
            if next_is_value {
                out.push(arg.clone());
            } else {
                out.push(OsString::from(format!("--token={STDIN_TOKEN_SENTINEL}")));
            }
            index += 1;
            continue;
        }
        out.push(arg.clone());
        index += 1;
    }
    out
}

fn map_request(cli: Cli) -> Request {
    if cli.version {
        return Request::Version;
    }

    match cli.command {
        None => Request::RootHelp,
        Some(Commands::Version) => Request::Version,
        Some(Commands::Completion(args)) => Request::Completion(args.shell),
        Some(Commands::Auth { command }) => Request::Auth(match command {
            None => AuthRequest::Help,
            Some(AuthCommands::Login(args)) => AuthRequest::Login(AuthLoginRequest {
                profile: args.profile,
                token: args.token,
                username: args.username,
                with_token: args.with_token,
                base_url: args.base_url,
            }),
            Some(AuthCommands::Status(args)) => AuthRequest::Status(AuthStatusRequest {
                profile: args.profile,
            }),
            Some(AuthCommands::Logout(args)) => AuthRequest::Logout(AuthLogoutRequest {
                profile: args.profile,
            }),
        }),
        Some(Commands::Api(args)) => Request::Api(ApiRequest {
            method: args.method,
            paginate: args.paginate,
            profile: args.profile,
            q: args.q,
            sort: args.sort,
            fields: args.fields,
            endpoint: args.endpoint,
        }),
        Some(Commands::Repo { command }) => Request::Repo(match command {
            None => RepoRequest::Help,
            Some(RepoCommands::List(args)) => RepoRequest::List(RepoListRequest {
                workspace: args.workspace,
                output: args.output,
                all: args.all,
                profile: args.profile,
                q: args.q,
                sort: args.sort,
                fields: args.fields,
                json_fields: args.json_fields,
            }),
        }),
        Some(Commands::Pr { command }) => Request::Pr(match command {
            None => PrRequest::Help,
            Some(PrCommands::List(args)) => PrRequest::List(PrListRequest {
                workspace: args.workspace,
                repo: args.repo,
                output: args.output,
                all: args.all,
                profile: args.profile,
                state: args.state,
                q: args.q,
                sort: args.sort,
                fields: args.fields,
                json_fields: args.json_fields,
            }),
            Some(PrCommands::Create(args)) => PrRequest::Create(PrCreateRequest {
                workspace: args.workspace,
                repo: args.repo,
                title: args.title,
                source: args.source,
                destination: args.destination,
                description: args.description,
                close_branch: args.close_branch,
                profile: args.profile,
                output: args.output,
            }),
            Some(PrCommands::Merge(args)) => PrRequest::Merge(PrMergeRequest {
                workspace: args.workspace,
                repo: args.repo,
                id: resolve_pr_id(args.id, args.pr_id),
                message: args.message,
                strategy: args.strategy,
                close_branch: args.close_branch,
                profile: args.profile,
                output: args.output,
            }),
            Some(PrCommands::Get(args)) => PrRequest::Get(PrGetRequest {
                workspace: args.workspace,
                repo: args.repo,
                id: resolve_pr_id(args.id, args.pr_id),
                profile: args.profile,
                output: args.output,
                fields: args.fields,
                json_fields: args.json_fields,
            }),
            Some(PrCommands::Update(args)) => PrRequest::Update(PrUpdateRequest {
                workspace: args.workspace,
                repo: args.repo,
                id: resolve_pr_id(args.id, args.pr_id),
                title: args.title,
                description: args.description,
                source: args.source,
                destination: args.destination,
                profile: args.profile,
                output: args.output,
            }),
            Some(PrCommands::Approve(args)) => PrRequest::Approve(PrApproveRequest {
                workspace: args.workspace,
                repo: args.repo,
                id: resolve_pr_id(args.id, args.pr_id),
                profile: args.profile,
                output: args.output,
            }),
            Some(PrCommands::Unapprove(args)) => PrRequest::Unapprove(PrUnapproveRequest {
                workspace: args.workspace,
                repo: args.repo,
                id: resolve_pr_id(args.id, args.pr_id),
                profile: args.profile,
                output: args.output,
            }),
            Some(PrCommands::RequestChanges(args)) => {
                PrRequest::RequestChanges(PrRequestChangesRequest {
                    workspace: args.workspace,
                    repo: args.repo,
                    id: resolve_pr_id(args.id, args.pr_id),
                    profile: args.profile,
                    output: args.output,
                })
            }
            Some(PrCommands::RemoveRequestChanges(args)) => {
                PrRequest::RemoveRequestChanges(PrRemoveRequestChangesRequest {
                    workspace: args.workspace,
                    repo: args.repo,
                    id: resolve_pr_id(args.id, args.pr_id),
                    profile: args.profile,
                    output: args.output,
                })
            }
            Some(PrCommands::Decline(args)) => PrRequest::Decline(PrDeclineRequest {
                workspace: args.workspace,
                repo: args.repo,
                id: resolve_pr_id(args.id, args.pr_id),
                profile: args.profile,
                output: args.output,
            }),
            Some(PrCommands::Comment(args)) => PrRequest::Comment(PrCommentRequest {
                workspace: args.workspace,
                repo: args.repo,
                id: resolve_pr_id(args.id, args.pr_id),
                content: args.content,
                profile: args.profile,
                output: args.output,
            }),
            Some(PrCommands::Comments(args)) => PrRequest::Comments(PrCommentsRequest {
                workspace: args.workspace,
                repo: args.repo,
                id: resolve_pr_id(args.id, args.pr_id),
                comment_id: args.comment_id,
                output: args.output,
                all: args.all,
                profile: args.profile,
                q: args.q,
                sort: args.sort,
                fields: args.fields,
                json_fields: args.json_fields,
            }),
            Some(PrCommands::Diff(args)) => PrRequest::Diff(PrDiffRequest {
                workspace: args.workspace,
                repo: args.repo,
                id: resolve_pr_id(args.id, args.pr_id),
                profile: args.profile,
                output: args.output,
            }),
            Some(PrCommands::Statuses(args)) => PrRequest::Statuses(PrStatusesRequest {
                workspace: args.workspace,
                repo: args.repo,
                id: resolve_pr_id(args.id, args.pr_id),
                output: args.output,
                all: args.all,
                profile: args.profile,
                q: args.q,
                sort: args.sort,
                fields: args.fields,
                json_fields: args.json_fields,
            }),
            Some(PrCommands::Activity(args)) => PrRequest::Activity(PrActivityRequest {
                workspace: args.workspace,
                repo: args.repo,
                id: resolve_pr_id(args.id, args.pr_id),
                output: args.output,
                all: args.all,
                profile: args.profile,
                q: args.q,
                sort: args.sort,
                fields: args.fields,
                json_fields: args.json_fields,
            }),
        }),
        Some(Commands::Pipeline { command }) => Request::Pipeline(match command {
            None => PipelineRequest::Help,
            Some(PipelineCommands::List(args)) => PipelineRequest::List(PipelineListRequest {
                workspace: args.workspace,
                repo: args.repo,
                output: args.output,
                all: args.all,
                profile: args.profile,
                sort: args.sort,
                fields: args.fields,
                json_fields: args.json_fields,
            }),
            Some(PipelineCommands::Get(args)) => PipelineRequest::Get(PipelineGetRequest {
                workspace: args.workspace,
                repo: args.repo,
                uuid: args.uuid,
                build: args.build,
                profile: args.profile,
                output: args.output,
                fields: args.fields,
                json_fields: args.json_fields,
            }),
            Some(PipelineCommands::Steps(args)) => PipelineRequest::Steps(PipelineStepsRequest {
                workspace: args.workspace,
                repo: args.repo,
                uuid: args.uuid,
                build: args.build,
                output: args.output,
                all: args.all,
                profile: args.profile,
                sort: args.sort,
                fields: args.fields,
                json_fields: args.json_fields,
            }),
            Some(PipelineCommands::Log(args)) => PipelineRequest::Log(PipelineLogRequest {
                workspace: args.workspace,
                repo: args.repo,
                uuid: args.uuid,
                build: args.build,
                step: args.step,
                profile: args.profile,
                output: args.output,
            }),
            Some(PipelineCommands::Run(args)) => PipelineRequest::Run(PipelineRunRequest {
                workspace: args.workspace,
                repo: args.repo,
                branch: args.branch,
                profile: args.profile,
                output: args.output,
            }),
        }),
        Some(Commands::Issue { command }) => Request::Issue(match command {
            None => IssueRequest::Help,
            Some(IssueCommands::List(args)) => IssueRequest::List(IssueListRequest {
                workspace: args.workspace,
                repo: args.repo,
                output: args.output,
                all: args.all,
                profile: args.profile,
                q: args.q,
                sort: args.sort,
                fields: args.fields,
            }),
            Some(IssueCommands::Create(args)) => IssueRequest::Create(IssueCreateRequest {
                workspace: args.workspace,
                repo: args.repo,
                title: args.title,
                content: args.content,
                state: args.state,
                kind: args.kind,
                priority: args.priority,
                profile: args.profile,
                output: args.output,
            }),
            Some(IssueCommands::Update(args)) => IssueRequest::Update(IssueUpdateRequest {
                workspace: args.workspace,
                repo: args.repo,
                id: args.id,
                title: args.title,
                content: args.content,
                state: args.state,
                kind: args.kind,
                priority: args.priority,
                profile: args.profile,
                output: args.output,
            }),
        }),
        Some(Commands::Wiki { command }) => Request::Wiki(match command {
            None => WikiRequest::Help,
            Some(WikiCommands::List(args)) => WikiRequest::List(WikiListRequest {
                workspace: args.workspace,
                repo: args.repo,
                profile: args.profile,
                output: args.output,
            }),
            Some(WikiCommands::Get(args)) => WikiRequest::Get(WikiGetRequest {
                workspace: args.workspace,
                repo: args.repo,
                page: args.page,
                profile: args.profile,
                output: args.output,
            }),
            Some(WikiCommands::Put(args)) => WikiRequest::Put(WikiPutRequest {
                workspace: args.workspace,
                repo: args.repo,
                page: args.page,
                content: args.content,
                file: args.file,
                message: args.message,
                profile: args.profile,
                output: args.output,
            }),
        }),
    }
}

fn resolve_pr_id(id: Option<String>, pr_id: Option<String>) -> Option<String> {
    id.or(pr_id)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            Request::Pr(PrRequest::Update(PrUpdateRequest {
                id: Some(id),
                ..
            })) if id == "42"
        ));
    }

    #[test]
    fn pr_request_changes_maps_to_request_changes_request() {
        let request = parse_from(["bb", "pr", "request-changes", "--id", "42"])
            .expect("parse should succeed");
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
        let request =
            parse_from(["bb", "pr", "close", "--id", "42"]).expect("parse should succeed");
        assert!(matches!(
            request,
            Request::Pr(PrRequest::Decline(PrDeclineRequest {
                id: Some(id),
                ..
            })) if id == "42"
        ));
    }

    #[test]
    fn pr_checks_alias_maps_to_statuses_request() {
        let request =
            parse_from(["bb", "pr", "checks", "--id", "42"]).expect("parse should succeed");
        assert!(matches!(
            request,
            Request::Pr(PrRequest::Statuses(PrStatusesRequest {
                id: Some(id),
                ..
            })) if id == "42"
        ));
    }

    #[test]
    fn pr_comment_maps_content_and_output() {
        let request = parse_from([
            "bb",
            "pr",
            "comment",
            "--id",
            "42",
            "--content",
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
    fn pr_comments_rejects_positional_and_flag_id_together() {
        let error = parse_from(["bb", "pr", "comments", "42", "--id", "43"])
            .expect_err("parse should fail");
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
    fn pipeline_get_rejects_uuid_and_build_together() {
        let error = parse_from(["bb", "pipeline", "get", "--uuid", "{1234}", "--build", "17"])
            .expect_err("parse should fail");
        assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
    }
}
