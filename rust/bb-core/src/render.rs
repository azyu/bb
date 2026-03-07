use std::env;
use std::io::Write;

use serde::Serialize;
use serde_json::Value;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use unicode_width::UnicodeWidthStr;

use crate::error::CliError;
use crate::version;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorOutput {
    Text,
    Json,
}

#[derive(Debug, Serialize)]
pub struct ErrorEnvelope {
    pub error: ErrorPayload,
}

#[derive(Debug, Serialize)]
pub struct ErrorPayload {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct PrTableRow {
    pub id: u64,
    pub title: String,
    pub branch: String,
    pub created_on: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WikiPage {
    pub path: String,
    pub size: u64,
}

pub fn print_json<W: Write, T: Serialize>(writer: &mut W, value: &T) -> Result<(), CliError> {
    let payload = serde_json::to_string_pretty(value)
        .map_err(|error| CliError::Internal(error.to_string()))?;
    writer.write_all(payload.as_bytes())?;
    writer.write_all(b"\n")?;
    Ok(())
}

pub fn root_usage() -> String {
    format!(
        "bb - Bitbucket CLI (Cloud MVP)\nVersion: {}\n\nUsage:\n  bb <command> [subcommand] [flags]\n\nCommands:\n  auth       Authenticate and inspect auth status\n  api        Call Bitbucket Cloud REST endpoints\n  repo       Repository operations\n  version    Show CLI version metadata\n  pr         Pull request operations\n  pipeline   Pipeline operations\n  wiki       Wiki operations\n  issue      Issue operations\n  completion Shell completion\n",
        version::display_version()
    )
}

pub fn auth_usage() -> &'static str {
    "Authenticate and inspect auth status\n\nUsage:\n  bb auth <command>\n\nCommands:\n  login    Authenticate with Bitbucket\n  status   Show current auth status\n  logout   Remove stored credentials\n"
}

pub fn repo_usage() -> &'static str {
    "Repository operations\n\nUsage:\n  bb repo <command>\n\nCommands:\n  list   List repositories in a workspace\n"
}

pub fn pr_usage() -> &'static str {
    "Pull request operations\n\nUsage:\n  bb pr <command>\n\nCommands:\n  list                    List pull requests\n  create                  Create a pull request\n  merge                   Merge a pull request\n  get                     Get a pull request\n  update                  Update a pull request\n  approve                 Approve a pull request\n  unapprove               Unapprove a pull request\n  request-changes         Request changes on a pull request\n  remove-request-changes  Remove change request for a pull request\n  decline                 Decline a pull request\n  comment                 Create a pull request comment\n  comments                List pull request comments\n  diff                    Get the diff for a pull request\n  statuses                List commit statuses for a pull request\n  activity                List pull request activity\n"
}

pub fn pipeline_usage() -> &'static str {
    "Pipeline operations\n\nUsage:\n  bb pipeline <command>\n\nCommands:\n  list   List pipelines\n  run    Trigger a pipeline\n"
}

pub fn issue_usage() -> &'static str {
    "Issue operations\n\nUsage:\n  bb issue <command>\n\nCommands:\n  list     List issues\n  create   Create an issue\n  update   Update an issue\n"
}

pub fn wiki_usage() -> &'static str {
    "Wiki operations\n\nUsage:\n  bb wiki <command>\n\nCommands:\n  list   List wiki pages\n  get    Get wiki page content\n  put    Create or update a wiki page\n"
}

pub fn completion_usage() -> &'static str {
    "Generate shell completion scripts\n\nUsage:\n  bb completion <shell>\n\nShells:\n  bash         Bash completion\n  zsh          Zsh completion\n  fish         Fish completion\n  powershell   PowerShell completion\n"
}

pub fn bash_completion_script() -> &'static str {
    r#"_bb_complete() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local prev="${COMP_WORDS[COMP_CWORD-1]}"
  case "${prev}" in
    auth)       COMPREPLY=($(compgen -W "login status logout" -- "${cur}")); return;;
    repo)       COMPREPLY=($(compgen -W "list" -- "${cur}")); return;;
    pr)         COMPREPLY=($(compgen -W "list create merge get update approve unapprove request-changes remove-request-changes decline comment comments diff statuses activity" -- "${cur}")); return;;
    pipeline)   COMPREPLY=($(compgen -W "list run" -- "${cur}")); return;;
    issue)      COMPREPLY=($(compgen -W "list create update" -- "${cur}")); return;;
    wiki)       COMPREPLY=($(compgen -W "list get put" -- "${cur}")); return;;
    completion) COMPREPLY=($(compgen -W "bash zsh fish powershell" -- "${cur}")); return;;
  esac
  local cmds="auth api repo pr pipeline wiki issue completion version help"
  COMPREPLY=($(compgen -W "${cmds}" -- "${cur}"))
}
complete -F _bb_complete bb"#
}

pub fn zsh_completion_script() -> &'static str {
    r#"#compdef bb
_bb() {
  local -a commands subcmds
  commands=(auth api repo pr pipeline wiki issue completion version help)
  _arguments "1:command:($commands)" "*::arg:->args"
  case $words[1] in
    auth)       subcmds=(login status logout);;
    repo)       subcmds=(list);;
    pr)         subcmds=(list create merge get update approve unapprove request-changes remove-request-changes decline comment comments diff statuses activity);;
    pipeline)   subcmds=(list run);;
    issue)      subcmds=(list create update);;
    wiki)       subcmds=(list get put);;
    completion) subcmds=(bash zsh fish powershell);;
  esac
  [[ -n "$subcmds" ]] && _describe 'subcommand' subcmds
}
compdef _bb bb"#
}

pub fn fish_completion_script() -> &'static str {
    r#"complete -c bb -f -n '__fish_use_subcommand' -a "auth api repo pr pipeline wiki issue completion version help"
complete -c bb -f -n '__fish_seen_subcommand_from auth' -a "login status logout"
complete -c bb -f -n '__fish_seen_subcommand_from repo' -a "list"
complete -c bb -f -n '__fish_seen_subcommand_from pr' -a "list create merge get update approve unapprove request-changes remove-request-changes decline comment comments diff statuses activity"
complete -c bb -f -n '__fish_seen_subcommand_from pipeline' -a "list run"
complete -c bb -f -n '__fish_seen_subcommand_from issue' -a "list create update"
complete -c bb -f -n '__fish_seen_subcommand_from wiki' -a "list get put"
complete -c bb -f -n '__fish_seen_subcommand_from completion' -a "bash zsh fish powershell""#
}

pub fn powershell_completion_script() -> &'static str {
    r#"Register-ArgumentCompleter -CommandName bb -ScriptBlock {
  param($wordToComplete, $commandAst)
  $tokens = $commandAst.ToString() -split '\s+'
  $subcmds = @{
    'auth'       = @('login','status','logout')
    'repo'       = @('list')
    'pr'         = @('list','create','merge','get','update','approve','unapprove','request-changes','remove-request-changes','decline','comment','comments','diff','statuses','activity')
    'pipeline'   = @('list','run')
    'issue'      = @('list','create','update')
    'wiki'       = @('list','get','put')
    'completion' = @('bash','zsh','fish','powershell')
  }
  if ($tokens.Count -ge 2 -and $subcmds.ContainsKey($tokens[1])) {
    $subcmds[$tokens[1]] |
      Where-Object { $_ -like "$wordToComplete*" } |
      ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }
  } else {
    "auth","api","repo","pr","pipeline","wiki","issue","completion","version","help" |
      Where-Object { $_ -like "$wordToComplete*" } |
      ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }
  }
}"#
}

pub fn should_use_color(stdout_is_tty: bool) -> bool {
    let mode = env::var("BB_COLOR")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    match mode.as_str() {
        "always" => return true,
        "never" => return false,
        _ => {}
    }
    if env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if env::var("CLICOLOR").unwrap_or_default().trim() == "0" {
        return false;
    }
    let force = env::var("CLICOLOR_FORCE").unwrap_or_default();
    if !force.trim().is_empty() && force.trim() != "0" {
        return true;
    }
    let term = env::var("TERM").unwrap_or_default();
    stdout_is_tty && !term.trim().is_empty() && term.trim() != "dumb"
}

pub fn render_repo_table(values: &[Value]) -> String {
    let rows = values
        .iter()
        .map(|value| {
            vec![
                string_field(value, &["slug"]).unwrap_or("-").to_string(),
                string_field(value, &["full_name"])
                    .unwrap_or("-")
                    .to_string(),
            ]
        })
        .collect::<Vec<_>>();
    format!("{}\n", render_table(&["SLUG", "FULL_NAME"], &rows))
}

pub fn render_pr_table(
    rows: &[PrTableRow],
    workspace: &str,
    repo: &str,
    state_label: &str,
    total_count: Option<usize>,
    use_color: bool,
) -> String {
    let mut width_id = UnicodeWidthStr::width("ID");
    let mut width_title = UnicodeWidthStr::width("TITLE");
    let mut width_branch = UnicodeWidthStr::width("BRANCH");
    let mut width_created = UnicodeWidthStr::width("CREATED AT");

    let view_rows = rows
        .iter()
        .map(|row| {
            let id = format!("#{}", row.id);
            let title = if row.title.trim().is_empty() {
                "-"
            } else {
                row.title.trim()
            }
            .to_string();
            let branch = if row.branch.trim().is_empty() {
                "-"
            } else {
                row.branch.trim()
            }
            .to_string();
            let created = relative_time_label(&row.created_on);
            width_id = width_id.max(UnicodeWidthStr::width(id.as_str()));
            width_title = width_title.max(UnicodeWidthStr::width(title.as_str()));
            width_branch = width_branch.max(UnicodeWidthStr::width(branch.as_str()));
            width_created = width_created.max(UnicodeWidthStr::width(created.as_str()));
            (id, title, branch, created)
        })
        .collect::<Vec<_>>();

    let mut output = match total_count {
        Some(total) if total > rows.len() => {
            format!(
                "Showing {} of {total} {state_label} in {workspace}/{repo}\n\n",
                rows.len()
            )
        }
        _ => format!(
            "Showing {} {state_label} in {workspace}/{repo}\n\n",
            rows.len()
        ),
    };
    output.push_str(&format!(
        "{}  {}  {}  {}\n",
        ansi(&pad_right("ID", width_id), "1", use_color),
        ansi(&pad_right("TITLE", width_title), "1", use_color),
        ansi(&pad_right("BRANCH", width_branch), "1", use_color),
        ansi(&pad_right("CREATED AT", width_created), "1", use_color),
    ));
    for (id, title, branch, created) in view_rows {
        output.push_str(&format!(
            "{}  {}  {}  {}\n",
            ansi(&pad_right(&id, width_id), "1;36", use_color),
            pad_right(&title, width_title),
            ansi(&pad_right(&branch, width_branch), "36", use_color),
            ansi(&pad_right(&created, width_created), "2", use_color),
        ));
    }
    output
}

pub fn render_pr_comments_table(values: &[Value]) -> String {
    let rows = values
        .iter()
        .map(|value| {
            vec![
                int_field(value, &["id"]).unwrap_or_default().to_string(),
                first_string_field(
                    value,
                    &[&["user", "display_name"], &["author", "display_name"]],
                )
                .unwrap_or("-")
                .to_string(),
                relative_time_label(string_field(value, &["created_on"]).unwrap_or_default()),
                compact_text(string_field(value, &["content", "raw"]).unwrap_or("-"), 60),
            ]
        })
        .collect::<Vec<_>>();
    format!(
        "{}\n",
        render_table(&["ID", "AUTHOR", "CREATED AT", "CONTENT"], &rows)
    )
}

pub fn render_pr_statuses_table(values: &[Value]) -> String {
    let rows = values
        .iter()
        .map(|value| {
            vec![
                string_field(value, &["key"]).unwrap_or("-").to_string(),
                string_field(value, &["state"]).unwrap_or("-").to_string(),
                first_string_field(value, &[&["name"], &["description"]])
                    .unwrap_or("-")
                    .to_string(),
                relative_time_label(
                    first_string_field(value, &[&["updated_on"], &["created_on"]])
                        .unwrap_or_default(),
                ),
            ]
        })
        .collect::<Vec<_>>();
    format!(
        "{}\n",
        render_table(&["KEY", "STATE", "NAME", "UPDATED AT"], &rows)
    )
}

pub fn render_pr_activity_table(values: &[Value]) -> String {
    let rows = values
        .iter()
        .map(|value| {
            vec![
                pr_activity_type(value),
                first_string_field(
                    value,
                    &[
                        &["user", "display_name"],
                        &["approval", "user", "display_name"],
                        &["comment", "user", "display_name"],
                        &["update", "author", "display_name"],
                    ],
                )
                .unwrap_or("-")
                .to_string(),
                relative_time_label(
                    first_string_field(
                        value,
                        &[
                            &["created_on"],
                            &["comment", "created_on"],
                            &["approval", "date"],
                            &["update", "date"],
                        ],
                    )
                    .unwrap_or_default(),
                ),
                compact_text(pr_activity_detail(value).as_str(), 60),
            ]
        })
        .collect::<Vec<_>>();
    format!(
        "{}\n",
        render_table(&["TYPE", "USER", "CREATED AT", "DETAIL"], &rows)
    )
}

pub fn render_pipeline_table(values: &[Value]) -> String {
    let rows = values
        .iter()
        .map(|value| {
            vec![
                string_field(value, &["uuid"]).unwrap_or("-").to_string(),
                pipeline_state_label(value),
                string_field(value, &["target", "ref_name"])
                    .unwrap_or("-")
                    .to_string(),
            ]
        })
        .collect::<Vec<_>>();
    format!("{}\n", render_table(&["UUID", "STATE", "REF"], &rows))
}

pub fn render_issue_table(values: &[Value]) -> String {
    let rows = values
        .iter()
        .map(|value| {
            vec![
                int_field(value, &["id"]).unwrap_or_default().to_string(),
                string_field(value, &["state"]).unwrap_or("-").to_string(),
                string_field(value, &["kind"]).unwrap_or("-").to_string(),
                string_field(value, &["priority"])
                    .unwrap_or("-")
                    .to_string(),
                string_field(value, &["title"]).unwrap_or("-").to_string(),
            ]
        })
        .collect::<Vec<_>>();
    format!(
        "{}\n",
        render_table(&["ID", "STATE", "KIND", "PRIORITY", "TITLE"], &rows)
    )
}

pub fn render_wiki_table(rows: &[WikiPage]) -> String {
    let table_rows = rows
        .iter()
        .map(|row| vec![row.path.clone(), row.size.to_string()])
        .collect::<Vec<_>>();
    format!("{}\n", render_table(&["PATH", "SIZE"], &table_rows))
}

pub fn string_field<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

pub fn int_field(value: &Value, path: &[&str]) -> Option<i64> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_i64()
}

pub fn pipeline_state_label(value: &Value) -> String {
    string_field(value, &["state", "result", "name"])
        .or_else(|| string_field(value, &["state", "name"]))
        .unwrap_or("-")
        .to_string()
}

fn first_string_field<'a>(value: &'a Value, paths: &[&[&str]]) -> Option<&'a str> {
    paths.iter().find_map(|path| string_field(value, path))
}

fn compact_text(value: &str, limit: usize) -> String {
    let compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.is_empty() {
        return "-".to_string();
    }
    let mut chars = compact.chars();
    let truncated = chars.by_ref().take(limit).collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        compact
    }
}

fn pr_activity_type(value: &Value) -> String {
    if value.get("approval").is_some() {
        return "approval".to_string();
    }
    if value.get("request_changes").is_some() || value.get("changes_request").is_some() {
        return "request_changes".to_string();
    }
    if value.get("comment").is_some() {
        return "comment".to_string();
    }
    if value.get("update").is_some() {
        return "update".to_string();
    }
    if value.get("merge").is_some() {
        return "merge".to_string();
    }
    if value.get("decline").is_some() {
        return "decline".to_string();
    }
    if value.get("task").is_some() {
        return "task".to_string();
    }
    string_field(value, &["type"]).unwrap_or("-").to_string()
}

fn pr_activity_detail(value: &Value) -> String {
    first_string_field(
        value,
        &[
            &["comment", "content", "raw"],
            &["task", "content", "raw"],
            &["update", "description"],
            &["update", "title"],
            &["decline", "reason"],
            &["merge", "commit", "message"],
            &["approval", "date"],
        ],
    )
    .unwrap_or("-")
    .to_string()
}

pub fn relative_time_label(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "-".to_string();
    }
    let Ok(created_at) = OffsetDateTime::parse(trimmed, &Rfc3339) else {
        return trimmed.to_string();
    };
    humanize_since(created_at, OffsetDateTime::now_utc())
}

fn humanize_since(created_at: OffsetDateTime, now: OffsetDateTime) -> String {
    if now < created_at {
        return "just now".to_string();
    }
    let seconds = (now - created_at).whole_seconds();
    if seconds < 60 {
        return "just now".to_string();
    }
    if seconds < 3_600 {
        let minutes = seconds / 60;
        if minutes == 1 {
            return "about 1 minute ago".to_string();
        }
        return format!("about {minutes} minutes ago");
    }
    if seconds < 86_400 {
        let hours = seconds / 3_600;
        if hours == 1 {
            return "about 1 hour ago".to_string();
        }
        return format!("about {hours} hours ago");
    }
    if seconds < 2_592_000 {
        let days = seconds / 86_400;
        if days == 1 {
            return "about 1 day ago".to_string();
        }
        return format!("about {days} days ago");
    }
    if seconds < 31_536_000 {
        let months = seconds / 2_592_000;
        if months <= 1 {
            return "about 1 month ago".to_string();
        }
        return format!("about {months} months ago");
    }
    let years = seconds / 31_536_000;
    if years <= 1 {
        return "about 1 year ago".to_string();
    }
    format!("about {years} years ago")
}

fn ansi(text: &str, code: &str, enabled: bool) -> String {
    if enabled {
        format!("\u{1b}[{code}m{text}\u{1b}[0m")
    } else {
        text.to_string()
    }
}

fn pad_right(value: &str, width: usize) -> String {
    let current = UnicodeWidthStr::width(value);
    if current >= width {
        return value.to_string();
    }
    format!("{value}{}", " ".repeat(width - current))
}

fn render_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut widths = headers
        .iter()
        .map(|header| UnicodeWidthStr::width(*header))
        .collect::<Vec<_>>();
    for row in rows {
        for (index, value) in row.iter().enumerate() {
            if let Some(width) = widths.get_mut(index) {
                *width = (*width).max(UnicodeWidthStr::width(value.as_str()));
            }
        }
    }
    let mut lines = Vec::with_capacity(rows.len() + 1);
    lines.push(
        headers
            .iter()
            .enumerate()
            .map(|(index, value)| pad_right(value, widths[index]))
            .collect::<Vec<_>>()
            .join("  "),
    );
    for row in rows {
        lines.push(
            row.iter()
                .enumerate()
                .map(|(index, value)| pad_right(value, widths[index]))
                .collect::<Vec<_>>()
                .join("  "),
        );
    }
    lines.join("\n")
}
