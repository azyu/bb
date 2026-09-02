use std::io::{BufRead, Write};

use crate::error::CliError;
use crate::render::{self, ErrorEnvelope, ErrorPayload};
use crate::version;
use crate::{
    CompletionShell, IssueRequest, PipelineRequest, PrRequest, RepoRequest, Request, WikiRequest,
};

mod api;
mod auth;
mod issue;
mod pipeline;
mod pr;
mod repo;
mod support;
mod wiki;

pub const STDIN_TOKEN_SENTINEL: &str = "__bb_stdin_token__";

pub fn run<R: BufRead, O: Write, E: Write>(
    request: Request,
    stdin: &mut R,
    stdout: &mut O,
    stderr: &mut E,
    stdout_is_tty: bool,
    stderr_is_tty: bool,
) -> u8 {
    match dispatch(
        &request,
        stdin,
        stdout,
        stderr,
        stdout_is_tty,
        stderr_is_tty,
    ) {
        Ok(()) => 0,
        Err(error) => {
            let _ = emit_error(&request, &error, stdout, stderr);
            1
        }
    }
}

fn dispatch<R: BufRead, O: Write, E: Write>(
    request: &Request,
    stdin: &mut R,
    stdout: &mut O,
    stderr: &mut E,
    stdout_is_tty: bool,
    stderr_is_tty: bool,
) -> Result<(), CliError> {
    match request {
        Request::RootHelp => {
            write!(stdout, "{}", render::root_usage())?;
        }
        Request::Version => {
            write!(
                stdout,
                "bb version {}\ncommit: {}\nbuilt: {}\n",
                version::display_version(),
                version::short_commit(),
                version::BUILD_DATE
            )?;
        }
        Request::Completion(shell) => {
            let Some(shell) = shell.as_deref() else {
                write!(stdout, "{}", render::completion_usage())?;
                return Ok(());
            };
            let script = match normalize_completion_shell(shell)? {
                CompletionShell::Bash => render::bash_completion_script(),
                CompletionShell::Zsh => render::zsh_completion_script(),
                CompletionShell::Fish => render::fish_completion_script(),
                CompletionShell::Powershell => render::powershell_completion_script(),
            };
            writeln!(stdout, "{script}")?;
        }
        Request::Auth(auth) => auth::handle_auth(auth, stdin, stdout)?,
        Request::Api(api) => api::handle_api(api, stdin, stdout)?,
        Request::Repo(repo) => repo::handle_repo(repo, stdout)?,
        Request::Pr(pr) => pr::handle_pr(pr, stdout, stderr, stdout_is_tty, stderr_is_tty)?,
        Request::Pipeline(pipeline) => pipeline::handle_pipeline(pipeline, stdout)?,
        Request::Issue(issue) => issue::handle_issue(issue, stdout)?,
        Request::Wiki(wiki) => wiki::handle_wiki(wiki, stdout)?,
    }

    Ok(())
}

fn emit_error<O: Write, E: Write>(
    request: &Request,
    error: &CliError,
    stdout: &mut O,
    stderr: &mut E,
) -> Result<(), CliError> {
    if wants_json_errors(request) {
        return render::print_json(
            stdout,
            &ErrorEnvelope {
                error: ErrorPayload {
                    code: error.code(),
                    message: error.message(),
                },
            },
        );
    }
    writeln!(stderr, "{}", error.message()).map_err(CliError::from)
}

fn wants_json_errors(request: &Request) -> bool {
    match request {
        Request::Api(_) => true,
        Request::Repo(RepoRequest::List(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::List(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Create(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Merge(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Get(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Update(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Approve(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Unapprove(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::RequestChanges(req)) => {
            req.output.trim().eq_ignore_ascii_case("json")
        }
        Request::Pr(PrRequest::RemoveRequestChanges(req)) => {
            req.output.trim().eq_ignore_ascii_case("json")
        }
        Request::Pr(PrRequest::Decline(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Comment(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::CommentUpdate(req)) => {
            req.output.trim().eq_ignore_ascii_case("json")
        }
        Request::Pr(PrRequest::Comments(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Diff(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Diffstat(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Statuses(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pr(PrRequest::Activity(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Pipeline(PipelineRequest::List(req)) => {
            req.output.trim().eq_ignore_ascii_case("json")
        }
        Request::Pipeline(PipelineRequest::Get(req)) => {
            req.output.trim().eq_ignore_ascii_case("json")
        }
        Request::Pipeline(PipelineRequest::Steps(req)) => {
            req.output.trim().eq_ignore_ascii_case("json")
        }
        Request::Pipeline(PipelineRequest::Log(req)) => {
            req.output.trim().eq_ignore_ascii_case("json")
        }
        Request::Pipeline(PipelineRequest::Run(req)) => {
            req.output.trim().eq_ignore_ascii_case("json")
        }
        Request::Issue(IssueRequest::List(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Issue(IssueRequest::Create(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Issue(IssueRequest::Update(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Wiki(WikiRequest::List(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Wiki(WikiRequest::Get(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        Request::Wiki(WikiRequest::Put(req)) => req.output.trim().eq_ignore_ascii_case("json"),
        _ => false,
    }
}

fn normalize_completion_shell(value: &str) -> Result<CompletionShell, CliError> {
    match value.trim().to_lowercase().as_str() {
        "bash" => Ok(CompletionShell::Bash),
        "zsh" => Ok(CompletionShell::Zsh),
        "fish" => Ok(CompletionShell::Fish),
        "powershell" => Ok(CompletionShell::Powershell),
        other => Err(CliError::InvalidInput(format!(
            "unsupported shell: {other}"
        ))),
    }
}
