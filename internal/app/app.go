package app

import (
	"bufio"
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
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
		fmt.Fprintln(stderr, "bb pr is not implemented yet")
		return 1
	case "pipeline":
		fmt.Fprintln(stderr, "bb pipeline is not implemented yet")
		return 1
	case "issue":
		fmt.Fprintln(stderr, "bb issue is not implemented yet")
		return 1
	case "completion":
		fmt.Fprintln(stderr, "bb completion is not implemented yet")
		return 1
	case "-h", "--help", "help":
		printRootUsage(stdout)
		return 0
	default:
		fmt.Fprintf(stderr, "unknown command: %s\n\n", args[0])
		printRootUsage(stderr)
		return 1
	}
}

func runAuth(args []string, stdout, stderr io.Writer) int {
	if len(args) == 0 {
		fmt.Fprintln(stderr, "usage: bb auth <login|status>")
		return 1
	}
	switch args[0] {
	case "login":
		return runAuthLogin(args[1:], stdout, stderr)
	case "status":
		return runAuthStatus(args[1:], stdout, stderr)
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

	cfg, err := config.Load()
	if err != nil {
		fmt.Fprintf(stderr, "load config: %v\n", err)
		return 1
	}
	cfg.SetProfile(*profile, resolvedToken, *baseURL)
	if err := cfg.Save(); err != nil {
		fmt.Fprintf(stderr, "save config: %v\n", err)
		return 1
	}

	fmt.Fprintf(stdout, "authenticated profile %q\n", *profile)
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
	if strings.TrimSpace(p.Token) == "" {
		fmt.Fprintln(stdout, "Token: not configured")
	} else {
		fmt.Fprintln(stdout, "Token: configured")
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
	if strings.TrimSpace(*workspace) == "" {
		fmt.Fprintln(stderr, "--workspace is required")
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

	path := fmt.Sprintf("/repositories/%s", *workspace)
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
	cfg, err := config.Load()
	if err != nil {
		return nil, fmt.Errorf("load config: %w", err)
	}
	p, _, err := cfg.ActiveProfile(profileName)
	if err != nil {
		return nil, fmt.Errorf("resolve profile: %w", err)
	}
	if strings.TrimSpace(p.Token) == "" {
		return nil, fmt.Errorf("profile has no token configured")
	}
	return api.NewClient(p.BaseURL, p.Token, nil), nil
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
	fmt.Fprintln(w, "  pr         Pull request operations (stub)")
	fmt.Fprintln(w, "  pipeline   Pipeline operations (stub)")
	fmt.Fprintln(w, "  issue      Issue operations (stub)")
	fmt.Fprintln(w, "  completion Shell completion (stub)")
}

func runVersion(stdout io.Writer) int {
	fmt.Fprintf(stdout, "bb version %s\n", version.DisplayVersion())
	fmt.Fprintf(stdout, "commit: %s\n", version.ShortCommit())
	fmt.Fprintf(stdout, "built: %s\n", version.BuildDate)
	return 0
}
