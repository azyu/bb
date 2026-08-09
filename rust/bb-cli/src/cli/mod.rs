use std::ffi::OsString;

use bb_core::Request;
use clap::{Parser, Subcommand};

mod api;
mod auth;
mod issue;
mod pipeline;
mod pr;
mod repo;
mod repository;
mod wiki;

#[cfg(test)]
mod tests;

const STDIN_TOKEN_SENTINEL: &str = bb_core::runtime::STDIN_TOKEN_SENTINEL;

#[derive(Debug, Parser)]
#[command(
    name = "bb",
    disable_version_flag = true,
    disable_help_subcommand = true
)]
struct Cli {
    #[arg(short = 'v', long = "version", global = true)]
    pub version: bool,

    /// Select a repository as WORKSPACE/REPO
    #[arg(
        short = 'R',
        long = "repository",
        global = true,
        value_name = "WORKSPACE/REPO",
        value_parser = repository::parse_repository_target
    )]
    repository: Option<repository::RepositoryTarget>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Authenticate and manage saved profiles
    Auth {
        #[command(subcommand)]
        command: Option<auth::AuthCommands>,
    },
    /// Call the Bitbucket Cloud REST API directly
    Api(api::ApiArgs),
    /// Work with Bitbucket repositories
    Repo {
        #[command(subcommand)]
        command: Option<repo::RepoCommands>,
    },
    /// Work with pull requests
    Pr {
        #[command(subcommand)]
        command: Option<pr::PrCommands>,
    },
    /// Inspect and run Pipelines
    Pipeline {
        #[command(subcommand)]
        command: Option<pipeline::PipelineCommands>,
    },
    /// Work with repository issues
    Issue {
        #[command(subcommand)]
        command: Option<issue::IssueCommands>,
    },
    /// Read and update repository wiki pages
    Wiki {
        #[command(subcommand)]
        command: Option<wiki::WikiCommands>,
    },
    /// Generate shell completion scripts
    Completion(CompletionArgs),
    /// Show build version metadata
    Version,
}

#[derive(Debug, clap::Args)]
struct CompletionArgs {
    /// Shell name supported by clap_complete
    pub shell: Option<String>,
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

    let mut cli = Cli::try_parse_from(normalized)?;
    let repository = cli.repository.take();
    let mut request = map_request(cli);
    if let Some(repository) = repository {
        repository::apply_repository_target(&mut request, repository)?;
    }
    Ok(request)
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
        Some(Commands::Auth { command }) => auth::map_request(command),
        Some(Commands::Api(args)) => api::map_request(args),
        Some(Commands::Repo { command }) => repo::map_request(command),
        Some(Commands::Pr { command }) => pr::map_request(command),
        Some(Commands::Pipeline { command }) => pipeline::map_request(command),
        Some(Commands::Issue { command }) => issue::map_request(command),
        Some(Commands::Wiki { command }) => wiki::map_request(command),
    }
}
