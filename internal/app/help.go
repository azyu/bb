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
	fmt.Fprintln(w, "  list     List pull requests")
	fmt.Fprintln(w, "  create   Create a pull request")
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
			{"profile", "Profile name override", ""},
			{"output", "Output format: text|json", "(default \"text\")"},
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
