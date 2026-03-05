package app

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strconv"
	"strings"
	"unicode/utf8"
)

type pullRequestRow struct {
	ID        int    `json:"id"`
	Title     string `json:"title"`
	State     string `json:"state"`
	CreatedOn string `json:"created_on"`
	Links     struct {
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

func runPR(args []string, stdout, stderr io.Writer) int {
	if len(args) == 0 || isHelpArg(args[0]) {
		printPRUsage(stdout)
		return 0
	}
	switch args[0] {
	case "list":
		return runPRList(args[1:], stdout, stderr)
	case "create":
		return runPRCreate(args[1:], stdout, stderr)
	case "merge":
		return runPRMerge(args[1:], stdout, stderr)
	case "view":
		return runPRView(args[1:], stdout, stderr)
	case "approve":
		return runPRApprove(args[1:], stdout, stderr)
	case "decline":
		return runPRDecline(args[1:], stdout, stderr)
	case "edit":
		return runPREdit(args[1:], stdout, stderr)
	case "comment":
		return runPRComment(args[1:], stdout, stderr)
	case "diff":
		return runPRDiff(args[1:], stdout, stderr)
	case "statuses":
		return runPRStatuses(args[1:], stdout, stderr)
	case "unapprove":
		return runPRUnapprove(args[1:], stdout, stderr)
	case "request-changes":
		return runPRRequestChanges(args[1:], stdout, stderr)
	case "checkout":
		return runPRCheckout(args[1:], stdout, stderr)
	case "activity":
		return runPRActivity(args[1:], stdout, stderr)
	case "comments":
		return runPRComments(args[1:], stdout, stderr)
	default:
		fmt.Fprintf(stderr, "unknown pr command: %s\n", args[0])
		return 1
	}
}

func runPRList(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pr list", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	output := fs.String("output", "table", "output format: table|json")
	all := fs.Bool("all", false, "fetch all pages")
	profile := fs.String("profile", "", "profile name override")
	state := fs.String("state", "", "pull request state filter (OPEN|MERGED|DECLINED)")
	q := fs.String("q", "", "Bitbucket q filter")
	sort := fs.String("sort", "", "sort expression")
	fields := fs.String("fields", "", "partial fields selector")
	fs.Usage = func() { printPRListHelp(stdout) }
	if err := fs.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return 0
		}
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	stateFilter, err := normalizePRStateFilter(*state)
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
	setQueryIfNotEmpty(query, "state", stateFilter)
	setQueryIfNotEmpty(query, "q", *q)
	setQueryIfNotEmpty(query, "sort", *sort)
	setQueryIfNotEmpty(query, "fields", *fields)

	path := fmt.Sprintf("/repositories/%s/%s/pullrequests", workspaceSlug, repoSlug)
	var values []json.RawMessage
	totalCount := -1
	if *all {
		values, err = client.GetAllValues(context.Background(), path, query)
		if err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
	} else {
		var page struct {
			Values []json.RawMessage `json:"values"`
			Size   int               `json:"size"`
		}
		if err := client.DoJSON(context.Background(), http.MethodGet, path, query, nil, &page); err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
		values = page.Values
		if page.Size > 0 {
			totalCount = page.Size
		}
	}

	switch *output {
	case "json":
		return printJSON(stdout, values, stderr)
	case "table":
		return printPRTable(stdout, values, workspaceSlug, repoSlug, stateFilter, totalCount, stderr)
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

func runPRCreate(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pr create", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	title := fs.String("title", "", "pull request title")
	source := fs.String("source", "", "source branch name")
	destination := fs.String("destination", "", "destination branch name")
	description := fs.String("description", "", "pull request description")
	closeBranch := fs.Bool("close-branch", false, "delete source branch after merge")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "text", "output format: text|json")
	fs.Usage = func() { printPRCreateHelp(stdout) }
	if err := fs.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return 0
		}
		fmt.Fprintf(stderr, "%v\n", err)
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
	switch *output {
	case "json", "text":
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
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
	if *closeBranch {
		body["close_source_branch"] = true
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

func runPRMerge(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pr merge", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	prID := fs.String("id", "", "pull request ID")
	message := fs.String("message", "", "merge commit message")
	strategy := fs.String("strategy", "", "merge strategy: merge_commit|squash|fast_forward")
	closeBranch := fs.Bool("close-branch", false, "delete source branch after merge")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "text", "output format: text|json")
	fs.Usage = func() { printPRMergeHelp(stdout) }
	if err := fs.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return 0
		}
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	idStr := strings.TrimSpace(*prID)
	if idStr == "" {
		fmt.Fprintln(stderr, "--id is required")
		return 1
	}
	if _, err := strconv.Atoi(idStr); err != nil {
		fmt.Fprintf(stderr, "--id must be a number: %s\n", idStr)
		return 1
	}
	switch *output {
	case "json", "text":
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
	mergeStrategy, err := normalizePRMergeStrategy(*strategy)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	body := map[string]any{}
	if strings.TrimSpace(*message) != "" {
		body["message"] = *message
	}
	if mergeStrategy != "" {
		body["merge_strategy"] = mergeStrategy
	}
	if *closeBranch {
		body["close_source_branch"] = true
	}
	payload, err := json.Marshal(body)
	if err != nil {
		fmt.Fprintf(stderr, "encode request body: %v\n", err)
		return 1
	}

	apiPath := fmt.Sprintf("/repositories/%s/%s/pullrequests/%s/merge", workspaceSlug, repoSlug, idStr)
	var merged pullRequestRow
	if err := client.DoJSON(context.Background(), http.MethodPost, apiPath, nil, bytes.NewReader(payload), &merged); err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	switch *output {
	case "json":
		return printJSON(stdout, merged, stderr)
	case "text":
		fmt.Fprintf(stdout, "Merged PR #%d (%s): %s\n", merged.ID, merged.State, merged.Title)
		if strings.TrimSpace(merged.Links.HTML.Href) != "" {
			fmt.Fprintf(stdout, "URL: %s\n", merged.Links.HTML.Href)
		}
		return 0
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

func printPRTable(stdout io.Writer, values []json.RawMessage, workspace, repo, stateFilter string, totalCount int, stderr io.Writer) int {
	rows := make([]pullRequestRow, 0, len(values))
	for _, raw := range values {
		var row pullRequestRow
		if err := json.Unmarshal(raw, &row); err != nil {
			fmt.Fprintf(stderr, "decode pull request row: %v\n", err)
			return 1
		}
		rows = append(rows, row)
	}

	stateLabel := describePRListState(stateFilter)
	if totalCount > len(rows) {
		fmt.Fprintf(stdout, "Showing %d of %d %s in %s/%s\n\n", len(rows), totalCount, stateLabel, workspace, repo)
	} else {
		fmt.Fprintf(stdout, "Showing %d %s in %s/%s\n\n", len(rows), stateLabel, workspace, repo)
	}

	useColor := shouldUseColor(stdout)
	headers := []string{"ID", "TITLE", "BRANCH", "CREATED AT"}
	type viewRow struct {
		ID        string
		Title     string
		Branch    string
		CreatedAt string
	}
	viewRows := make([]viewRow, 0, len(rows))
	widthID := utf8.RuneCountInString(headers[0])
	widthTitle := utf8.RuneCountInString(headers[1])
	widthBranch := utf8.RuneCountInString(headers[2])
	widthCreatedAt := utf8.RuneCountInString(headers[3])

	for _, row := range rows {
		branch := strings.TrimSpace(row.Source.Branch.Name)
		if branch == "" {
			branch = "-"
		}
		title := strings.TrimSpace(row.Title)
		if title == "" {
			title = "-"
		}
		createdAt := relativeTimeLabel(row.CreatedOn)
		view := viewRow{
			ID:        fmt.Sprintf("#%d", row.ID),
			Title:     title,
			Branch:    branch,
			CreatedAt: createdAt,
		}
		viewRows = append(viewRows, view)
		widthID = maxRuneWidth(widthID, view.ID)
		widthTitle = maxRuneWidth(widthTitle, view.Title)
		widthBranch = maxRuneWidth(widthBranch, view.Branch)
		widthCreatedAt = maxRuneWidth(widthCreatedAt, view.CreatedAt)
	}

	headerID := padRight(headers[0], widthID)
	headerTitle := padRight(headers[1], widthTitle)
	headerBranch := padRight(headers[2], widthBranch)
	headerCreatedAt := padRight(headers[3], widthCreatedAt)
	fmt.Fprintf(
		stdout,
		"%s  %s  %s  %s\n",
		ansi(headerID, "1", useColor),
		ansi(headerTitle, "1", useColor),
		ansi(headerBranch, "1", useColor),
		ansi(headerCreatedAt, "1", useColor),
	)

	for _, row := range viewRows {
		id := padRight(row.ID, widthID)
		title := padRight(row.Title, widthTitle)
		branch := padRight(row.Branch, widthBranch)
		createdAt := padRight(row.CreatedAt, widthCreatedAt)
		fmt.Fprintf(
			stdout,
			"%s  %s  %s  %s\n",
			ansi(id, "1;36", useColor),
			title,
			ansi(branch, "36", useColor),
			ansi(createdAt, "2", useColor),
		)
	}

	return 0
}

func describePRListState(stateFilter string) string {
	state := strings.ToUpper(strings.TrimSpace(stateFilter))
	switch state {
	case "OPEN":
		return "open pull requests"
	case "MERGED":
		return "merged pull requests"
	case "DECLINED":
		return "declined pull requests"
	case "":
		return "pull requests"
	default:
		return strings.ToLower(state) + " pull requests"
	}
}

func normalizePRStateFilter(value string) (string, error) {
	state := strings.ToUpper(strings.TrimSpace(value))
	if state == "" {
		return "", nil
	}
	switch state {
	case "OPEN", "MERGED", "DECLINED":
		return state, nil
	default:
		return "", fmt.Errorf("--state must be one of OPEN, MERGED, DECLINED")
	}
}

func normalizePRMergeStrategy(value string) (string, error) {
	strategy := strings.ToLower(strings.TrimSpace(value))
	if strategy == "" {
		return "", nil
	}
	switch strategy {
	case "merge_commit", "squash", "fast_forward":
		return strategy, nil
	default:
		return "", fmt.Errorf("--strategy must be one of merge_commit, squash, fast_forward")
	}
}

func parsePRID(idStr string) (string, error) {
	trimmed := strings.TrimSpace(idStr)
	if trimmed == "" {
		return "", fmt.Errorf("--id is required")
	}
	if _, err := strconv.Atoi(trimmed); err != nil {
		return "", fmt.Errorf("--id must be a number: %s", trimmed)
	}
	return trimmed, nil
}

// ---------- pr view ----------

type pullRequestDetail struct {
	ID          int    `json:"id"`
	Title       string `json:"title"`
	Description string `json:"description"`
	State       string `json:"state"`
	CreatedOn   string `json:"created_on"`
	UpdatedOn   string `json:"updated_on"`
	Author      struct {
		DisplayName string `json:"display_name"`
	} `json:"author"`
	CloseSourceBranch bool `json:"close_source_branch"`
	Links             struct {
		HTML struct {
			Href string `json:"href"`
		} `json:"html"`
	} `json:"links"`
	Source struct {
		Branch struct {
			Name string `json:"name"`
		} `json:"branch"`
		Repository struct {
			FullName string `json:"full_name"`
			Links    struct {
				HTML struct {
					Href string `json:"href"`
				} `json:"html"`
			} `json:"links"`
		} `json:"repository"`
	} `json:"source"`
	Destination struct {
		Branch struct {
			Name string `json:"name"`
		} `json:"branch"`
		Repository struct {
			FullName string `json:"full_name"`
		} `json:"repository"`
	} `json:"destination"`
	Participants []struct {
		User struct {
			DisplayName string `json:"display_name"`
		} `json:"user"`
		Role     string `json:"role"`
		Approved bool   `json:"approved"`
	} `json:"participants"`
	CommentCount int `json:"comment_count"`
	TaskCount    int `json:"task_count"`
}

func runPRView(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pr view", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	prID := fs.String("id", "", "pull request ID")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "text", "output format: text|json")
	fs.Usage = func() { printPRViewHelp(stdout) }
	if err := fs.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return 0
		}
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	idStr, err := parsePRID(*prID)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	switch *output {
	case "json", "text":
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	apiPath := fmt.Sprintf("/repositories/%s/%s/pullrequests/%s", workspaceSlug, repoSlug, idStr)
	var detail pullRequestDetail
	if err := client.DoJSON(context.Background(), http.MethodGet, apiPath, nil, nil, &detail); err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	switch *output {
	case "json":
		return printJSON(stdout, detail, stderr)
	case "text":
		fmt.Fprintf(stdout, "Title:        %s\n", detail.Title)
		fmt.Fprintf(stdout, "State:        %s\n", detail.State)
		if detail.Author.DisplayName != "" {
			fmt.Fprintf(stdout, "Author:       %s\n", detail.Author.DisplayName)
		}
		src := strings.TrimSpace(detail.Source.Branch.Name)
		dst := strings.TrimSpace(detail.Destination.Branch.Name)
		if src != "" && dst != "" {
			fmt.Fprintf(stdout, "Source:       %s → %s\n", src, dst)
		}
		fmt.Fprintf(stdout, "Created:      %s\n", relativeTimeLabel(detail.CreatedOn))
		fmt.Fprintf(stdout, "Updated:      %s\n", relativeTimeLabel(detail.UpdatedOn))
		if len(detail.Participants) > 0 {
			var parts []string
			for _, p := range detail.Participants {
				if p.Role == "REVIEWER" {
					status := "pending"
					if p.Approved {
						status = "approved"
					}
					parts = append(parts, fmt.Sprintf("%s (%s)", p.User.DisplayName, status))
				}
			}
			if len(parts) > 0 {
				fmt.Fprintf(stdout, "Reviewers:    %s\n", strings.Join(parts, ", "))
			}
		}
		if detail.CommentCount > 0 {
			fmt.Fprintf(stdout, "Comments:     %d\n", detail.CommentCount)
		}
		if strings.TrimSpace(detail.Links.HTML.Href) != "" {
			fmt.Fprintf(stdout, "URL:          %s\n", detail.Links.HTML.Href)
		}
		if strings.TrimSpace(detail.Description) != "" {
			fmt.Fprintf(stdout, "\n%s\n", detail.Description)
		}
		return 0
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

// ---------- pr approve ----------

type participantResponse struct {
	User struct {
		DisplayName string `json:"display_name"`
	} `json:"user"`
	Role     string `json:"role"`
	Approved bool   `json:"approved"`
}

func runPRApprove(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pr approve", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	prID := fs.String("id", "", "pull request ID")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "text", "output format: text|json")
	fs.Usage = func() { printPRApproveHelp(stdout) }
	if err := fs.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return 0
		}
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	idStr, err := parsePRID(*prID)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	switch *output {
	case "json", "text":
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	apiPath := fmt.Sprintf("/repositories/%s/%s/pullrequests/%s/approve", workspaceSlug, repoSlug, idStr)
	var participant participantResponse
	if err := client.DoJSON(context.Background(), http.MethodPost, apiPath, nil, nil, &participant); err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	switch *output {
	case "json":
		return printJSON(stdout, participant, stderr)
	case "text":
		fmt.Fprintf(stdout, "Approved PR #%s in %s/%s\n", idStr, workspaceSlug, repoSlug)
		return 0
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

// ---------- pr decline ----------

func runPRDecline(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pr decline", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	prID := fs.String("id", "", "pull request ID")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "text", "output format: text|json")
	fs.Usage = func() { printPRDeclineHelp(stdout) }
	if err := fs.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return 0
		}
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	idStr, err := parsePRID(*prID)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	switch *output {
	case "json", "text":
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	apiPath := fmt.Sprintf("/repositories/%s/%s/pullrequests/%s/decline", workspaceSlug, repoSlug, idStr)
	var declined pullRequestRow
	if err := client.DoJSON(context.Background(), http.MethodPost, apiPath, nil, nil, &declined); err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	switch *output {
	case "json":
		return printJSON(stdout, declined, stderr)
	case "text":
		fmt.Fprintf(stdout, "Declined PR #%d (%s): %s\n", declined.ID, declined.State, declined.Title)
		if strings.TrimSpace(declined.Links.HTML.Href) != "" {
			fmt.Fprintf(stdout, "URL: %s\n", declined.Links.HTML.Href)
		}
		return 0
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

// ---------- pr edit ----------

func runPREdit(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pr edit", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	prID := fs.String("id", "", "pull request ID")
	title := fs.String("title", "", "pull request title")
	description := fs.String("description", "", "pull request description")
	closeBranch := fs.Bool("close-branch", false, "delete source branch after merge")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "text", "output format: text|json")
	fs.Usage = func() { printPREditHelp(stdout) }
	if err := fs.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return 0
		}
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	idStr, err := parsePRID(*prID)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	switch *output {
	case "json", "text":
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}

	body := map[string]any{}
	if strings.TrimSpace(*title) != "" {
		body["title"] = *title
	}
	if strings.TrimSpace(*description) != "" {
		body["description"] = *description
	}
	if *closeBranch {
		body["close_source_branch"] = true
	}
	if len(body) == 0 {
		fmt.Fprintln(stderr, "at least one field to update is required (--title, --description, --close-branch)")
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

	apiPath := fmt.Sprintf("/repositories/%s/%s/pullrequests/%s", workspaceSlug, repoSlug, idStr)
	var updated pullRequestRow
	if err := client.DoJSON(context.Background(), http.MethodPut, apiPath, nil, bytes.NewReader(payload), &updated); err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	switch *output {
	case "json":
		return printJSON(stdout, updated, stderr)
	case "text":
		fmt.Fprintf(stdout, "Updated PR #%d (%s): %s\n", updated.ID, updated.State, updated.Title)
		if strings.TrimSpace(updated.Links.HTML.Href) != "" {
			fmt.Fprintf(stdout, "URL: %s\n", updated.Links.HTML.Href)
		}
		return 0
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

// ---------- pr comment ----------

type prCommentResponse struct {
	ID      int `json:"id"`
	Content struct {
		Raw string `json:"raw"`
	} `json:"content"`
	CreatedOn string `json:"created_on"`
	User      struct {
		DisplayName string `json:"display_name"`
	} `json:"user"`
}

func runPRComment(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pr comment", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	prID := fs.String("id", "", "pull request ID")
	body := fs.String("body", "", "comment body text")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "text", "output format: text|json")
	fs.Usage = func() { printPRCommentHelp(stdout) }
	if err := fs.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return 0
		}
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	idStr, err := parsePRID(*prID)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	if strings.TrimSpace(*body) == "" {
		fmt.Fprintln(stderr, "--body is required")
		return 1
	}
	switch *output {
	case "json", "text":
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	reqBody := map[string]any{
		"content": map[string]any{
			"raw": *body,
		},
	}
	payload, err := json.Marshal(reqBody)
	if err != nil {
		fmt.Fprintf(stderr, "encode request body: %v\n", err)
		return 1
	}

	apiPath := fmt.Sprintf("/repositories/%s/%s/pullrequests/%s/comments", workspaceSlug, repoSlug, idStr)
	var comment prCommentResponse
	if err := client.DoJSON(context.Background(), http.MethodPost, apiPath, nil, bytes.NewReader(payload), &comment); err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	switch *output {
	case "json":
		return printJSON(stdout, comment, stderr)
	case "text":
		fmt.Fprintf(stdout, "Added comment #%d to PR #%s\n", comment.ID, idStr)
		return 0
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

// ---------- pr diff ----------

func runPRDiff(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pr diff", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	prID := fs.String("id", "", "pull request ID")
	profile := fs.String("profile", "", "profile name override")
	fs.Usage = func() { printPRDiffHelp(stdout) }
	if err := fs.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return 0
		}
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	idStr, err := parsePRID(*prID)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	apiPath := fmt.Sprintf("/repositories/%s/%s/pullrequests/%s/diff", workspaceSlug, repoSlug, idStr)
	resp, err := client.Request(context.Background(), http.MethodGet, apiPath, nil, nil)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}
	defer resp.Body.Close()

	if resp.StatusCode >= http.StatusBadRequest {
		limited, _ := io.ReadAll(io.LimitReader(resp.Body, 4*1024))
		fmt.Fprintf(stderr, "api request failed: status %d: %s\n", resp.StatusCode, strings.TrimSpace(string(limited)))
		return 1
	}
	if _, err := io.Copy(stdout, resp.Body); err != nil {
		fmt.Fprintf(stderr, "read diff: %v\n", err)
		return 1
	}
	return 0
}

// ---------- pr statuses ----------

type commitStatusRow struct {
	State       string `json:"state"`
	Key         string `json:"key"`
	Name        string `json:"name"`
	Description string `json:"description"`
	URL         string `json:"url"`
	CreatedOn   string `json:"created_on"`
}

func runPRStatuses(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pr statuses", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	prID := fs.String("id", "", "pull request ID")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "table", "output format: table|json")
	all := fs.Bool("all", false, "fetch all pages")
	fs.Usage = func() { printPRStatusesHelp(stdout) }
	if err := fs.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return 0
		}
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	idStr, err := parsePRID(*prID)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	switch *output {
	case "json", "table":
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	apiPath := fmt.Sprintf("/repositories/%s/%s/pullrequests/%s/statuses", workspaceSlug, repoSlug, idStr)
	var values []json.RawMessage
	if *all {
		values, err = client.GetAllValues(context.Background(), apiPath, nil)
		if err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
	} else {
		var page struct {
			Values []json.RawMessage `json:"values"`
		}
		if err := client.DoJSON(context.Background(), http.MethodGet, apiPath, nil, nil, &page); err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
		values = page.Values
	}

	switch *output {
	case "json":
		return printJSON(stdout, values, stderr)
	case "table":
		return printStatusTable(stdout, values, stderr)
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

// ---------- pr unapprove ----------

func runPRUnapprove(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pr unapprove", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	prID := fs.String("id", "", "pull request ID")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "text", "output format: text|json")
	fs.Usage = func() { printPRUnapproveHelp(stdout) }
	if err := fs.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return 0
		}
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	idStr, err := parsePRID(*prID)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	switch *output {
	case "json", "text":
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	apiPath := fmt.Sprintf("/repositories/%s/%s/pullrequests/%s/approve", workspaceSlug, repoSlug, idStr)
	if err := client.DoJSON(context.Background(), http.MethodDelete, apiPath, nil, nil, nil); err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	switch *output {
	case "json":
		return printJSON(stdout, map[string]any{"status": "unapproved", "pr_id": idStr}, stderr)
	case "text":
		fmt.Fprintf(stdout, "Unapproved PR #%s in %s/%s\n", idStr, workspaceSlug, repoSlug)
		return 0
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

// ---------- pr request-changes ----------

func runPRRequestChanges(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pr request-changes", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	prID := fs.String("id", "", "pull request ID")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "text", "output format: text|json")
	fs.Usage = func() { printPRRequestChangesHelp(stdout) }
	if err := fs.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return 0
		}
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	idStr, err := parsePRID(*prID)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	switch *output {
	case "json", "text":
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	apiPath := fmt.Sprintf("/repositories/%s/%s/pullrequests/%s/request-changes", workspaceSlug, repoSlug, idStr)
	var participant participantResponse
	if err := client.DoJSON(context.Background(), http.MethodPost, apiPath, nil, nil, &participant); err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	switch *output {
	case "json":
		return printJSON(stdout, participant, stderr)
	case "text":
		fmt.Fprintf(stdout, "Requested changes on PR #%s in %s/%s\n", idStr, workspaceSlug, repoSlug)
		return 0
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

// ---------- pr checkout ----------

func runPRCheckout(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pr checkout", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	prID := fs.String("id", "", "pull request ID")
	profile := fs.String("profile", "", "profile name override")
	fs.Usage = func() { printPRCheckoutHelp(stdout) }
	if err := fs.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return 0
		}
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	idStr, err := parsePRID(*prID)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	apiPath := fmt.Sprintf("/repositories/%s/%s/pullrequests/%s", workspaceSlug, repoSlug, idStr)
	var detail pullRequestDetail
	if err := client.DoJSON(context.Background(), http.MethodGet, apiPath, nil, nil, &detail); err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	branch := strings.TrimSpace(detail.Source.Branch.Name)
	if branch == "" {
		fmt.Fprintln(stderr, "source branch not found in PR")
		return 1
	}

	srcRepo := strings.TrimSpace(detail.Source.Repository.FullName)
	dstRepo := strings.TrimSpace(detail.Destination.Repository.FullName)
	isFork := srcRepo != "" && dstRepo != "" && srcRepo != dstRepo

	ctx := context.Background()
	if isFork {
		forkURL := strings.TrimSpace(detail.Source.Repository.Links.HTML.Href)
		if forkURL == "" {
			forkURL = fmt.Sprintf("https://bitbucket.org/%s", srcRepo)
		}
		if _, err := gitCommandRunner(ctx, "", "fetch", forkURL, branch+":"+branch); err != nil {
			fmt.Fprintf(stderr, "git fetch from fork failed: %v\n", err)
			return 1
		}
	} else {
		if _, err := gitCommandRunner(ctx, "", "fetch", "origin", branch); err != nil {
			fmt.Fprintf(stderr, "git fetch failed: %v\n", err)
			return 1
		}
	}
	if _, err := gitCommandRunner(ctx, "", "checkout", branch); err != nil {
		fmt.Fprintf(stderr, "git checkout failed: %v\n", err)
		return 1
	}

	fmt.Fprintf(stdout, "Switched to branch '%s' from PR #%s\n", branch, idStr)
	return 0
}

// ---------- pr activity ----------

func runPRActivity(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pr activity", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	prID := fs.String("id", "", "pull request ID")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "table", "output format: table|json")
	all := fs.Bool("all", false, "fetch all pages")
	fs.Usage = func() { printPRActivityHelp(stdout) }
	if err := fs.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return 0
		}
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	idStr, err := parsePRID(*prID)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	switch *output {
	case "json", "table":
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	apiPath := fmt.Sprintf("/repositories/%s/%s/pullrequests/%s/activity", workspaceSlug, repoSlug, idStr)
	var values []json.RawMessage
	if *all {
		values, err = client.GetAllValues(context.Background(), apiPath, nil)
		if err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
	} else {
		var page struct {
			Values []json.RawMessage `json:"values"`
		}
		if err := client.DoJSON(context.Background(), http.MethodGet, apiPath, nil, nil, &page); err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
		values = page.Values
	}

	switch *output {
	case "json":
		return printJSON(stdout, values, stderr)
	case "table":
		return printActivityTable(stdout, values, stderr)
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

type activityEntry struct {
	Update   *activityUpdate   `json:"update"`
	Approval *activityApproval `json:"approval"`
	Comment  *activityComment  `json:"comment"`
}

type activityUpdate struct {
	State  string `json:"state"`
	Date   string `json:"date"`
	Author struct {
		DisplayName string `json:"display_name"`
	} `json:"author"`
}

type activityApproval struct {
	Date string `json:"date"`
	User struct {
		DisplayName string `json:"display_name"`
	} `json:"user"`
}

type activityComment struct {
	CreatedOn string `json:"created_on"`
	User      struct {
		DisplayName string `json:"display_name"`
	} `json:"user"`
	Content struct {
		Raw string `json:"raw"`
	} `json:"content"`
}

func printActivityTable(stdout io.Writer, values []json.RawMessage, stderr io.Writer) int {
	type viewRow struct {
		Kind   string
		Author string
		Date   string
		Detail string
	}

	var rows []viewRow
	for _, raw := range values {
		var entry activityEntry
		if err := json.Unmarshal(raw, &entry); err != nil {
			fmt.Fprintf(stderr, "decode activity entry: %v\n", err)
			return 1
		}
		switch {
		case entry.Update != nil:
			rows = append(rows, viewRow{
				Kind:   "update",
				Author: entry.Update.Author.DisplayName,
				Date:   relativeTimeLabel(entry.Update.Date),
				Detail: entry.Update.State,
			})
		case entry.Approval != nil:
			rows = append(rows, viewRow{
				Kind:   "approval",
				Author: entry.Approval.User.DisplayName,
				Date:   relativeTimeLabel(entry.Approval.Date),
			})
		case entry.Comment != nil:
			snippet := entry.Comment.Content.Raw
			if len(snippet) > 60 {
				snippet = snippet[:60] + "..."
			}
			rows = append(rows, viewRow{
				Kind:   "comment",
				Author: entry.Comment.User.DisplayName,
				Date:   relativeTimeLabel(entry.Comment.CreatedOn),
				Detail: snippet,
			})
		}
	}

	useColor := shouldUseColor(stdout)
	headers := []string{"TYPE", "AUTHOR", "DATE", "DETAIL"}
	widthKind := utf8.RuneCountInString(headers[0])
	widthAuthor := utf8.RuneCountInString(headers[1])
	widthDate := utf8.RuneCountInString(headers[2])
	widthDetail := utf8.RuneCountInString(headers[3])
	for _, r := range rows {
		widthKind = maxRuneWidth(widthKind, r.Kind)
		widthAuthor = maxRuneWidth(widthAuthor, r.Author)
		widthDate = maxRuneWidth(widthDate, r.Date)
		widthDetail = maxRuneWidth(widthDetail, r.Detail)
	}

	fmt.Fprintf(stdout, "%s  %s  %s  %s\n",
		ansi(padRight(headers[0], widthKind), "1", useColor),
		ansi(padRight(headers[1], widthAuthor), "1", useColor),
		ansi(padRight(headers[2], widthDate), "1", useColor),
		ansi(padRight(headers[3], widthDetail), "1", useColor),
	)
	for _, r := range rows {
		fmt.Fprintf(stdout, "%s  %s  %s  %s\n",
			ansi(padRight(r.Kind, widthKind), "36", useColor),
			padRight(r.Author, widthAuthor),
			ansi(padRight(r.Date, widthDate), "2", useColor),
			padRight(r.Detail, widthDetail),
		)
	}
	return 0
}

// ---------- pr comments ----------

func runPRComments(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("pr comments", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	workspace := fs.String("workspace", "", "workspace slug")
	repo := fs.String("repo", "", "repository slug")
	prID := fs.String("id", "", "pull request ID")
	profile := fs.String("profile", "", "profile name override")
	output := fs.String("output", "table", "output format: table|json")
	all := fs.Bool("all", false, "fetch all pages")
	fs.Usage = func() { printPRCommentsHelp(stdout) }
	if err := fs.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return 0
		}
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}
	workspaceSlug, repoSlug, err := resolveRepoTarget(*workspace, *repo, true)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	idStr, err := parsePRID(*prID)
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
		return 1
	}
	switch *output {
	case "json", "table":
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}

	client, err := newClientFromProfile(*profile)
	if err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return 1
	}

	apiPath := fmt.Sprintf("/repositories/%s/%s/pullrequests/%s/comments", workspaceSlug, repoSlug, idStr)
	var values []json.RawMessage
	if *all {
		values, err = client.GetAllValues(context.Background(), apiPath, nil)
		if err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
	} else {
		var page struct {
			Values []json.RawMessage `json:"values"`
		}
		if err := client.DoJSON(context.Background(), http.MethodGet, apiPath, nil, nil, &page); err != nil {
			fmt.Fprintf(stderr, "%v\n", err)
			return 1
		}
		values = page.Values
	}

	switch *output {
	case "json":
		return printJSON(stdout, values, stderr)
	case "table":
		return printCommentsTable(stdout, values, stderr)
	default:
		fmt.Fprintf(stderr, "unsupported output format: %s\n", *output)
		return 1
	}
}

func printCommentsTable(stdout io.Writer, values []json.RawMessage, stderr io.Writer) int {
	type commentRow struct {
		ID      int    `json:"id"`
		Content struct {
			Raw string `json:"raw"`
		} `json:"content"`
		CreatedOn string `json:"created_on"`
		User      struct {
			DisplayName string `json:"display_name"`
		} `json:"user"`
	}

	rows := make([]commentRow, 0, len(values))
	for _, raw := range values {
		var row commentRow
		if err := json.Unmarshal(raw, &row); err != nil {
			fmt.Fprintf(stderr, "decode comment row: %v\n", err)
			return 1
		}
		rows = append(rows, row)
	}

	useColor := shouldUseColor(stdout)
	headers := []string{"ID", "AUTHOR", "CREATED", "BODY"}
	type viewRow struct {
		ID      string
		Author  string
		Created string
		Body    string
	}
	viewRows := make([]viewRow, 0, len(rows))
	widthID := utf8.RuneCountInString(headers[0])
	widthAuthor := utf8.RuneCountInString(headers[1])
	widthCreated := utf8.RuneCountInString(headers[2])
	widthBody := utf8.RuneCountInString(headers[3])

	for _, row := range rows {
		body := strings.ReplaceAll(row.Content.Raw, "\n", " ")
		if len(body) > 60 {
			body = body[:60] + "..."
		}
		v := viewRow{
			ID:      fmt.Sprintf("#%d", row.ID),
			Author:  row.User.DisplayName,
			Created: relativeTimeLabel(row.CreatedOn),
			Body:    body,
		}
		viewRows = append(viewRows, v)
		widthID = maxRuneWidth(widthID, v.ID)
		widthAuthor = maxRuneWidth(widthAuthor, v.Author)
		widthCreated = maxRuneWidth(widthCreated, v.Created)
		widthBody = maxRuneWidth(widthBody, v.Body)
	}

	fmt.Fprintf(stdout, "%s  %s  %s  %s\n",
		ansi(padRight(headers[0], widthID), "1", useColor),
		ansi(padRight(headers[1], widthAuthor), "1", useColor),
		ansi(padRight(headers[2], widthCreated), "1", useColor),
		ansi(padRight(headers[3], widthBody), "1", useColor),
	)
	for _, r := range viewRows {
		fmt.Fprintf(stdout, "%s  %s  %s  %s\n",
			ansi(padRight(r.ID, widthID), "1;36", useColor),
			padRight(r.Author, widthAuthor),
			ansi(padRight(r.Created, widthCreated), "2", useColor),
			padRight(r.Body, widthBody),
		)
	}
	return 0
}

func printStatusTable(stdout io.Writer, values []json.RawMessage, stderr io.Writer) int {
	rows := make([]commitStatusRow, 0, len(values))
	for _, raw := range values {
		var row commitStatusRow
		if err := json.Unmarshal(raw, &row); err != nil {
			fmt.Fprintf(stderr, "decode status row: %v\n", err)
			return 1
		}
		rows = append(rows, row)
	}

	useColor := shouldUseColor(stdout)
	headers := []string{"STATE", "NAME", "URL"}
	widthState := utf8.RuneCountInString(headers[0])
	widthName := utf8.RuneCountInString(headers[1])
	widthURL := utf8.RuneCountInString(headers[2])
	for _, row := range rows {
		widthState = maxRuneWidth(widthState, row.State)
		name := row.Name
		if name == "" {
			name = row.Key
		}
		widthName = maxRuneWidth(widthName, name)
		widthURL = maxRuneWidth(widthURL, row.URL)
	}

	fmt.Fprintf(stdout, "%s  %s  %s\n",
		ansi(padRight(headers[0], widthState), "1", useColor),
		ansi(padRight(headers[1], widthName), "1", useColor),
		ansi(padRight(headers[2], widthURL), "1", useColor),
	)
	for _, row := range rows {
		stateCode := "0"
		switch strings.ToUpper(row.State) {
		case "SUCCESSFUL":
			stateCode = "32"
		case "FAILED":
			stateCode = "31"
		case "INPROGRESS":
			stateCode = "33"
		}
		name := row.Name
		if name == "" {
			name = row.Key
		}
		fmt.Fprintf(stdout, "%s  %s  %s\n",
			ansi(padRight(row.State, widthState), stateCode, useColor),
			padRight(name, widthName),
			padRight(row.URL, widthURL),
		)
	}
	return 0
}
