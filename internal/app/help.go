package app

import (
	"fmt"
	"io"
)

func isHelpArg(s string) bool {
	return s == "-h" || s == "--help" || s == "help"
}

func printCmdHelp(w io.Writer, name, desc, usage string, flags [][3]string) {
	fmt.Fprintf(w, "%s\n\n", desc)
	fmt.Fprintf(w, "Usage:\n  %s\n", usage)
	if len(flags) > 0 {
		fmt.Fprintln(w, "\nFlags:")
		for _, f := range flags {
			fmt.Fprintf(w, "  --%-16s %s %s\n", f[0], f[1], f[2])
		}
	}
}

// ---------- group usage printers ----------

func printAuthUsage(w io.Writer) {
	fmt.Fprintln(w, "Authenticate and inspect auth status")
	fmt.Fprintln(w, "")
	fmt.Fprintln(w, "Usage:")
	fmt.Fprintln(w, "  bb auth <command>")
	fmt.Fprintln(w, "")
	fmt.Fprintln(w, "Commands:")
	fmt.Fprintln(w, "  login    Authenticate with Bitbucket")
	fmt.Fprintln(w, "  status   Show current auth status")
	fmt.Fprintln(w, "  logout   Remove stored credentials")
}

func printRepoUsage(w io.Writer) {
	fmt.Fprintln(w, "Repository operations")
	fmt.Fprintln(w, "")
	fmt.Fprintln(w, "Usage:")
	fmt.Fprintln(w, "  bb repo <command>")
	fmt.Fprintln(w, "")
	fmt.Fprintln(w, "Commands:")
	fmt.Fprintln(w, "  list   List repositories in a workspace")
}

func printPRUsage(w io.Writer) {
	fmt.Fprintln(w, "Pull request operations")
	fmt.Fprintln(w, "")
	fmt.Fprintln(w, "Usage:")
	fmt.Fprintln(w, "  bb pr <command>")
	fmt.Fprintln(w, "")
	fmt.Fprintln(w, "Commands:")
	fmt.Fprintln(w, "  list              List pull requests")
	fmt.Fprintln(w, "  create            Create a pull request")
	fmt.Fprintln(w, "  merge             Merge a pull request")
	fmt.Fprintln(w, "  view              View pull request details")
	fmt.Fprintln(w, "  edit              Update a pull request")
	fmt.Fprintln(w, "  approve           Approve a pull request")
	fmt.Fprintln(w, "  decline           Decline a pull request")
	fmt.Fprintln(w, "  comment           Add a comment to a pull request")
	fmt.Fprintln(w, "  diff              View pull request diff")
	fmt.Fprintln(w, "  statuses          Show build/CI statuses")
	fmt.Fprintln(w, "  unapprove         Remove approval from a pull request")
	fmt.Fprintln(w, "  request-changes   Request changes on a pull request")
	fmt.Fprintln(w, "  checkout          Check out PR source branch locally")
	fmt.Fprintln(w, "  activity          View pull request activity feed")
	fmt.Fprintln(w, "  comments          List pull request comments")
}

func printPipelineUsage(w io.Writer) {
	fmt.Fprintln(w, "Pipeline operations")
	fmt.Fprintln(w, "")
	fmt.Fprintln(w, "Usage:")
	fmt.Fprintln(w, "  bb pipeline <command>")
	fmt.Fprintln(w, "")
	fmt.Fprintln(w, "Commands:")
	fmt.Fprintln(w, "  list   List pipelines")
	fmt.Fprintln(w, "  run    Trigger a pipeline")
}

func printIssueUsage(w io.Writer) {
	fmt.Fprintln(w, "Issue operations")
	fmt.Fprintln(w, "")
	fmt.Fprintln(w, "Usage:")
	fmt.Fprintln(w, "  bb issue <command>")
	fmt.Fprintln(w, "")
	fmt.Fprintln(w, "Commands:")
	fmt.Fprintln(w, "  list     List issues")
	fmt.Fprintln(w, "  create   Create an issue")
	fmt.Fprintln(w, "  update   Update an issue")
}

func printWikiUsage(w io.Writer) {
	fmt.Fprintln(w, "Wiki operations")
	fmt.Fprintln(w, "")
	fmt.Fprintln(w, "Usage:")
	fmt.Fprintln(w, "  bb wiki <command>")
	fmt.Fprintln(w, "")
	fmt.Fprintln(w, "Commands:")
	fmt.Fprintln(w, "  list   List wiki pages")
	fmt.Fprintln(w, "  get    Get wiki page content")
	fmt.Fprintln(w, "  put    Create or update a wiki page")
}

func printCompletionUsage(w io.Writer) {
	fmt.Fprintln(w, "Generate shell completion scripts")
	fmt.Fprintln(w, "")
	fmt.Fprintln(w, "Usage:")
	fmt.Fprintln(w, "  bb completion <shell>")
	fmt.Fprintln(w, "")
	fmt.Fprintln(w, "Shells:")
	fmt.Fprintln(w, "  bash         Bash completion")
	fmt.Fprintln(w, "  zsh          Zsh completion")
	fmt.Fprintln(w, "  fish         Fish completion")
	fmt.Fprintln(w, "  powershell   PowerShell completion")
}

// ---------- leaf help printers ----------

func printAuthLoginHelp(w io.Writer) {
	printCmdHelp(w, "auth login",
		"Authenticate with Bitbucket",
		"bb auth login [flags]",
		[][3]string{
			{"profile", "Profile name", "(default \"default\")"},
			{"token", "API token value", ""},
			{"username", "Bitbucket username/email for Basic auth", ""},
			{"with-token", "Read API token from stdin", ""},
			{"base-url", "Bitbucket API base URL", ""},
		})
}

func printAuthStatusHelp(w io.Writer) {
	printCmdHelp(w, "auth status",
		"Show current auth status",
		"bb auth status [flags]",
		[][3]string{
			{"profile", "Profile name override", ""},
		})
}

func printAuthLogoutHelp(w io.Writer) {
	printCmdHelp(w, "auth logout",
		"Remove stored credentials",
		"bb auth logout [flags]",
		[][3]string{
			{"profile", "Profile name override", ""},
		})
}

func printAPIHelp(w io.Writer) {
	printCmdHelp(w, "api",
		"Call Bitbucket Cloud REST endpoints",
		"bb api [flags] <endpoint>",
		[][3]string{
			{"method", "HTTP method", "(default \"GET\")"},
			{"paginate", "Follow pagination", ""},
			{"profile", "Profile name override", ""},
			{"q", "Bitbucket q filter", ""},
			{"sort", "Sort expression", ""},
			{"fields", "Partial fields selector", ""},
		})
}

func printRepoListHelp(w io.Writer) {
	printCmdHelp(w, "repo list",
		"List repositories in a workspace",
		"bb repo list [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"output", "Output format: table|json", "(default \"table\")"},
			{"all", "Fetch all pages", ""},
			{"profile", "Profile name override", ""},
			{"q", "Bitbucket q filter", ""},
			{"sort", "Sort expression", ""},
			{"fields", "Partial fields selector", ""},
		})
}

func printPRListHelp(w io.Writer) {
	printCmdHelp(w, "pr list",
		"List pull requests",
		"bb pr list [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"output", "Output format: table|json", "(default \"table\")"},
			{"all", "Fetch all pages", ""},
			{"profile", "Profile name override", ""},
			{"state", "State filter (OPEN|MERGED|DECLINED)", ""},
			{"q", "Bitbucket q filter", ""},
			{"sort", "Sort expression", ""},
			{"fields", "Partial fields selector", ""},
		})
}

func printPRCreateHelp(w io.Writer) {
	printCmdHelp(w, "pr create",
		"Create a pull request",
		"bb pr create [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"title", "Pull request title", "(required)"},
			{"source", "Source branch name", "(required)"},
			{"destination", "Destination branch name", "(required)"},
			{"description", "Pull request description", ""},
			{"close-branch", "Delete source branch after merge", ""},
			{"profile", "Profile name override", ""},
			{"output", "Output format: text|json", "(default \"text\")"},
		})
}

func printPRMergeHelp(w io.Writer) {
	printCmdHelp(w, "pr merge",
		"Merge a pull request",
		"bb pr merge [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"id", "Pull request ID", "(required)"},
			{"message", "Merge commit message", ""},
			{"strategy", "Merge strategy: merge_commit|squash|fast_forward", ""},
			{"close-branch", "Delete source branch after merge", ""},
			{"profile", "Profile name override", ""},
			{"output", "Output format: text|json", "(default \"text\")"},
		})
}

func printPRViewHelp(w io.Writer) {
	printCmdHelp(w, "pr view",
		"View pull request details",
		"bb pr view [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"id", "Pull request ID", "(required)"},
			{"profile", "Profile name override", ""},
			{"output", "Output format: text|json", "(default \"text\")"},
		})
}

func printPREditHelp(w io.Writer) {
	printCmdHelp(w, "pr edit",
		"Update a pull request",
		"bb pr edit [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"id", "Pull request ID", "(required)"},
			{"title", "Pull request title", ""},
			{"description", "Pull request description", ""},
			{"close-branch", "Delete source branch after merge", ""},
			{"profile", "Profile name override", ""},
			{"output", "Output format: text|json", "(default \"text\")"},
		})
}

func printPRApproveHelp(w io.Writer) {
	printCmdHelp(w, "pr approve",
		"Approve a pull request",
		"bb pr approve [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"id", "Pull request ID", "(required)"},
			{"profile", "Profile name override", ""},
			{"output", "Output format: text|json", "(default \"text\")"},
		})
}

func printPRDeclineHelp(w io.Writer) {
	printCmdHelp(w, "pr decline",
		"Decline a pull request",
		"bb pr decline [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"id", "Pull request ID", "(required)"},
			{"profile", "Profile name override", ""},
			{"output", "Output format: text|json", "(default \"text\")"},
		})
}

func printPRCommentHelp(w io.Writer) {
	printCmdHelp(w, "pr comment",
		"Add a comment to a pull request",
		"bb pr comment [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"id", "Pull request ID", "(required)"},
			{"body", "Comment body text", "(required)"},
			{"profile", "Profile name override", ""},
			{"output", "Output format: text|json", "(default \"text\")"},
		})
}

func printPRDiffHelp(w io.Writer) {
	printCmdHelp(w, "pr diff",
		"View pull request diff",
		"bb pr diff [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"id", "Pull request ID", "(required)"},
			{"profile", "Profile name override", ""},
		})
}

func printPRStatusesHelp(w io.Writer) {
	printCmdHelp(w, "pr statuses",
		"Show build/CI statuses for a pull request",
		"bb pr statuses [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"id", "Pull request ID", "(required)"},
			{"output", "Output format: table|json", "(default \"table\")"},
			{"all", "Fetch all pages", ""},
			{"profile", "Profile name override", ""},
		})
}

func printPRUnapproveHelp(w io.Writer) {
	printCmdHelp(w, "pr unapprove",
		"Remove approval from a pull request",
		"bb pr unapprove [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"id", "Pull request ID", "(required)"},
			{"profile", "Profile name override", ""},
			{"output", "Output format: text|json", "(default \"text\")"},
		})
}

func printPRRequestChangesHelp(w io.Writer) {
	printCmdHelp(w, "pr request-changes",
		"Request changes on a pull request",
		"bb pr request-changes [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"id", "Pull request ID", "(required)"},
			{"profile", "Profile name override", ""},
			{"output", "Output format: text|json", "(default \"text\")"},
		})
}

func printPRCheckoutHelp(w io.Writer) {
	printCmdHelp(w, "pr checkout",
		"Check out PR source branch locally",
		"bb pr checkout [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"id", "Pull request ID", "(required)"},
			{"profile", "Profile name override", ""},
		})
}

func printPRActivityHelp(w io.Writer) {
	printCmdHelp(w, "pr activity",
		"View pull request activity feed",
		"bb pr activity [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"id", "Pull request ID", "(required)"},
			{"output", "Output format: table|json", "(default \"table\")"},
			{"all", "Fetch all pages", ""},
			{"profile", "Profile name override", ""},
		})
}

func printPRCommentsHelp(w io.Writer) {
	printCmdHelp(w, "pr comments",
		"List pull request comments",
		"bb pr comments [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"id", "Pull request ID", "(required)"},
			{"output", "Output format: table|json", "(default \"table\")"},
			{"all", "Fetch all pages", ""},
			{"profile", "Profile name override", ""},
		})
}

func printPipelineListHelp(w io.Writer) {
	printCmdHelp(w, "pipeline list",
		"List pipelines",
		"bb pipeline list [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"output", "Output format: table|json", "(default \"table\")"},
			{"all", "Fetch all pages", ""},
			{"profile", "Profile name override", ""},
			{"sort", "Sort expression", ""},
			{"fields", "Partial fields selector", ""},
		})
}

func printPipelineRunHelp(w io.Writer) {
	printCmdHelp(w, "pipeline run",
		"Trigger a pipeline",
		"bb pipeline run [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"branch", "Target branch name", "(required)"},
			{"profile", "Profile name override", ""},
			{"output", "Output format: text|json", "(default \"text\")"},
		})
}

func printIssueListHelp(w io.Writer) {
	printCmdHelp(w, "issue list",
		"List issues",
		"bb issue list [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"output", "Output format: table|json", "(default \"table\")"},
			{"all", "Fetch all pages", ""},
			{"profile", "Profile name override", ""},
			{"q", "Bitbucket q filter", ""},
			{"sort", "Sort expression", ""},
			{"fields", "Partial fields selector", ""},
		})
}

func printIssueCreateHelp(w io.Writer) {
	printCmdHelp(w, "issue create",
		"Create an issue",
		"bb issue create [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"title", "Issue title", "(required)"},
			{"content", "Issue content (raw text)", ""},
			{"state", "Issue state", ""},
			{"kind", "Issue kind (bug|enhancement|proposal|task)", ""},
			{"priority", "Issue priority (trivial|minor|major|critical|blocker)", ""},
			{"profile", "Profile name override", ""},
			{"output", "Output format: text|json", "(default \"text\")"},
		})
}

func printIssueUpdateHelp(w io.Writer) {
	printCmdHelp(w, "issue update",
		"Update an issue",
		"bb issue update [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"id", "Issue id", "(required)"},
			{"title", "Issue title", ""},
			{"content", "Issue content (raw text)", ""},
			{"state", "Issue state", ""},
			{"kind", "Issue kind (bug|enhancement|proposal|task)", ""},
			{"priority", "Issue priority (trivial|minor|major|critical|blocker)", ""},
			{"profile", "Profile name override", ""},
			{"output", "Output format: text|json", "(default \"text\")"},
		})
}

func printWikiListHelp(w io.Writer) {
	printCmdHelp(w, "wiki list",
		"List wiki pages",
		"bb wiki list [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"profile", "Profile name override", ""},
			{"output", "Output format: table|json", "(default \"table\")"},
		})
}

func printWikiGetHelp(w io.Writer) {
	printCmdHelp(w, "wiki get",
		"Get wiki page content",
		"bb wiki get [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"page", "Wiki page path", "(required)"},
			{"profile", "Profile name override", ""},
			{"output", "Output format: text|json", "(default \"text\")"},
		})
}

func printWikiPutHelp(w io.Writer) {
	printCmdHelp(w, "wiki put",
		"Create or update a wiki page",
		"bb wiki put [flags]",
		[][3]string{
			{"workspace", "Workspace slug", ""},
			{"repo", "Repository slug", ""},
			{"page", "Wiki page path", "(required)"},
			{"content", "Wiki page content", ""},
			{"file", "Read content from file path", ""},
			{"message", "Git commit message", ""},
			{"profile", "Profile name override", ""},
			{"output", "Output format: text|json", "(default \"text\")"},
		})
}
