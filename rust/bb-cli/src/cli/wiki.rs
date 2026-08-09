use bb_core::{Request, WikiGetRequest, WikiListRequest, WikiPutRequest, WikiRequest};
use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub(super) enum WikiCommands {
    /// List wiki pages
    List(WikiListArgs),
    /// Get a wiki page
    Get(WikiGetArgs),
    /// Create or update a wiki page
    Put(WikiPutArgs),
}

#[derive(Debug, Args)]
pub(super) struct WikiListArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "table")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct WikiGetArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) page: Option<String>,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct WikiPutArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) page: Option<String>,
    #[arg(long)]
    pub(super) content: Option<String>,
    #[arg(long)]
    pub(super) file: Option<String>,
    #[arg(long)]
    pub(super) message: Option<String>,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

pub(super) fn map_request(command: Option<WikiCommands>) -> Request {
    Request::Wiki(match command {
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
    })
}
