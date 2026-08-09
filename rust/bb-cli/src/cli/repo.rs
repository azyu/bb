use bb_core::{RepoListRequest, RepoRequest, Request};
use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub(super) enum RepoCommands {
    /// List repositories in a workspace
    List(RepoListArgs),
}

#[derive(Debug, Args)]
pub(super) struct RepoListArgs {
    #[arg(long)]
    /// Bitbucket workspace slug
    pub(super) workspace: Option<String>,
    #[arg(long, default_value = "table")]
    /// Output format
    pub(super) output: String,
    #[arg(long)]
    /// Fetch all pages instead of the first page only
    pub(super) all: bool,
    #[arg(long)]
    /// Authentication profile name
    pub(super) profile: Option<String>,
    #[arg(long)]
    /// Bitbucket Cloud API filter expression
    pub(super) q: Option<String>,
    #[arg(long)]
    /// Bitbucket Cloud API sort expression
    pub(super) sort: Option<String>,
    #[arg(long)]
    /// Bitbucket Cloud API partial-response fields
    pub(super) fields: Option<String>,
    #[arg(long)]
    /// Comma-separated fields to include in JSON output
    pub(super) json_fields: Option<String>,
}

pub(super) fn map_request(command: Option<RepoCommands>) -> Request {
    Request::Repo(match command {
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
    })
}
