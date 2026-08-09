use bb_core::{
    PipelineGetRequest, PipelineListRequest, PipelineLogRequest, PipelineRequest,
    PipelineRunRequest, PipelineStepsRequest, Request,
};
use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub(super) enum PipelineCommands {
    /// List pipelines
    List(PipelineListArgs),
    /// Get a pipeline
    Get(PipelineGetArgs),
    /// List steps for a pipeline
    Steps(PipelineStepsArgs),
    /// Get a pipeline step log
    Log(PipelineLogArgs),
    /// Run a pipeline for a branch
    Run(PipelineRunArgs),
}

#[derive(Debug, Args)]
pub(super) struct PipelineListArgs {
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
    pub(super) branch: Option<String>,
    #[arg(
        long,
        help = "Sort expression passed to the API; when unset the API is observed to return oldest-first, so pass --sort=-created_on for the most recent builds"
    )]
    pub(super) sort: Option<String>,
    #[arg(long)]
    pub(super) fields: Option<String>,
    #[arg(long)]
    pub(super) json_fields: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct PipelineGetArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long, conflicts_with = "build")]
    pub(super) uuid: Option<String>,
    #[arg(long, conflicts_with = "uuid")]
    pub(super) build: Option<String>,
    #[arg(index = 1, value_name = "SELECTOR", conflicts_with_all = ["uuid", "build"])]
    pub(super) selector: Option<String>,
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
pub(super) struct PipelineStepsArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long, conflicts_with = "build")]
    pub(super) uuid: Option<String>,
    #[arg(long, conflicts_with = "uuid")]
    pub(super) build: Option<String>,
    #[arg(index = 1, value_name = "SELECTOR", conflicts_with_all = ["uuid", "build"])]
    pub(super) selector: Option<String>,
    #[arg(long, default_value = "table")]
    pub(super) output: String,
    #[arg(long)]
    pub(super) all: bool,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long)]
    pub(super) sort: Option<String>,
    #[arg(long)]
    pub(super) fields: Option<String>,
    #[arg(long)]
    pub(super) json_fields: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct PipelineLogArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long, conflicts_with = "build")]
    pub(super) uuid: Option<String>,
    #[arg(long, conflicts_with = "uuid")]
    pub(super) build: Option<String>,
    #[arg(index = 1, value_name = "SELECTOR", conflicts_with_all = ["uuid", "build"])]
    pub(super) selector: Option<String>,
    #[arg(long)]
    pub(super) step: Option<String>,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct PipelineRunArgs {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    #[arg(long)]
    pub(super) branch: Option<String>,
    #[arg(long)]
    pub(super) profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

pub(super) fn map_request(command: Option<PipelineCommands>) -> Request {
    Request::Pipeline(match command {
        None => PipelineRequest::Help,
        Some(PipelineCommands::List(args)) => PipelineRequest::List(PipelineListRequest {
            workspace: args.workspace,
            repo: args.repo,
            output: args.output,
            all: args.all,
            profile: args.profile,
            branch: args.branch,
            sort: args.sort,
            fields: args.fields,
            json_fields: args.json_fields,
        }),
        Some(PipelineCommands::Get(args)) => PipelineRequest::Get(PipelineGetRequest {
            workspace: args.workspace,
            repo: args.repo,
            uuid: args.uuid,
            build: args.build,
            positional_selector: args.selector,
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
            positional_selector: args.selector,
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
            positional_selector: args.selector,
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
    })
}
