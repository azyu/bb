use bb_core::{AuthLoginRequest, AuthLogoutRequest, AuthRequest, AuthStatusRequest, Request};
use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub(super) enum AuthCommands {
    /// Save a Bitbucket Cloud authentication profile
    Login(AuthLoginArgs),
    /// Show authentication profile status
    Status(AuthStatusArgs),
    /// Remove a saved authentication profile
    Logout(AuthLogoutArgs),
}

#[derive(Debug, Args)]
pub(super) struct AuthLoginArgs {
    #[arg(long, default_value = "default")]
    /// Authentication profile name
    pub(super) profile: String,
    #[arg(long)]
    /// API token value, or read from stdin when omitted
    pub(super) token: Option<String>,
    #[arg(long)]
    /// Bitbucket username used with API tokens
    pub(super) username: Option<String>,
    #[arg(long)]
    /// Read the API token from stdin
    pub(super) with_token: bool,
    #[arg(long)]
    /// Bitbucket API base URL
    pub(super) base_url: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct AuthStatusArgs {
    #[arg(long)]
    /// Authentication profile name
    pub(super) profile: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct AuthLogoutArgs {
    #[arg(long)]
    /// Authentication profile name
    pub(super) profile: Option<String>,
}

pub(super) fn map_request(command: Option<AuthCommands>) -> Request {
    Request::Auth(match command {
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
    })
}
