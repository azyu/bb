use std::io::{BufRead, Write};

use crate::config;
use crate::error::CliError;
use crate::render;
use crate::{AuthLoginRequest, AuthRequest, AuthStatusRequest};

use super::STDIN_TOKEN_SENTINEL;

pub(super) fn handle_auth<R: BufRead, O: Write>(
    request: &AuthRequest,
    stdin: &mut R,
    stdout: &mut O,
) -> Result<(), CliError> {
    match request {
        AuthRequest::Help => write!(stdout, "{}", render::auth_usage()).map_err(CliError::from),
        AuthRequest::Login(request) => handle_auth_login(request, stdin, stdout),
        AuthRequest::Status(request) => handle_auth_status(request, stdout),
        AuthRequest::Logout(request) => handle_auth_logout(request, stdout),
    }
}

fn handle_auth_login<R: BufRead, O: Write>(
    request: &AuthLoginRequest,
    stdin: &mut R,
    stdout: &mut O,
) -> Result<(), CliError> {
    let token = resolve_login_token(request, stdin)?;
    let username = request
        .username
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            std::env::var("BITBUCKET_USERNAME")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_default();

    let mut config = config::load()?;
    config.set_profile_with_auth(
        &request.profile,
        &username,
        &token,
        request.base_url.as_deref(),
    );
    config::save(&config)?;

    writeln!(stdout, "authenticated profile {:?}", request.profile)?;
    if username.is_empty() {
        writeln!(stdout, "auth mode: bearer token")?;
    } else {
        writeln!(stdout, "auth mode: basic ({username})")?;
    }
    Ok(())
}

fn handle_auth_status<O: Write>(
    request: &AuthStatusRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let config = config::load()?;
    let (profile, name) = config.active_profile(request.profile.as_deref())?;

    writeln!(stdout, "Profile: {name}")?;
    writeln!(stdout, "Base URL: {}", profile.base_url)?;
    if profile.username.trim().is_empty() {
        writeln!(stdout, "Auth: bearer token")?;
    } else {
        writeln!(stdout, "Auth: basic ({})", profile.username.trim())?;
    }
    if profile.token.trim().is_empty() {
        writeln!(stdout, "Token: not configured")?;
    } else {
        writeln!(stdout, "Token: configured")?;
    }
    Ok(())
}

fn handle_auth_logout<O: Write>(
    request: &crate::AuthLogoutRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let mut config = config::load()?;
    if request
        .profile
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
        && config.current.trim().is_empty()
    {
        return Err(CliError::NotLoggedIn);
    }

    let (removed, ok) = config.remove_profile(request.profile.as_deref());
    if !ok {
        return if removed.trim().is_empty() {
            Err(CliError::NotLoggedIn)
        } else {
            Err(CliError::Config(format!("profile {:?} not found", removed)))
        };
    }

    config::save(&config)?;
    writeln!(stdout, "logged out profile {:?}", removed)?;
    if !config.current.trim().is_empty() {
        writeln!(stdout, "active profile: {:?}", config.current)?;
    }
    Ok(())
}
fn resolve_login_token<R: BufRead>(
    request: &AuthLoginRequest,
    stdin: &mut R,
) -> Result<String, CliError> {
    let token = request
        .token
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if let Some(token) = token {
        if token == STDIN_TOKEN_SENTINEL {
            return read_token_from_stdin(stdin);
        }
        return Ok(token.to_string());
    }
    if request.with_token {
        return read_token_from_stdin(stdin);
    }
    if let Ok(token) = std::env::var("BITBUCKET_TOKEN") {
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }
    Err(CliError::InvalidInput(
        "token is required: use --token <value>, --with-token, or BITBUCKET_TOKEN".to_string(),
    ))
}

fn read_token_from_stdin<R: BufRead>(stdin: &mut R) -> Result<String, CliError> {
    let mut buffer = String::new();
    let bytes = stdin.read_line(&mut buffer)?;
    if bytes == 0 {
        return Err(CliError::InvalidInput(
            "no token provided on stdin".to_string(),
        ));
    }
    let token = buffer.trim().to_string();
    if token.is_empty() {
        return Err(CliError::InvalidInput(
            "no token provided on stdin".to_string(),
        ));
    }
    Ok(token)
}
