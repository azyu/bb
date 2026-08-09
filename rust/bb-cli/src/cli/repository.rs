use bb_core::{IssueRequest, PipelineRequest, PrRequest, Request, WikiRequest};

#[derive(Debug, Clone)]
pub(super) struct RepositoryTarget {
    pub(super) workspace: String,
    pub(super) repo: String,
}

pub(super) fn parse_repository_target(value: &str) -> Result<RepositoryTarget, String> {
    let mut parts = value.split('/');
    let workspace = parts.next().unwrap_or_default();
    let repo = parts.next().unwrap_or_default();
    if workspace.is_empty() || repo.is_empty() || parts.next().is_some() {
        return Err("must use WORKSPACE/REPO with exactly one slash".to_string());
    }
    Ok(RepositoryTarget {
        workspace: workspace.to_string(),
        repo: repo.to_string(),
    })
}

pub(super) fn apply_repository_target(
    request: &mut Request,
    target: RepositoryTarget,
) -> Result<(), clap::Error> {
    match request {
        Request::Pr(request) => match request {
            PrRequest::Help => unsupported_repository_target(),
            PrRequest::List(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PrRequest::Create(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PrRequest::Merge(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PrRequest::Get(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PrRequest::Update(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PrRequest::Approve(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PrRequest::Unapprove(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PrRequest::RequestChanges(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PrRequest::RemoveRequestChanges(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PrRequest::Decline(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PrRequest::Comment(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PrRequest::CommentUpdate(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PrRequest::Comments(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PrRequest::Diff(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PrRequest::Diffstat(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PrRequest::Statuses(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PrRequest::Activity(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
        },
        Request::Pipeline(request) => match request {
            PipelineRequest::Help => unsupported_repository_target(),
            PipelineRequest::List(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PipelineRequest::Get(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PipelineRequest::Steps(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PipelineRequest::Log(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            PipelineRequest::Run(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
        },
        Request::Issue(request) => match request {
            IssueRequest::Help => unsupported_repository_target(),
            IssueRequest::List(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            IssueRequest::Create(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            IssueRequest::Update(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
        },
        Request::Wiki(request) => match request {
            WikiRequest::Help => unsupported_repository_target(),
            WikiRequest::List(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            WikiRequest::Get(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
            WikiRequest::Put(request) => {
                set_repository_target(&mut request.workspace, &mut request.repo, target)
            }
        },
        _ => unsupported_repository_target(),
    }
}

fn set_repository_target(
    workspace: &mut Option<String>,
    repo: &mut Option<String>,
    target: RepositoryTarget,
) -> Result<(), clap::Error> {
    if workspace.is_some() || repo.is_some() {
        return Err(clap::Error::raw(
            clap::error::ErrorKind::ArgumentConflict,
            "-R/--repository cannot be combined with --workspace or --repo",
        ));
    }
    *workspace = Some(target.workspace);
    *repo = Some(target.repo);
    Ok(())
}

fn unsupported_repository_target() -> Result<(), clap::Error> {
    Err(clap::Error::raw(
        clap::error::ErrorKind::ArgumentConflict,
        "-R/--repository is only valid for repository-scoped commands",
    ))
}
