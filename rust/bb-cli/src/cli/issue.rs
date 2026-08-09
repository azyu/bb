use bb_core::{IssueCreateRequest, IssueListRequest, IssueRequest, IssueUpdateRequest, Request};
use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub(super) enum IssueCommands {
    /// List repository issues
    List(IssueListArgs),
    /// Create a repository issue
    Create(IssueCreateArgs),
    /// Update a repository issue
    Update(IssueUpdateArgs),
}

#[derive(Debug, Args)]
pub(super) struct IssueListArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long, default_value = "table")]
    pub(super) output: String,
    #[arg(long)]
    pub(super) all: bool,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long)]
    pub(super) q: Option<String>,
    #[arg(long)]
    pub(super) sort: Option<String>,
    #[arg(long)]
    pub(super) fields: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct IssueCreateArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) title: Option<String>,
    #[arg(long)]
    pub(super) content: Option<String>,
    #[arg(long)]
    pub(super) state: Option<String>,
    #[arg(long)]
    pub(super) kind: Option<String>,
    #[arg(long)]
    pub(super) priority: Option<String>,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct IssueUpdateArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) id: Option<u64>,
    #[arg(long)]
    pub(super) title: Option<String>,
    #[arg(long)]
    pub(super) content: Option<String>,
    #[arg(long)]
    pub(super) state: Option<String>,
    #[arg(long)]
    pub(super) kind: Option<String>,
    #[arg(long)]
    pub(super) priority: Option<String>,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

pub(super) fn map_request(command: Option<IssueCommands>) -> Request {
    Request::Issue(match command {
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
    })
}
