use bb_core::{
    PrActivityRequest, PrApproveRequest, PrCommentRequest, PrCommentUpdateRequest,
    PrCommentsRequest, PrCreateRequest, PrDeclineRequest, PrDiffRequest, PrDiffstatRequest,
    PrGetRequest, PrListRequest, PrMergeRequest, PrRemoveRequestChangesRequest, PrRequest,
    PrRequestChangesRequest, PrStatusesRequest, PrUnapproveRequest, PrUpdateRequest, Request,
};
use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub(super) enum PrCommands {
    /// List pull requests
    List(PrListArgs),
    /// Create a pull request
    Create(PrCreateArgs),
    /// Merge a pull request
    Merge(PrMergeArgs),
    /// Get a pull request
    #[command(visible_alias = "view")]
    Get(PrGetArgs),
    /// Update a pull request
    #[command(visible_alias = "edit")]
    Update(PrUpdateArgs),
    /// Approve a pull request
    Approve(PrApproveArgs),
    /// Remove your pull request approval
    Unapprove(PrUnapproveArgs),
    /// Request changes on a pull request
    RequestChanges(PrRequestChangesArgs),
    /// Remove your change request
    RemoveRequestChanges(PrRemoveRequestChangesArgs),
    /// Decline a pull request
    #[command(visible_alias = "close")]
    Decline(PrDeclineArgs),
    /// Create a new pull request comment
    #[command(
        long_about = "Create a new pull request comment. This command does not edit existing comments; use comment-update for that."
    )]
    Comment(PrCommentArgs),
    /// Update an existing pull request comment
    CommentUpdate(PrCommentUpdateArgs),
    /// List pull request comments or get one comment
    Comments(PrCommentsArgs),
    /// Get the raw pull request diff
    Diff(PrDiffArgs),
    /// List pull request file changes and line counts
    Diffstat(PrDiffstatArgs),
    /// List pull request commit statuses
    #[command(visible_alias = "checks")]
    Statuses(PrStatusesArgs),
    /// List pull request activity
    Activity(PrActivityArgs),
}

#[derive(Debug, Args)]
pub(super) struct PrListArgs {
    #[arg(long)]
    /// Bitbucket workspace slug
    pub(super) workspace: Option<String>,
    #[arg(long)]
    /// Bitbucket repository slug
    pub(super) repo: Option<String>,
    #[arg(long, default_value = "table")]
    /// Output format
    pub(super) output: String,
    #[arg(long)]
    /// Fetch all pages instead of the first page only
    pub(super) all: bool,
    /// Maximum number of pull requests to fetch
    #[arg(
        short = 'L',
        long,
        value_name = "N",
        conflicts_with = "all",
        value_parser = parse_positive_usize
    )]
    pub(super) limit: Option<usize>,
    #[arg(long)]
    /// Authentication profile name
    pub(super) profile: Option<String>,
    #[arg(long)]
    /// Pull request state: OPEN, MERGED, or DECLINED
    pub(super) state: Option<String>,
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

#[derive(Debug, Args)]
pub(super) struct PrCreateArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) title: Option<String>,
    #[arg(long)]
    pub(super) source: Option<String>,
    #[arg(long)]
    pub(super) destination: Option<String>,
    #[arg(long, visible_alias = "body")]
    pub(super) description: Option<String>,
    #[arg(long)]
    pub(super) close_branch: bool,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct PrMergeArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub(super) pr_id: Option<String>,
    #[arg(long)]
    pub(super) message: Option<String>,
    #[arg(long)]
    pub(super) strategy: Option<String>,
    #[arg(long)]
    pub(super) close_branch: bool,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct PrGetArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub(super) pr_id: Option<String>,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
    #[arg(long)]
    pub(super) fields: Option<String>,
    #[arg(long)]
    pub(super) json_fields: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct PrUpdateArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub(super) pr_id: Option<String>,
    #[arg(long)]
    pub(super) title: Option<String>,
    #[arg(long, visible_alias = "body")]
    pub(super) description: Option<String>,
    #[arg(long)]
    pub(super) source: Option<String>,
    #[arg(long)]
    pub(super) destination: Option<String>,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct PrApproveArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub(super) pr_id: Option<String>,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct PrUnapproveArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub(super) pr_id: Option<String>,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct PrRequestChangesArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub(super) pr_id: Option<String>,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct PrRemoveRequestChangesArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub(super) pr_id: Option<String>,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct PrDeclineArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub(super) pr_id: Option<String>,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct PrCommentArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long, value_name = "PR_ID", help = "Pull request ID")]
    pub(super) id: Option<String>,
    #[arg(index = 1, value_name = "PR_ID", conflicts_with = "id")]
    pub(super) pr_id: Option<String>,
    #[arg(long, visible_alias = "body")]
    pub(super) content: Option<String>,
    /// Parent comment ID for replies
    #[arg(long)]
    pub(super) parent: Option<String>,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct PrCommentUpdateArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long, value_name = "PR_ID", help = "Pull request ID")]
    pub(super) id: Option<String>,
    #[arg(index = 1, value_name = "PR_ID", conflicts_with = "id")]
    pub(super) pr_id: Option<String>,
    #[arg(long, value_name = "COMMENT_ID")]
    pub(super) comment_id: Option<String>,
    #[arg(long, visible_alias = "body")]
    pub(super) content: Option<String>,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct PrCommentsArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long, value_name = "PR_ID", help = "Pull request ID")]
    pub(super) id: Option<String>,
    #[arg(index = 1, value_name = "PR_ID", conflicts_with = "id")]
    pub(super) pr_id: Option<String>,
    #[arg(long, value_name = "COMMENT_ID")]
    pub(super) comment_id: Option<String>,
    #[arg(long, default_value = "table")]
    pub(super) output: String,
    #[arg(long, help = "Fetch all comment pages instead of the first page only")]
    pub(super) all: bool,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long)]
    pub(super) q: Option<String>,
    #[arg(long)]
    pub(super) sort: Option<String>,
    #[arg(long)]
    pub(super) fields: Option<String>,
    #[arg(long)]
    pub(super) json_fields: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct PrDiffArgs {
    /// Bitbucket workspace slug
    #[arg(long)]
    pub(super) workspace: Option<String>,
    /// Bitbucket repository slug
    #[arg(long)]
    pub(super) repo: Option<String>,
    /// Pull request ID
    #[arg(long)]
    pub(super) id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub(super) pr_id: Option<String>,
    /// Print changed file paths instead of the raw diff
    #[arg(long)]
    pub(super) name_only: bool,
    /// Authentication profile name
    #[arg(long)]
    pub(super) profile: Option<String>,
    /// Output format
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct PrDiffstatArgs {
    /// Bitbucket workspace slug
    #[arg(long)]
    pub(super) workspace: Option<String>,
    /// Bitbucket repository slug
    #[arg(long)]
    pub(super) repo: Option<String>,
    /// Pull request ID
    #[arg(long)]
    pub(super) id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub(super) pr_id: Option<String>,
    /// Output format
    #[arg(long, default_value = "table")]
    pub(super) output: String,
    /// Fetch all pages instead of the first page only
    #[arg(long)]
    pub(super) all: bool,
    /// Authentication profile name
    #[arg(long)]
    pub(super) profile: Option<String>,
    /// Bitbucket Cloud API filter expression
    #[arg(long)]
    pub(super) q: Option<String>,
    /// Bitbucket Cloud API sort expression
    #[arg(long)]
    pub(super) sort: Option<String>,
    /// Bitbucket Cloud API partial-response fields
    #[arg(long)]
    pub(super) fields: Option<String>,
    /// Comma-separated fields to include in JSON output
    #[arg(long)]
    pub(super) json_fields: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct PrStatusesArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub(super) pr_id: Option<String>,
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
    #[arg(long)]
    pub(super) json_fields: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct PrActivityArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) id: Option<String>,
    #[arg(index = 1, value_name = "ID", conflicts_with = "id")]
    pub(super) pr_id: Option<String>,
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
    #[arg(long)]
    pub(super) json_fields: Option<String>,
}

fn parse_positive_usize(value: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "must be a positive integer".to_string())?;
    if parsed == 0 {
        return Err("must be greater than zero".to_string());
    }
    Ok(parsed)
}

fn resolve_pr_id(id: Option<String>, pr_id: Option<String>) -> Option<String> {
    id.or(pr_id)
}

pub(super) fn map_request(command: Option<PrCommands>) -> Request {
    Request::Pr(match command {
        None => PrRequest::Help,
        Some(PrCommands::List(args)) => PrRequest::List(PrListRequest {
            workspace: args.workspace,
            repo: args.repo,
            output: args.output,
            all: args.all,
            limit: args.limit,
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
            parent: args.parent,
            profile: args.profile,
            output: args.output,
        }),
        Some(PrCommands::CommentUpdate(args)) => PrRequest::CommentUpdate(PrCommentUpdateRequest {
            workspace: args.workspace,
            repo: args.repo,
            id: resolve_pr_id(args.id, args.pr_id),
            comment_id: args.comment_id,
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
            name_only: args.name_only,
        }),
        Some(PrCommands::Diffstat(args)) => PrRequest::Diffstat(PrDiffstatRequest {
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
    })
}
