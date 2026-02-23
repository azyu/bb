package app

import (
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"sort"
	"strings"
	"text/tabwriter"

	"bitbucket-cli/internal/api"
	"bitbucket-cli/internal/config"
	"bitbucket-cli/internal/version"
)

func Run(args []string, stdout, stderr io.Writer) int {
	if len(args) == 0 {
		printRootUsage(stdout)
		return 0
	}

	switch args[0] {
	case "version", "--version", "-v":
		return runVersion(stdout)
	case "auth":
		return runAuth(args[1:], stdout, stderr)
	case "api":
		return runAPI(args[1:], stdout, stderr)
	case "repo":
		return runRepo(args[1:], stdout, stderr)
	case "pr":
		return runPR(args[1:], stdout, stderr)
	case "pipeline":
		return runPipeline(args[1:], stdout, stderr)
	case "wiki":
		return runWiki(args[1:], stdout, stderr)
	case "issue":
		return runIssue(args[1:], stdout, stderr)
	case "completion":
		return runCompletion(args[1:], stdout, stderr)
	case "-h", "--help", "help":
		printRootUsage(stdout)
		return 0
	default:
		fmt.Fprintf(stderr, "unknown command: %s\n\n", args[0])
		printRootUsage(stderr)
		return 1
	}
}

var wikiRemoteURLBuilder = buildWikiRemoteURL
var gitCommandRunner = runGitCommand

func runAuth(args []string, stdout, stderr io.Writer) int {
	if len(args) == 0 {
		fmt.Fprintln(stderr, "usage: bb auth <login|status|logout>")
		return 1
	}
	switch args[0] {
	case "login":
		return runAuthLogin(args[1:], stdout, stderr)
	case "status":
		return runAuthStatus(args[1:], stdout, stderr)
	case "logout":
		return runAuthLogout(args[1:], stdout, stderr)
	default:
		fmt.Fprintf(stderr, "unknown auth command: %s\n", args[0])
		return 1
	}
}

func runAuthLogin(args []string, stdout, stderr io.Writer) int {
	args = normalizeAuthLoginArgs(args)

	fs := flag.NewFlagSet("auth login", flag.ContinueOnError)
	fs.SetOutput(stderr)
	profile := fs.String("profile", "default", "profile name")
	token := fs.String("token", "", "API token value")
	username := fs.String("username", "", "Bitbucket username/email for Basic auth")
	withToken := fs.Bool("with-token", false, "read API token from stdin")
	baseURL := fs.String("base-url", "", "Bitbucket API base URL")
	if err := fs.Parse(args); err != nil {
		return 1
	}

	resolvedToken := strings.TrimSpace(*token)
	if resolvedToken == "" && *withToken {
		var err error
		resolvedToken, err = readTokenFromStdin()
		if err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
	}
	if resolvedToken == "" {
		resolvedToken = strings.TrimSpace(os.Getenv("BITBUCKET_TOKEN"))
	}
	if resolvedToken == "" {
		fmt.Fprintln(stderr, "token is required: use --token <value>, --with-token, or BITBUCKET_TOKEN")
		return 1
	}
	resolvedUsername := strings.TrimSpace(*username)
	if resolvedUsername == "" {
		resolvedUsername = strings.TrimSpace(os.Getenv("BITBUCKET_USERNAME"))
	}

	cfg, err := config.Load()
	if err != nil {
		fmt.Fprintf(stderr, "load config: %v\n", err)
		return 1
	}
	cfg.SetProfileWithAuth(*profile, resolvedUsername, resolvedToken, *baseURL)
	if err := cfg.Save(); err != nil {
		fmt.Fprintf(stderr, "save config: %v\n", err)
		return 1
	}

	fmt.Fprintf(stdout, "authenticated profile %q\n", *profile)
	if resolvedUsername != "" {
		fmt.Fprintf(stdout, "auth mode: basic (%s)\n", resolvedUsername)
	} else {
		fmt.Fprintln(stdout, "auth mode: bearer token")
	}
	return 0
}

func normalizeAuthLoginArgs(args []string) []string {
	out := make([]string, 0, len(args))
	for i := 0; i < len(args); i++ {
		arg := args[i]
		if arg == "--token" || arg == "-token" {
			if i+1 < len(args) && !strings.HasPrefix(args[i+1], "-") {
				out = append(out, arg, args[i+1])
				i++
				continue
			}
			out = append(out, "--with-token")
			continue
		}
		out = append(out, arg)
	}
	return out
}

func readTokenFromStdin() (string, error) {
	scanner := bufio.NewScanner(os.Stdin)
	if !scanner.Scan() {
		if err := scanner.Err(); err != nil {
			return "", fmt.Errorf("read token from stdin: %w", err)
		}
		return "", fmt.Errorf("no token provided on stdin")
	}
	token := strings.TrimSpace(scanner.Text())
	if token == "" {
		return "", fmt.Errorf("no token provided on stdin")
	}
	return token, nil
}

func runAuthStatus(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("auth status", flag.ContinueOnError)
	fs.SetOutput(stderr)
	profile := fs.String("profile", "", "profile name override")
	if err := fs.Parse(args); err != nil {
		return 1
	}

	cfg, err := config.Load()
	if err != nil {
		fmt.Fprintf(stderr, "load config: %v\n", err)
		return 1
	}
	p, name, err := cfg.ActiveProfile(*profile)
	if err != nil {
		fmt.Fprintln(stderr, "not logged in: run `bb auth login`")
		return 1
	}

	fmt.Fprintf(stdout, "Profile: %s\n", name)
	fmt.Fprintf(stdout, "Base URL: %s\n", p.BaseURL)
	if strings.TrimSpace(p.Username) != "" {
		fmt.Fprintf(stdout, "Auth: basic (%s)\n", p.Username)
	} else {
		fmt.Fprintln(stdout, "Auth: bearer token")
	}
	if strings.TrimSpace(p.Token) == "" {
		fmt.Fprintln(stdout, "Token: not configured")
	} else {
		fmt.Fprintln(stdout, "Token: configured")
	}
	return 0
}

func runAuthLogout(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("auth logout", flag.ContinueOnError)
	fs.SetOutput(stderr)
	profile := fs.String("profile", "", "profile name override")
	if err := fs.Parse(args); err != nil {
		return 1
	}

	cfg, err := config.Load()
	if err != nil {
		fmt.Fprintf(stderr, "load config: %v\n", err)
		return 1
	}

	target := strings.TrimSpace(*profile)
	if target == "" && strings.TrimSpace(cfg.Current) == "" {
		fmt.Fprintln(stderr, "not logged in: run `bb auth login`")
		return 1
	}

	removed, ok := cfg.RemoveProfile(target)
	if !ok {
		if strings.TrimSpace(removed) == "" {
			fmt.Fprintln(stderr, "not logged in: run `bb auth login`")
		} else {
			fmt.Fprintf(stderr, "profile %q not found\n", removed)
		}
		return 1
	}

	if err := cfg.Save(); err != nil {
		fmt.Fprintf(stderr, "save config: %v\n", err)
		return 1
	}

	fmt.Fprintf(stdout, "logged out profile %q\n", removed)
	if strings.TrimSpace(cfg.Current) != "" {
		fmt.Fprintf(stdout, "active profile: %q\n", cfg.Current)
	}
	return 0
}

func runAPI(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("api", flag.ContinueOnError)
	fs.SetOutput(stderr)
	method := fs.String("method", http.MethodGet, "HTTP method")
	paginate := fs.Bool("paginate", false, "follow pagination")
	profile := fs.String("profile", "", "profile name override")
	q := fs.String("q", "", "Bitbucket q filter")
	sort := fs.String("sort", "", "sort expression")
	fields := fs.String("fields", "", "partial fields selector")
	if err := fs.Parse(args); err != nil {
		return 1
	}

	remaining := fs.Args()
	if len(remaining) != 1 {
		fmt.Fprintln(stderr, "usage: bb api [flags] <endpoint>")
		return 1
	}
	endpoint := remaining[0]

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	query := url.Values{}
	if strings.TrimSpace(*q) != "" {
		query.Set("q", *q)
	}
	if strings.TrimSpace(*sort) != "" {
		query.Set("sort", *sort)
	}
	if strings.TrimSpace(*fields) != "" {
		query.Set("fields", *fields)
	}

	ctx := context.Background()
	if *paginate {
		values, err := client.GetAllValues(ctx, endpoint, query)
		if err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
		return printJSON(stdout, values, stderr)
	}

	var out any
	if err := client.DoJSON(ctx, strings.ToUpper(*method), endpoint, query, nil, &out); err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}
	return printJSON(stdout, out, stderr)
}

func runRepo(args []string, stdout, stderr io.Writer) int {
	if len(args) == 0 {
		fmt.Fprintln(stderr, "usage: bb repo <list>")
		return 1
	}
	switch args[0] {
	case "list":
		return runRepoList(args[1:], stdout, stderr)
	default:
		fmt.Fprintf(stderr, "unknown repo command: %s\n", args[0])
		return 1
	}
}

func runRepoList(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("repo list", flag.ContinueOnError)
	fs.SetOutput(stderr)
	workspace := fs.String("workspace", "", "workspace slug")
	output := fs.String("output", "table", "output format: table|json")
	all := fs.Bool("all", false, "fetch all pages")
	profile := fs.String("profile", "", "profile name override")
	q := fs.String("q", "", "Bitbucket q filter")
	sort := fs.String("sort", "", "sort expression")
	fields := fs.String("fields", "", "partial fields selector")
	if err := fs.Parse(args); err != nil {
		return 1
	}
	workspaceSlug, _, err := resolveRepoTarget(*workspace, "", false)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	query := url.Values{}
	if strings.TrimSpace(*q) != "" {
		query.Set("q", *q)
	}
	if strings.TrimSpace(*sort) != "" {
		query.Set("sort", *sort)
	}
	if strings.TrimSpace(*fields) != "" {
		query.Set("fields", *fields)
	}

	path := fmt.Sprintf("/repositories/%s", workspaceSlug)
	var values []json.RawMessage
	if *all {
		values, err = client.GetAllValues(context.Background(), path, query)
		if err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
	} else {
		var page struct {
			Values []json.RawMessage `json:"values"`
		}
		if err := client.DoJSON(context.Background(), http.MethodGet, path, query, nil, &page); err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
		values = page.Values
	}

	switch *output {
	case "json":
		return printJSON(stdout, values, stderr)
	case "table":
		return printRepoTable(stdout, values, stderr)
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

func runPR(args []string, stdout, stderr io.Writer) int {
	if len(args) == 0 {
		fmt.Fprintln(stderr, "usage: bb pr <list|create>")
		return 1
	}
	switch args[0] {
	case "list":
		return runPRList(args[1:], stdout, stderr)
	case "create":
		return runPRCreate(args[1:], stdout, stderr)
	default:
		fmt.Fprintf(stderr, "unknown pr command: %s\n", args[0])
		return 1
	}
}

func runPRList(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pr list", flag.ContinueOnError)
	fs.SetOutput(stderr)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	output := fs.String("output", "table", "output format: table|json")
	all := fs.Bool("all", false, "fetch all pages")
	profile := fs.String("profile", "", "profile name override")
	state := fs.String("state", "", "pull request state filter (OPEN|MERGED|DECLINED)")
	q := fs.String("q", "", "Bitbucket q filter")
	sort := fs.String("sort", "", "sort expression")
	fields := fs.String("fields", "", "partial fields selector")
	if err := fs.Parse(args); err != nil {
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	query := url.Values{}
	setQueryIfNotEmpty(query, "state", strings.ToUpper(strings.TrimSpace(*state)))
	setQueryIfNotEmpty(query, "q", *q)
	setQueryIfNotEmpty(query, "sort", *sort)
	setQueryIfNotEmpty(query, "fields", *fields)

	path := fmt.Sprintf("/repositories/%s/%s/pullrequests", workspaceSlug, repoSlug)
	var values []json.RawMessage
	if *all {
		values, err = client.GetAllValues(context.Background(), path, query)
		if err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
	} else {
		var page struct {
			Values []json.RawMessage `json:"values"`
		}
		if err := client.DoJSON(context.Background(), http.MethodGet, path, query, nil, &page); err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
		values = page.Values
	}

	switch *output {
	case "json":
		return printJSON(stdout, values, stderr)
	case "table":
		return printPRTable(stdout, values, stderr)
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

func runPRCreate(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pr create", flag.ContinueOnError)
	fs.SetOutput(stderr)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	title := fs.String("title", "", "pull request title")
	source := fs.String("source", "", "source branch name")
	destination := fs.String("destination", "", "destination branch name")
	description := fs.String("description", "", "pull request description")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "text", "output format: text|json")
	if err := fs.Parse(args); err != nil {
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	if strings.TrimSpace(*title) == "" {
		fmt.Fprintln(stderr, "--title is required")
		return 1
	}
	if strings.TrimSpace(*source) == "" {
		fmt.Fprintln(stderr, "--source is required")
		return 1
	}
	if strings.TrimSpace(*destination) == "" {
		fmt.Fprintln(stderr, "--destination is required")
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	body := map[string]any{
		"title": *title,
		"source": map[string]any{
			"branch": map[string]any{
				"name": *source,
			},
		},
		"destination": map[string]any{
			"branch": map[string]any{
				"name": *destination,
			},
		},
	}
	if strings.TrimSpace(*description) != "" {
		body["description"] = *description
	}
	payload, err := json.Marshal(body)
	if err != nil {
		fmt.Fprintf(stderr, "encode request body: %v\n", err)
		return 1
	}

	path := fmt.Sprintf("/repositories/%s/%s/pullrequests", workspaceSlug, repoSlug)
	var created pullRequestRow
	if err := client.DoJSON(context.Background(), http.MethodPost, path, nil, bytes.NewReader(payload), &created); err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	switch *output {
	case "json":
		return printJSON(stdout, created, stderr)
	case "text":
		fmt.Fprintf(stdout, "Created PR #%d (%s): %s\n", created.ID, created.State, created.Title)
		if strings.TrimSpace(created.Links.HTML.Href) != "" {
			fmt.Fprintf(stdout, "URL: %s\n", created.Links.HTML.Href)
		}
		return 0
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

func runPipeline(args []string, stdout, stderr io.Writer) int {
	if len(args) == 0 {
		fmt.Fprintln(stderr, "usage: bb pipeline <list|run>")
		return 1
	}
	switch args[0] {
	case "list":
		return runPipelineList(args[1:], stdout, stderr)
	case "run":
		return runPipelineRun(args[1:], stdout, stderr)
	default:
		fmt.Fprintf(stderr, "unknown pipeline command: %s\n", args[0])
		return 1
	}
}

func runPipelineList(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pipeline list", flag.ContinueOnError)
	fs.SetOutput(stderr)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	output := fs.String("output", "table", "output format: table|json")
	all := fs.Bool("all", false, "fetch all pages")
	profile := fs.String("profile", "", "profile name override")
	sort := fs.String("sort", "", "sort expression")
	fields := fs.String("fields", "", "partial fields selector")
	if err := fs.Parse(args); err != nil {
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	query := url.Values{}
	setQueryIfNotEmpty(query, "sort", *sort)
	setQueryIfNotEmpty(query, "fields", *fields)

	path := fmt.Sprintf("/repositories/%s/%s/pipelines", workspaceSlug, repoSlug)
	var values []json.RawMessage
	if *all {
		values, err = client.GetAllValues(context.Background(), path, query)
		if err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
	} else {
		var page struct {
			Values []json.RawMessage `json:"values"`
		}
		if err := client.DoJSON(context.Background(), http.MethodGet, path, query, nil, &page); err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
		values = page.Values
	}

	switch *output {
	case "json":
		return printJSON(stdout, values, stderr)
	case "table":
		return printPipelineTable(stdout, values, stderr)
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

func runPipelineRun(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pipeline run", flag.ContinueOnError)
	fs.SetOutput(stderr)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	branch := fs.String("branch", "", "target branch name")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "text", "output format: text|json")
	if err := fs.Parse(args); err != nil {
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	if strings.TrimSpace(*branch) == "" {
		fmt.Fprintln(stderr, "--branch is required")
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	body := map[string]any{
		"target": map[string]any{
			"type":     "pipeline_ref_target",
			"ref_type": "branch",
			"ref_name": *branch,
		},
	}
	payload, err := json.Marshal(body)
	if err != nil {
		fmt.Fprintf(stderr, "encode request body: %v\n", err)
		return 1
	}

	path := fmt.Sprintf("/repositories/%s/%s/pipelines", workspaceSlug, repoSlug)
	var triggered pipelineRow
	if err := client.DoJSON(context.Background(), http.MethodPost, path, nil, bytes.NewReader(payload), &triggered); err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	switch *output {
	case "json":
		return printJSON(stdout, triggered, stderr)
	case "text":
		fmt.Fprintf(stdout, "Triggered pipeline %s\n", triggered.UUID)
		fmt.Fprintf(stdout, "State: %s\n", pipelineStateLabel(triggered))
		if strings.TrimSpace(triggered.Target.RefName) != "" {
			fmt.Fprintf(stdout, "Ref: %s\n", triggered.Target.RefName)
		}
		return 0
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

func runIssue(args []string, stdout, stderr io.Writer) int {
	if len(args) == 0 {
		fmt.Fprintln(stderr, "usage: bb issue <list|create|update>")
		return 1
	}
	switch args[0] {
	case "list":
		return runIssueList(args[1:], stdout, stderr)
	case "create":
		return runIssueCreate(args[1:], stdout, stderr)
	case "update":
		return runIssueUpdate(args[1:], stdout, stderr)
	default:
		fmt.Fprintf(stderr, "unknown issue command: %s\n", args[0])
		return 1
	}
}

func runIssueList(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("issue list", flag.ContinueOnError)
	fs.SetOutput(stderr)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	output := fs.String("output", "table", "output format: table|json")
	all := fs.Bool("all", false, "fetch all pages")
	profile := fs.String("profile", "", "profile name override")
	q := fs.String("q", "", "Bitbucket q filter")
	sort := fs.String("sort", "", "sort expression")
	fields := fs.String("fields", "", "partial fields selector")
	if err := fs.Parse(args); err != nil {
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	query := url.Values{}
	setQueryIfNotEmpty(query, "q", *q)
	setQueryIfNotEmpty(query, "sort", *sort)
	setQueryIfNotEmpty(query, "fields", *fields)

	path := fmt.Sprintf("/repositories/%s/%s/issues", workspaceSlug, repoSlug)
	var values []json.RawMessage
	if *all {
		values, err = client.GetAllValues(context.Background(), path, query)
		if err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
	} else {
		var page struct {
			Values []json.RawMessage `json:"values"`
		}
		if err := client.DoJSON(context.Background(), http.MethodGet, path, query, nil, &page); err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
		values = page.Values
	}

	switch *output {
	case "json":
		return printJSON(stdout, values, stderr)
	case "table":
		return printIssueTable(stdout, values, stderr)
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

func runIssueCreate(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("issue create", flag.ContinueOnError)
	fs.SetOutput(stderr)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	title := fs.String("title", "", "issue title")
	content := fs.String("content", "", "issue content (raw text)")
	state := fs.String("state", "", "issue state")
	kind := fs.String("kind", "", "issue kind (bug|enhancement|proposal|task)")
	priority := fs.String("priority", "", "issue priority (trivial|minor|major|critical|blocker)")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "text", "output format: text|json")
	if err := fs.Parse(args); err != nil {
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	if strings.TrimSpace(*title) == "" {
		fmt.Fprintln(stderr, "--title is required")
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	body := map[string]any{
		"title": *title,
	}
	if trimmed := strings.TrimSpace(*content); trimmed != "" {
		body["content"] = map[string]any{
			"raw": trimmed,
		}
	}
	setOptionalIssueField(body, "state", *state)
	setOptionalIssueField(body, "kind", *kind)
	setOptionalIssueField(body, "priority", *priority)

	payload, err := json.Marshal(body)
	if err != nil {
		fmt.Fprintf(stderr, "encode request body: %v\n", err)
		return 1
	}

	path := fmt.Sprintf("/repositories/%s/%s/issues", workspaceSlug, repoSlug)
	var created issueRow
	if err := client.DoJSON(context.Background(), http.MethodPost, path, nil, bytes.NewReader(payload), &created); err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	switch *output {
	case "json":
		return printJSON(stdout, created, stderr)
	case "text":
		fmt.Fprintf(stdout, "Created issue #%d (%s): %s\n", created.ID, created.State, created.Title)
		if strings.TrimSpace(created.Links.HTML.Href) != "" {
			fmt.Fprintf(stdout, "URL: %s\n", created.Links.HTML.Href)
		}
		return 0
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

func runIssueUpdate(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("issue update", flag.ContinueOnError)
	fs.SetOutput(stderr)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	id := fs.Int("id", 0, "issue id")
	title := fs.String("title", "", "issue title")
	content := fs.String("content", "", "issue content (raw text)")
	state := fs.String("state", "", "issue state")
	kind := fs.String("kind", "", "issue kind (bug|enhancement|proposal|task)")
	priority := fs.String("priority", "", "issue priority (trivial|minor|major|critical|blocker)")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "text", "output format: text|json")
	if err := fs.Parse(args); err != nil {
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	if *id <= 0 {
		fmt.Fprintln(stderr, "--id is required")
		return 1
	}

	body := map[string]any{}
	setOptionalIssueField(body, "title", *title)
	setOptionalIssueField(body, "state", *state)
	setOptionalIssueField(body, "kind", *kind)
	setOptionalIssueField(body, "priority", *priority)
	if trimmed := strings.TrimSpace(*content); trimmed != "" {
		body["content"] = map[string]any{
			"raw": trimmed,
		}
	}
	if len(body) == 0 {
		fmt.Fprintln(stderr, "at least one field to update is required")
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	payload, err := json.Marshal(body)
	if err != nil {
		fmt.Fprintf(stderr, "encode request body: %v\n", err)
		return 1
	}

	path := fmt.Sprintf("/repositories/%s/%s/issues/%d", workspaceSlug, repoSlug, *id)
	var updated issueRow
	if err := client.DoJSON(context.Background(), http.MethodPut, path, nil, bytes.NewReader(payload), &updated); err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	switch *output {
	case "json":
		return printJSON(stdout, updated, stderr)
	case "text":
		fmt.Fprintf(stdout, "Updated issue #%d (%s): %s\n", updated.ID, updated.State, updated.Title)
		if strings.TrimSpace(updated.Links.HTML.Href) != "" {
			fmt.Fprintf(stdout, "URL: %s\n", updated.Links.HTML.Href)
		}
		return 0
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

func runWiki(args []string, stdout, stderr io.Writer) int {
	if len(args) == 0 {
		fmt.Fprintln(stderr, "usage: bb wiki <list|get|put>")
		return 1
	}
	switch args[0] {
	case "list":
		return runWikiList(args[1:], stdout, stderr)
	case "get":
		return runWikiGet(args[1:], stdout, stderr)
	case "put":
		return runWikiPut(args[1:], stdout, stderr)
	default:
		fmt.Fprintf(stderr, "unknown wiki command: %s\n", args[0])
		return 1
	}
}

func runWikiList(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("wiki list", flag.ContinueOnError)
	fs.SetOutput(stderr)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "table", "output format: table|json")
	if err := fs.Parse(args); err != nil {
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}

	p, err := profileFromConfig(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	ctx := context.Background()
	repoDir, err := cloneWikiToTemp(ctx, p, workspaceSlug, repoSlug)
	if err != nil {
		fmt.Fprintf(stderr, "%s\n", redactToken(err.Error(), p.Token))
		return 1
	}
	defer os.RemoveAll(repoDir)

	rows, err := listWikiPages(repoDir)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	switch *output {
	case "json":
		return printJSON(stdout, rows, stderr)
	case "table":
		return printWikiTable(stdout, rows, stderr)
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

func runWikiGet(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("wiki get", flag.ContinueOnError)
	fs.SetOutput(stderr)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	page := fs.String("page", "", "wiki page path")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "text", "output format: text|json")
	if err := fs.Parse(args); err != nil {
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	cleanPage, err := normalizeWikiPagePath(*page)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	p, err := profileFromConfig(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	ctx := context.Background()
	repoDir, err := cloneWikiToTemp(ctx, p, workspaceSlug, repoSlug)
	if err != nil {
		fmt.Fprintf(stderr, "%s\n", redactToken(err.Error(), p.Token))
		return 1
	}
	defer os.RemoveAll(repoDir)

	absPath := filepath.Join(repoDir, filepath.FromSlash(cleanPage))
	raw, err := os.ReadFile(absPath)
	if err != nil {
		if os.IsNotExist(err) {
			fmt.Fprintf(stderr, "wiki page not found: %s\n", cleanPage)
			return 1
		}
		fmt.Fprintf(stderr, "read wiki page: %v\n", err)
		return 1
	}

	switch *output {
	case "text":
		fmt.Fprint(stdout, string(raw))
		return 0
	case "json":
		return printJSON(stdout, map[string]any{
			"page":    cleanPage,
			"content": string(raw),
		}, stderr)
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

func runWikiPut(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("wiki put", flag.ContinueOnError)
	fs.SetOutput(stderr)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	page := fs.String("page", "", "wiki page path")
	content := fs.String("content", "", "wiki page content")
	fileInput := fs.String("file", "", "read wiki page content from file path")
	message := fs.String("message", "", "git commit message")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "text", "output format: text|json")
	if err := fs.Parse(args); err != nil {
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	cleanPage, err := normalizeWikiPagePath(*page)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}
	if strings.TrimSpace(*content) == "" && strings.TrimSpace(*fileInput) == "" {
		fmt.Fprintln(stderr, "either --content or --file is required")
		return 1
	}
	if strings.TrimSpace(*content) != "" && strings.TrimSpace(*fileInput) != "" {
		fmt.Fprintln(stderr, "use only one of --content or --file")
		return 1
	}

	var pageContent string
	if strings.TrimSpace(*fileInput) != "" {
		raw, err := os.ReadFile(strings.TrimSpace(*fileInput))
		if err != nil {
			fmt.Fprintf(stderr, "read --file: %v\n", err)
			return 1
		}
		pageContent = string(raw)
	} else {
		pageContent = *content
	}

	p, err := profileFromConfig(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	ctx := context.Background()
	repoDir, err := cloneWikiToTemp(ctx, p, workspaceSlug, repoSlug)
	if err != nil {
		fmt.Fprintf(stderr, "%s\n", redactToken(err.Error(), p.Token))
		return 1
	}
	defer os.RemoveAll(repoDir)

	absPath := filepath.Join(repoDir, filepath.FromSlash(cleanPage))
	if err := os.MkdirAll(filepath.Dir(absPath), 0o755); err != nil {
		fmt.Fprintf(stderr, "create wiki page directory: %v\n", err)
		return 1
	}
	if err := os.WriteFile(absPath, []byte(pageContent), 0o644); err != nil {
		fmt.Fprintf(stderr, "write wiki page: %v\n", err)
		return 1
	}

	pageRelPath := filepath.ToSlash(filepath.FromSlash(cleanPage))
	if _, err := gitCommandRunner(ctx, repoDir, "add", "--", pageRelPath); err != nil {
		fmt.Fprintf(stderr, "%s\n", redactToken(err.Error(), p.Token))
		return 1
	}
	statusOut, err := gitCommandRunner(ctx, repoDir, "status", "--porcelain", "--", pageRelPath)
	if err != nil {
		fmt.Fprintf(stderr, "%s\n", redactToken(err.Error(), p.Token))
		return 1
	}
	if strings.TrimSpace(string(statusOut)) == "" {
		switch *output {
		case "json":
			return printJSON(stdout, map[string]any{
				"page":   cleanPage,
				"status": "no_change",
			}, stderr)
		case "text":
			fmt.Fprintf(stdout, "No changes for wiki page: %s\n", cleanPage)
			return 0
		default:
			fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
			return 1
		}
	}

	commitMsg := strings.TrimSpace(*message)
	if commitMsg == "" {
		commitMsg = fmt.Sprintf("Update wiki page %s", cleanPage)
	}

	commitEmail := "bb-cli@local"
	commitName := "bb-cli"
	if strings.Contains(p.Username, "@") {
		commitEmail = p.Username
		commitName = strings.SplitN(p.Username, "@", 2)[0]
	}
	if _, err := gitCommandRunner(ctx, repoDir, "config", "user.name", commitName); err != nil {
		fmt.Fprintf(stderr, "%s\n", redactToken(err.Error(), p.Token))
		return 1
	}
	if _, err := gitCommandRunner(ctx, repoDir, "config", "user.email", commitEmail); err != nil {
		fmt.Fprintf(stderr, "%s\n", redactToken(err.Error(), p.Token))
		return 1
	}
	if _, err := gitCommandRunner(ctx, repoDir, "commit", "-m", commitMsg); err != nil {
		fmt.Fprintf(stderr, "%s\n", redactToken(err.Error(), p.Token))
		return 1
	}
	if _, err := gitCommandRunner(ctx, repoDir, "push", "origin", "HEAD"); err != nil {
		fmt.Fprintf(stderr, "%s\n", redactToken(err.Error(), p.Token))
		return 1
	}

	switch *output {
	case "json":
		return printJSON(stdout, map[string]any{
			"page":   cleanPage,
			"status": "updated",
		}, stderr)
	case "text":
		fmt.Fprintf(stdout, "Updated wiki page: %s\n", cleanPage)
		return 0
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

func runCompletion(args []string, stdout, stderr io.Writer) int {
	if len(args) != 1 {
		fmt.Fprintln(stderr, "usage: bb completion <bash|zsh|fish|powershell>")
		return 1
	}
	switch strings.ToLower(strings.TrimSpace(args[0])) {
	case "bash":
		fmt.Fprintln(stdout, bashCompletionScript)
		return 0
	case "zsh":
		fmt.Fprintln(stdout, zshCompletionScript)
		return 0
	case "fish":
		fmt.Fprintln(stdout, fishCompletionScript)
		return 0
	case "powershell":
		fmt.Fprintln(stdout, powershellCompletionScript)
		return 0
	default:
		fmt.Fprintf(stderr, "unsupported shell: %s\n", args[0])
		return 1
	}
}

type pullRequestRow struct {
	ID    int    `json:"id"`
	Title string `json:"title"`
	State string `json:"state"`
	Links struct {
		HTML struct {
			Href string `json:"href"`
		} `json:"html"`
	} `json:"links"`
	Source struct {
		Branch struct {
			Name string `json:"name"`
		} `json:"branch"`
	} `json:"source"`
	Destination struct {
		Branch struct {
			Name string `json:"name"`
		} `json:"branch"`
	} `json:"destination"`
}

type pipelineRow struct {
	UUID  string `json:"uuid"`
	State struct {
		Name   string `json:"name"`
		Result struct {
			Name string `json:"name"`
		} `json:"result"`
	} `json:"state"`
	Target struct {
		RefName string `json:"ref_name"`
	} `json:"target"`
}

type issueRow struct {
	ID       int    `json:"id"`
	Title    string `json:"title"`
	State    string `json:"state"`
	Kind     string `json:"kind"`
	Priority string `json:"priority"`
	Links    struct {
		HTML struct {
			Href string `json:"href"`
		} `json:"html"`
	} `json:"links"`
}

type wikiPageRow struct {
	Path string `json:"path"`
	Size int64  `json:"size"`
}

func printRepoTable(stdout io.Writer, values []json.RawMessage, stderr io.Writer) int {
	type repoRow struct {
		Slug     string `json:"slug"`
		FullName string `json:"full_name"`
	}

	tw := tabwriter.NewWriter(stdout, 0, 0, 2, ' ', 0)
	fmt.Fprintln(tw, "SLUG\tFULL_NAME")
	for _, raw := range values {
		var row repoRow
		if err := json.Unmarshal(raw, &row); err != nil {
			fmt.Fprintf(stderr, "decode repo row: %v\n", err)
			return 1
		}
		fmt.Fprintf(tw, "%s\t%s\n", row.Slug, row.FullName)
	}
	if err := tw.Flush(); err != nil {
		fmt.Fprintf(stderr, "flush table: %v\n", err)
		return 1
	}
	return 0
}

func printPRTable(stdout io.Writer, values []json.RawMessage, stderr io.Writer) int {
	tw := tabwriter.NewWriter(stdout, 0, 0, 2, ' ', 0)
	fmt.Fprintln(tw, "ID\tSTATE\tSOURCE\tDEST\tTITLE")
	for _, raw := range values {
		var row pullRequestRow
		if err := json.Unmarshal(raw, &row); err != nil {
			fmt.Fprintf(stderr, "decode pull request row: %v\n", err)
			return 1
		}
		fmt.Fprintf(
			tw,
			"%d\t%s\t%s\t%s\t%s\n",
			row.ID,
			row.State,
			row.Source.Branch.Name,
			row.Destination.Branch.Name,
			row.Title,
		)
	}
	if err := tw.Flush(); err != nil {
		fmt.Fprintf(stderr, "flush table: %v\n", err)
		return 1
	}
	return 0
}

func printPipelineTable(stdout io.Writer, values []json.RawMessage, stderr io.Writer) int {
	tw := tabwriter.NewWriter(stdout, 0, 0, 2, ' ', 0)
	fmt.Fprintln(tw, "UUID\tSTATE\tREF")
	for _, raw := range values {
		var row pipelineRow
		if err := json.Unmarshal(raw, &row); err != nil {
			fmt.Fprintf(stderr, "decode pipeline row: %v\n", err)
			return 1
		}
		fmt.Fprintf(tw, "%s\t%s\t%s\n", row.UUID, pipelineStateLabel(row), row.Target.RefName)
	}
	if err := tw.Flush(); err != nil {
		fmt.Fprintf(stderr, "flush table: %v\n", err)
		return 1
	}
	return 0
}

func printIssueTable(stdout io.Writer, values []json.RawMessage, stderr io.Writer) int {
	tw := tabwriter.NewWriter(stdout, 0, 0, 2, ' ', 0)
	fmt.Fprintln(tw, "ID\tSTATE\tKIND\tPRIORITY\tTITLE")
	for _, raw := range values {
		var row issueRow
		if err := json.Unmarshal(raw, &row); err != nil {
			fmt.Fprintf(stderr, "decode issue row: %v\n", err)
			return 1
		}
		fmt.Fprintf(tw, "%d\t%s\t%s\t%s\t%s\n", row.ID, row.State, row.Kind, row.Priority, row.Title)
	}
	if err := tw.Flush(); err != nil {
		fmt.Fprintf(stderr, "flush table: %v\n", err)
		return 1
	}
	return 0
}

func printWikiTable(stdout io.Writer, rows []wikiPageRow, stderr io.Writer) int {
	tw := tabwriter.NewWriter(stdout, 0, 0, 2, ' ', 0)
	fmt.Fprintln(tw, "PATH\tSIZE")
	for _, row := range rows {
		fmt.Fprintf(tw, "%s\t%d\n", row.Path, row.Size)
	}
	if err := tw.Flush(); err != nil {
		fmt.Fprintf(stderr, "flush table: %v\n", err)
		return 1
	}
	return 0
}

func pipelineStateLabel(row pipelineRow) string {
	if strings.TrimSpace(row.State.Result.Name) != "" {
		return row.State.Result.Name
	}
	return row.State.Name
}

func printJSON(stdout io.Writer, v any, stderr io.Writer) int {
	payload, err := json.MarshalIndent(v, "", "  ")
	if err != nil {
		fmt.Fprintf(stderr, "encode output: %v\n", err)
		return 1
	}
	fmt.Fprintln(stdout, string(payload))
	return 0
}

func newClientFromProfile(profileName string) (*api.Client, error) {
	p, err := profileFromConfig(profileName)
	if err != nil {
		return nil, err
	}
	return api.NewClientWithUser(p.BaseURL, p.Username, p.Token, nil), nil
}

func profileFromConfig(profileName string) (config.Profile, error) {
	cfg, err := config.Load()
	if err != nil {
		return config.Profile{}, fmt.Errorf("load config: %w", err)
	}
	p, _, err := cfg.ActiveProfile(profileName)
	if err != nil {
		return config.Profile{}, fmt.Errorf("resolve profile: %w", err)
	}
	if strings.TrimSpace(p.Token) == "" {
		return config.Profile{}, fmt.Errorf("profile has no token configured")
	}
	return p, nil
}

func printRootUsage(w io.Writer) {
	fmt.Fprintln(w, "bb - Bitbucket CLI (Cloud MVP)")
	fmt.Fprintf(w, "Version: %s\n", version.DisplayVersion())
	fmt.Fprintln(w, "")
	fmt.Fprintln(w, "Usage:")
	fmt.Fprintln(w, "  bb <command> [subcommand] [flags]")
	fmt.Fprintln(w, "")
	fmt.Fprintln(w, "Commands:")
	fmt.Fprintln(w, "  auth       Authenticate and inspect auth status")
	fmt.Fprintln(w, "  api        Call Bitbucket Cloud REST endpoints")
	fmt.Fprintln(w, "  repo       Repository operations")
	fmt.Fprintln(w, "  version    Show CLI version metadata")
	fmt.Fprintln(w, "  pr         Pull request operations")
	fmt.Fprintln(w, "  pipeline   Pipeline operations")
	fmt.Fprintln(w, "  wiki       Wiki operations")
	fmt.Fprintln(w, "  issue      Issue operations")
	fmt.Fprintln(w, "  completion Shell completion")
}

func runVersion(stdout io.Writer) int {
	fmt.Fprintf(stdout, "bb version %s\n", version.DisplayVersion())
	fmt.Fprintf(stdout, "commit: %s\n", version.ShortCommit())
	fmt.Fprintf(stdout, "built: %s\n", version.BuildDate)
	return 0
}

func setQueryIfNotEmpty(values url.Values, key, value string) {
	trimmed := strings.TrimSpace(value)
	if trimmed != "" {
		values.Set(key, trimmed)
	}
}

func setOptionalIssueField(body map[string]any, key, value string) {
	trimmed := strings.TrimSpace(value)
	if trimmed != "" {
		body[key] = trimmed
	}
}

func resolveRepoTarget(workspaceValue, repoValue string, requireRepo bool) (string, string, error) {
	workspace := strings.TrimSpace(workspaceValue)
	repo := strings.TrimSpace(repoValue)
	if workspace == "" || (requireRepo && repo == "") {
		inferredWorkspace, inferredRepo, err := inferBitbucketRepoFromGit(context.Background(), "")
		if err == nil {
			if workspace == "" {
				workspace = inferredWorkspace
			}
			if repo == "" {
				repo = inferredRepo
			}
		}
	}
	if workspace == "" {
		return "", "", fmt.Errorf("--workspace is required")
	}
	if requireRepo && repo == "" {
		return "", "", fmt.Errorf("--repo is required")
	}
	return workspace, repo, nil
}

func inferBitbucketRepoFromGit(ctx context.Context, dir string) (string, string, error) {
	out, err := gitCommandRunner(ctx, dir, "config", "--get", "remote.origin.url")
	if err != nil {
		return "", "", err
	}
	remote := strings.TrimSpace(string(out))
	if remote == "" {
		return "", "", fmt.Errorf("remote.origin.url not set")
	}

	workspace, repo, ok := parseBitbucketRemote(remote)
	if !ok {
		return "", "", fmt.Errorf("origin remote is not a Bitbucket repository")
	}
	return workspace, repo, nil
}

func parseBitbucketRemote(remote string) (string, string, bool) {
	trimmed := strings.TrimSpace(remote)
	if trimmed == "" {
		return "", "", false
	}

	if strings.Contains(trimmed, "://") {
		u, err := url.Parse(trimmed)
		if err != nil {
			return "", "", false
		}
		if strings.ToLower(u.Hostname()) != "bitbucket.org" {
			return "", "", false
		}
		return parseBitbucketPath(u.Path)
	}

	hostPart, pathPart, ok := strings.Cut(trimmed, ":")
	if !ok {
		return "", "", false
	}
	if at := strings.LastIndex(hostPart, "@"); at >= 0 {
		hostPart = hostPart[at+1:]
	}
	if strings.ToLower(strings.TrimSpace(hostPart)) != "bitbucket.org" {
		return "", "", false
	}
	return parseBitbucketPath(pathPart)
}

func parseBitbucketPath(rawPath string) (string, string, bool) {
	path := strings.TrimSpace(rawPath)
	path = strings.TrimPrefix(path, "/")
	path = strings.TrimSuffix(path, "/")
	parts := strings.Split(path, "/")
	if len(parts) != 2 {
		return "", "", false
	}
	workspace := strings.TrimSpace(parts[0])
	repo := strings.TrimSpace(strings.TrimSuffix(parts[1], ".git"))
	if workspace == "" || repo == "" {
		return "", "", false
	}
	return workspace, repo, true
}

const bashCompletionScript = `_bb_complete() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local cmds="auth api repo pr pipeline wiki issue completion version help"
  COMPREPLY=($(compgen -W "${cmds}" -- "${cur}"))
}
complete -F _bb_complete bb`

const zshCompletionScript = `#compdef bb
_arguments "1:command:(auth api repo pr pipeline wiki issue completion version help)"`

const fishCompletionScript = `complete -c bb -f -a "auth api repo pr pipeline wiki issue completion version help"`

const powershellCompletionScript = `Register-ArgumentCompleter -CommandName bb -ScriptBlock {
  param($wordToComplete)
  "auth","api","repo","pr","pipeline","wiki","issue","completion","version","help" |
    Where-Object { $_ -like "$wordToComplete*" } |
    ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }
}`

func cloneWikiToTemp(ctx context.Context, p config.Profile, workspace, repo string) (string, error) {
	remoteURL, err := wikiRemoteURLBuilder(p, workspace, repo)
	if err != nil {
		return "", err
	}
	tmpDir, err := os.MkdirTemp("", "bb-wiki-*")
	if err != nil {
		return "", fmt.Errorf("create temp dir: %w", err)
	}
	if _, err := gitCommandRunner(ctx, "", "clone", "--depth", "1", remoteURL, tmpDir); err != nil {
		_ = os.RemoveAll(tmpDir)
		return "", err
	}
	return tmpDir, nil
}

func listWikiPages(repoDir string) ([]wikiPageRow, error) {
	var rows []wikiPageRow
	err := filepath.WalkDir(repoDir, func(filePath string, d os.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if d.IsDir() {
			if d.Name() == ".git" {
				return filepath.SkipDir
			}
			return nil
		}

		rel, err := filepath.Rel(repoDir, filePath)
		if err != nil {
			return err
		}
		info, err := d.Info()
		if err != nil {
			return err
		}
		rows = append(rows, wikiPageRow{
			Path: filepath.ToSlash(rel),
			Size: info.Size(),
		})
		return nil
	})
	if err != nil {
		return nil, fmt.Errorf("list wiki pages: %w", err)
	}
	sort.Slice(rows, func(i, j int) bool {
		return rows[i].Path < rows[j].Path
	})
	return rows, nil
}

func normalizeWikiPagePath(page string) (string, error) {
	trimmed := strings.TrimSpace(page)
	if trimmed == "" {
		return "", fmt.Errorf("--page is required")
	}
	clean := path.Clean(trimmed)
	if clean == "." || clean == "/" {
		return "", fmt.Errorf("invalid --page value")
	}
	if strings.HasPrefix(clean, "../") || clean == ".." {
		return "", fmt.Errorf("invalid --page value")
	}
	return clean, nil
}

func buildWikiRemoteURL(p config.Profile, workspace, repo string) (string, error) {
	host := "bitbucket.org"
	if parsed, err := url.Parse(strings.TrimSpace(p.BaseURL)); err == nil && parsed.Host != "" {
		host = parsed.Host
		if host == "api.bitbucket.org" {
			host = "bitbucket.org"
		}
	}

	user := resolveWikiAuthUser(p.Username)
	if strings.TrimSpace(p.Token) == "" {
		return "", fmt.Errorf("profile has no token configured")
	}

	u := url.URL{
		Scheme: "https",
		Host:   host,
		Path:   fmt.Sprintf("/%s/%s.git/wiki", workspace, repo),
		User:   url.UserPassword(user, p.Token),
	}
	return u.String(), nil
}

func resolveWikiAuthUser(profileUsername string) string {
	user := strings.TrimSpace(profileUsername)
	if user == "" {
		// Access-token style flows (no API username in profile)
		// authenticate wiki Git over HTTPS with x-token-auth.
		return "x-token-auth"
	}
	if strings.Contains(user, "@") {
		// Personal API token profiles use Atlassian account email for REST,
		// but wiki Git over HTTPS expects token-auth style username.
		return "x-bitbucket-api-token-auth"
	}
	return user
}

func runGitCommand(ctx context.Context, dir string, args ...string) ([]byte, error) {
	cmd := exec.CommandContext(ctx, "git", args...)
	if strings.TrimSpace(dir) != "" {
		cmd.Dir = dir
	}
	out, err := cmd.CombinedOutput()
	if err != nil {
		msg := strings.TrimSpace(string(out))
		if msg == "" {
			msg = err.Error()
		}
		return nil, fmt.Errorf("git command failed: %s", msg)
	}
	return out, nil
}

func redactToken(input, token string) string {
	trimmedToken := strings.TrimSpace(token)
	if trimmedToken == "" {
		return input
	}
	return strings.ReplaceAll(input, trimmedToken, "***")
}
