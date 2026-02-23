package app

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"

	"bitbucket-cli/internal/config"
	"bitbucket-cli/internal/version"
)

func TestAuthLoginAndStatus(t *testing.T) {
	configPath := filepath.Join(t.TempDir(), "config.json")
	t.Setenv("BB_CONFIG_PATH", configPath)
	t.Setenv("BITBUCKET_TOKEN", "token-from-env")

	var stdout, stderr bytes.Buffer
	code := Run([]string{"auth", "login", "--profile", "default"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}

	stdout.Reset()
	stderr.Reset()
	code = Run([]string{"auth", "status"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), "Profile: default") {
		t.Fatalf("unexpected status output: %q", stdout.String())
	}
	if strings.Contains(stdout.String(), "token-from-env") {
		t.Fatal("status output leaked raw token")
	}
}

func TestAuthLoginWithUsernameAndStatus(t *testing.T) {
	configPath := filepath.Join(t.TempDir(), "config.json")
	t.Setenv("BB_CONFIG_PATH", configPath)
	t.Setenv("BITBUCKET_TOKEN", "token-from-env")

	var stdout, stderr bytes.Buffer
	code := Run([]string{"auth", "login", "--profile", "default", "--username", "dev@example.com"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}

	stdout.Reset()
	stderr.Reset()
	code = Run([]string{"auth", "status"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), "Auth: basic (dev@example.com)") {
		t.Fatalf("unexpected status output: %q", stdout.String())
	}
}

func TestRepoListJSON(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/2.0/repositories/acme" {
			http.NotFound(w, r)
			return
		}
		_ = json.NewEncoder(w).Encode(map[string]any{
			"values": []map[string]any{
				{"slug": "one", "full_name": "acme/one"},
				{"slug": "two", "full_name": "acme/two"},
			},
		})
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL+"/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"repo", "list", "--workspace", "acme", "--output", "json"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), "\"slug\": \"one\"") {
		t.Fatalf("expected repo output, got %q", stdout.String())
	}
}

func TestRepoListUsesBasicAuthWhenUsernameConfigured(t *testing.T) {
	var gotUser string
	var gotPass string
	var gotBasic bool

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotUser, gotPass, gotBasic = r.BasicAuth()
		if r.URL.Path != "/2.0/repositories/acme" {
			http.NotFound(w, r)
			return
		}
		_ = json.NewEncoder(w).Encode(map[string]any{
			"values": []map[string]any{
				{"slug": "one", "full_name": "acme/one"},
			},
		})
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfileWithAuth("default", "dev@example.com", "token-123", server.URL+"/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"repo", "list", "--workspace", "acme", "--output", "json"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !gotBasic {
		t.Fatal("expected basic auth to be used")
	}
	if gotUser != "dev@example.com" || gotPass != "token-123" {
		t.Fatalf("unexpected basic auth values: %q / %q", gotUser, gotPass)
	}
}

func TestUnknownCommand(t *testing.T) {
	var stdout, stderr bytes.Buffer
	code := Run([]string{"nope"}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit for unknown command, got %d", code)
	}
}

func TestRootHelp(t *testing.T) {
	origVersion := version.Version
	origCommit := version.Commit
	origBuildDate := version.BuildDate
	version.Version = "0.0.1"
	version.Commit = "abc123456789"
	version.BuildDate = "2026-02-23T00:00:00Z"
	defer func() {
		version.Version = origVersion
		version.Commit = origCommit
		version.BuildDate = origBuildDate
	}()

	var stdout, stderr bytes.Buffer
	code := Run(nil, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d", code)
	}
	if !strings.Contains(stdout.String(), "bb - Bitbucket CLI") {
		t.Fatalf("unexpected help output: %q", stdout.String())
	}
	if !strings.Contains(stdout.String(), "Version: 0.0.1+abc1234") {
		t.Fatalf("expected version in help output, got %q", stdout.String())
	}
}

func TestHelpAlias(t *testing.T) {
	var stdout, stderr bytes.Buffer
	code := Run([]string{"help"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d", code)
	}
	if !strings.Contains(stdout.String(), "Commands:") {
		t.Fatalf("unexpected help output: %q", stdout.String())
	}
}

func TestVersionCommand(t *testing.T) {
	origVersion := version.Version
	origCommit := version.Commit
	origBuildDate := version.BuildDate
	version.Version = "0.0.1"
	version.Commit = "abc123456789"
	version.BuildDate = "2026-02-23T00:00:00Z"
	defer func() {
		version.Version = origVersion
		version.Commit = origCommit
		version.BuildDate = origBuildDate
	}()

	var stdout, stderr bytes.Buffer
	code := Run([]string{"version"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	out := stdout.String()
	if !strings.Contains(out, "bb version 0.0.1+abc1234") {
		t.Fatalf("unexpected version output: %q", out)
	}
	if !strings.Contains(out, "commit: abc1234") {
		t.Fatalf("expected commit line in output: %q", out)
	}
	if !strings.Contains(out, "built: 2026-02-23T00:00:00Z") {
		t.Fatalf("expected build date line in output: %q", out)
	}
}

func TestVersionFlag(t *testing.T) {
	origVersion := version.Version
	origCommit := version.Commit
	version.Version = "0.0.1"
	version.Commit = "deadbeef123456"
	defer func() {
		version.Version = origVersion
		version.Commit = origCommit
	}()

	var stdout, stderr bytes.Buffer
	code := Run([]string{"--version"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), "bb version 0.0.1+deadbee") {
		t.Fatalf("unexpected --version output: %q", stdout.String())
	}
}

func TestAuthLoginRequiresToken(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	t.Setenv("BITBUCKET_TOKEN", "")

	var stdout, stderr bytes.Buffer
	code := Run([]string{"auth", "login", "--profile", "default"}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit, stderr=%q", stderr.String())
	}
}

func TestAuthLoginWithTokenFlagValue(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	t.Setenv("BITBUCKET_TOKEN", "")

	var stdout, stderr bytes.Buffer
	code := Run([]string{"auth", "login", "--profile", "default", "--token", "token-from-flag"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected zero exit, got %d, stderr=%q", code, stderr.String())
	}
}

func TestAuthLoginWithTokenFromStdin(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	t.Setenv("BITBUCKET_TOKEN", "")

	oldStdin := os.Stdin
	r, w, err := os.Pipe()
	if err != nil {
		t.Fatalf("pipe failed: %v", err)
	}
	if _, err := w.WriteString("token-from-stdin\n"); err != nil {
		t.Fatalf("write pipe failed: %v", err)
	}
	_ = w.Close()
	os.Stdin = r
	defer func() {
		os.Stdin = oldStdin
		_ = r.Close()
	}()

	var stdout, stderr bytes.Buffer
	code := Run([]string{"auth", "login", "--with-token"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected zero exit, got %d, stderr=%q", code, stderr.String())
	}
}

func TestAuthLoginBareTokenFlagMapsToWithToken(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	t.Setenv("BITBUCKET_TOKEN", "")

	oldStdin := os.Stdin
	r, w, err := os.Pipe()
	if err != nil {
		t.Fatalf("pipe failed: %v", err)
	}
	if _, err := w.WriteString("token-from-stdin\n"); err != nil {
		t.Fatalf("write pipe failed: %v", err)
	}
	_ = w.Close()
	os.Stdin = r
	defer func() {
		os.Stdin = oldStdin
		_ = r.Close()
	}()

	var stdout, stderr bytes.Buffer
	code := Run([]string{"auth", "login", "--token"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected zero exit, got %d, stderr=%q", code, stderr.String())
	}
}

func TestAuthStatusWithoutLogin(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))

	var stdout, stderr bytes.Buffer
	code := Run([]string{"auth", "status"}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit, stderr=%q", stderr.String())
	}
}

func TestAPICommandPaginate(t *testing.T) {
	var server *httptest.Server
	server = httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/2.0/repositories/acme" {
			http.NotFound(w, r)
			return
		}
		if r.URL.Query().Get("page") == "2" {
			_ = json.NewEncoder(w).Encode(map[string]any{
				"values": []map[string]any{{"slug": "two"}},
			})
			return
		}
		_ = json.NewEncoder(w).Encode(map[string]any{
			"values": []map[string]any{{"slug": "one"}},
			"next":   server.URL + "/2.0/repositories/acme?page=2",
		})
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL+"/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"api", "--paginate", "/repositories/acme"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), "\"slug\": \"one\"") || !strings.Contains(stdout.String(), "\"slug\": \"two\"") {
		t.Fatalf("expected paginated values, got %q", stdout.String())
	}
}

func TestRepoListTableOutput(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]any{
			"values": []map[string]any{{"slug": "one", "full_name": "acme/one"}},
		})
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL)
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"repo", "list", "--workspace", "acme"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), "SLUG") || !strings.Contains(stdout.String(), "acme/one") {
		t.Fatalf("unexpected table output: %q", stdout.String())
	}
}

func TestRepoListUnsupportedOutput(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]any{"values": []map[string]any{}})
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL)
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"repo", "list", "--workspace", "acme", "--output", "xml"}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit for unsupported output, stderr=%q", stderr.String())
	}
}

func TestPRListJSON(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/2.0/repositories/acme/app/pullrequests" {
			http.NotFound(w, r)
			return
		}
		_ = json.NewEncoder(w).Encode(map[string]any{
			"values": []map[string]any{
				{
					"id":    12,
					"title": "Add feature",
					"state": "OPEN",
					"source": map[string]any{
						"branch": map[string]any{"name": "feature"},
					},
					"destination": map[string]any{
						"branch": map[string]any{"name": "main"},
					},
				},
			},
		})
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL+"/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "list", "--workspace", "acme", "--repo", "app", "--output", "json"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), "\"title\": \"Add feature\"") {
		t.Fatalf("expected pr list output, got %q", stdout.String())
	}
}

func TestPRCreate(t *testing.T) {
	var gotMethod string
	var gotPath string
	var gotBody map[string]any

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotMethod = r.Method
		gotPath = r.URL.Path
		if err := json.NewDecoder(r.Body).Decode(&gotBody); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		_ = json.NewEncoder(w).Encode(map[string]any{
			"id":    42,
			"title": "Add feature",
			"state": "OPEN",
			"links": map[string]any{
				"html": map[string]any{
					"href": "https://bitbucket.org/acme/app/pull-requests/42",
				},
			},
		})
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL+"/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{
		"pr", "create",
		"--workspace", "acme",
		"--repo", "app",
		"--title", "Add feature",
		"--source", "feature",
		"--destination", "main",
	}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if gotMethod != http.MethodPost {
		t.Fatalf("expected POST method, got %q", gotMethod)
	}
	if gotPath != "/2.0/repositories/acme/app/pullrequests" {
		t.Fatalf("unexpected path: %q", gotPath)
	}
	if gotBody["title"] != "Add feature" {
		t.Fatalf("unexpected body title: %#v", gotBody["title"])
	}
	if !strings.Contains(stdout.String(), "Created PR #42") {
		t.Fatalf("unexpected output: %q", stdout.String())
	}
}

func TestPipelineListJSON(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/2.0/repositories/acme/app/pipelines" {
			http.NotFound(w, r)
			return
		}
		_ = json.NewEncoder(w).Encode(map[string]any{
			"values": []map[string]any{
				{
					"uuid": "{pipeline-1}",
					"state": map[string]any{
						"name": "COMPLETED",
					},
					"target": map[string]any{
						"ref_name": "main",
					},
				},
			},
		})
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL+"/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pipeline", "list", "--workspace", "acme", "--repo", "app", "--output", "json"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), "{pipeline-1}") {
		t.Fatalf("expected pipeline output, got %q", stdout.String())
	}
}

func TestPipelineRun(t *testing.T) {
	var gotMethod string
	var gotPath string
	var gotBody map[string]any

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotMethod = r.Method
		gotPath = r.URL.Path
		if err := json.NewDecoder(r.Body).Decode(&gotBody); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		_ = json.NewEncoder(w).Encode(map[string]any{
			"uuid": "{pipeline-2}",
			"state": map[string]any{
				"name": "PENDING",
			},
			"target": map[string]any{
				"ref_name": "main",
			},
		})
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL+"/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pipeline", "run", "--workspace", "acme", "--repo", "app", "--branch", "main"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if gotMethod != http.MethodPost {
		t.Fatalf("expected POST method, got %q", gotMethod)
	}
	if gotPath != "/2.0/repositories/acme/app/pipelines" {
		t.Fatalf("unexpected path: %q", gotPath)
	}
	target, ok := gotBody["target"].(map[string]any)
	if !ok {
		t.Fatalf("missing target in body: %#v", gotBody)
	}
	if target["ref_name"] != "main" {
		t.Fatalf("unexpected target ref_name: %#v", target["ref_name"])
	}
	if !strings.Contains(stdout.String(), "Triggered pipeline {pipeline-2}") {
		t.Fatalf("unexpected output: %q", stdout.String())
	}
}

func TestCompletionBash(t *testing.T) {
	var stdout, stderr bytes.Buffer
	code := Run([]string{"completion", "bash"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), "complete -F _bb_complete bb") {
		t.Fatalf("unexpected completion output: %q", stdout.String())
	}
}

func TestCompletionUnsupportedShell(t *testing.T) {
	var stdout, stderr bytes.Buffer
	code := Run([]string{"completion", "tcsh"}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero for unsupported shell, stderr=%q", stderr.String())
	}
}

func TestIssueListJSON(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/2.0/repositories/acme/app/issues" {
			http.NotFound(w, r)
			return
		}
		_ = json.NewEncoder(w).Encode(map[string]any{
			"values": []map[string]any{
				{
					"id":       7,
					"title":    "Fix bug",
					"state":    "new",
					"kind":     "bug",
					"priority": "major",
				},
			},
		})
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL+"/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"issue", "list", "--workspace", "acme", "--repo", "app", "--output", "json"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), "\"title\": \"Fix bug\"") {
		t.Fatalf("expected issue output, got %q", stdout.String())
	}
}

func TestIssueListRequiresWorkspace(t *testing.T) {
	var stdout, stderr bytes.Buffer
	code := Run([]string{"issue", "list", "--repo", "app"}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit, stderr=%q", stderr.String())
	}
}

func TestIssueCreate(t *testing.T) {
	var gotMethod string
	var gotPath string
	var gotBody map[string]any

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotMethod = r.Method
		gotPath = r.URL.Path
		if err := json.NewDecoder(r.Body).Decode(&gotBody); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		_ = json.NewEncoder(w).Encode(map[string]any{
			"id":    9,
			"title": "Add docs",
			"state": "new",
			"links": map[string]any{
				"html": map[string]any{
					"href": "https://bitbucket.org/acme/app/issues/9",
				},
			},
		})
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL+"/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{
		"issue", "create",
		"--workspace", "acme",
		"--repo", "app",
		"--title", "Add docs",
		"--content", "body text",
		"--kind", "task",
		"--priority", "minor",
	}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if gotMethod != http.MethodPost {
		t.Fatalf("expected POST method, got %q", gotMethod)
	}
	if gotPath != "/2.0/repositories/acme/app/issues" {
		t.Fatalf("unexpected path: %q", gotPath)
	}
	if gotBody["title"] != "Add docs" {
		t.Fatalf("unexpected body title: %#v", gotBody["title"])
	}
	content, ok := gotBody["content"].(map[string]any)
	if !ok {
		t.Fatalf("missing content in body: %#v", gotBody)
	}
	if content["raw"] != "body text" {
		t.Fatalf("unexpected content raw: %#v", content["raw"])
	}
	if !strings.Contains(stdout.String(), "Created issue #9") {
		t.Fatalf("unexpected output: %q", stdout.String())
	}
}

func TestIssueUpdate(t *testing.T) {
	var gotMethod string
	var gotPath string
	var gotBody map[string]any

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotMethod = r.Method
		gotPath = r.URL.Path
		if err := json.NewDecoder(r.Body).Decode(&gotBody); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		_ = json.NewEncoder(w).Encode(map[string]any{
			"id":    7,
			"title": "Fix bug",
			"state": "resolved",
		})
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL+"/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{
		"issue", "update",
		"--workspace", "acme",
		"--repo", "app",
		"--id", "7",
		"--state", "resolved",
	}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if gotMethod != http.MethodPut {
		t.Fatalf("expected PUT method, got %q", gotMethod)
	}
	if gotPath != "/2.0/repositories/acme/app/issues/7" {
		t.Fatalf("unexpected path: %q", gotPath)
	}
	if gotBody["state"] != "resolved" {
		t.Fatalf("unexpected state body: %#v", gotBody["state"])
	}
	if !strings.Contains(stdout.String(), "Updated issue #7") {
		t.Fatalf("unexpected output: %q", stdout.String())
	}
}

func TestIssueUpdateRequiresAnyField(t *testing.T) {
	var stdout, stderr bytes.Buffer
	code := Run([]string{
		"issue", "update",
		"--workspace", "acme",
		"--repo", "app",
		"--id", "7",
	}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit, stderr=%q", stderr.String())
	}
}

func TestWikiGetText(t *testing.T) {
	requireGit(t)
	remote := initLocalWikiRemote(t, map[string]string{
		"Home.md": "# Hello Wiki\n",
	})

	origBuilder := wikiRemoteURLBuilder
	wikiRemoteURLBuilder = func(_ config.Profile, _, _ string) (string, error) {
		return remote, nil
	}
	defer func() { wikiRemoteURLBuilder = origBuilder }()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"wiki", "get", "--workspace", "acme", "--repo", "app", "--page", "Home.md"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), "Hello Wiki") {
		t.Fatalf("unexpected wiki get output: %q", stdout.String())
	}
}

func TestWikiListJSON(t *testing.T) {
	requireGit(t)
	remote := initLocalWikiRemote(t, map[string]string{
		"Home.md":         "# Home\n",
		"docs/Runbook.md": "runbook\n",
	})

	origBuilder := wikiRemoteURLBuilder
	wikiRemoteURLBuilder = func(_ config.Profile, _, _ string) (string, error) {
		return remote, nil
	}
	defer func() { wikiRemoteURLBuilder = origBuilder }()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"wiki", "list", "--workspace", "acme", "--repo", "app", "--output", "json"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), "\"path\": \"Home.md\"") {
		t.Fatalf("expected Home.md in output, got %q", stdout.String())
	}
	if !strings.Contains(stdout.String(), "\"path\": \"docs/Runbook.md\"") {
		t.Fatalf("expected docs/Runbook.md in output, got %q", stdout.String())
	}
}

func TestWikiPutUpdatesRemote(t *testing.T) {
	requireGit(t)
	remote := initLocalWikiRemote(t, map[string]string{
		"Home.md": "# Old\n",
	})

	origBuilder := wikiRemoteURLBuilder
	wikiRemoteURLBuilder = func(_ config.Profile, _, _ string) (string, error) {
		return remote, nil
	}
	defer func() { wikiRemoteURLBuilder = origBuilder }()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{
		"wiki", "put",
		"--workspace", "acme",
		"--repo", "app",
		"--page", "Home.md",
		"--content", "# Updated\n",
		"--message", "test update",
	}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), "Updated wiki page: Home.md") {
		t.Fatalf("unexpected put output: %q", stdout.String())
	}

	checkoutDir := filepath.Join(t.TempDir(), "checkout")
	runGitLocal(t, "", "clone", "--depth", "1", remote, checkoutDir)
	raw, err := os.ReadFile(filepath.Join(checkoutDir, "Home.md"))
	if err != nil {
		t.Fatalf("read checkout file failed: %v", err)
	}
	if string(raw) != "# Updated\n" {
		t.Fatalf("unexpected wiki content: %q", string(raw))
	}
}

func TestWikiPutRequiresContentOrFile(t *testing.T) {
	var stdout, stderr bytes.Buffer
	code := Run([]string{
		"wiki", "put",
		"--workspace", "acme",
		"--repo", "app",
		"--page", "Home.md",
	}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit, stderr=%q", stderr.String())
	}
}

func TestAuthUnknownSubcommand(t *testing.T) {
	var stdout, stderr bytes.Buffer
	code := Run([]string{"auth", "whoami"}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit, stderr=%q", stderr.String())
	}
}

func TestAuthUsageWithoutSubcommand(t *testing.T) {
	var stdout, stderr bytes.Buffer
	code := Run([]string{"auth"}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit, stderr=%q", stderr.String())
	}
}

func TestRepoUnknownSubcommand(t *testing.T) {
	var stdout, stderr bytes.Buffer
	code := Run([]string{"repo", "remove"}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit, stderr=%q", stderr.String())
	}
}

func requireGit(t *testing.T) {
	t.Helper()
	if _, err := exec.LookPath("git"); err != nil {
		t.Skip("git is not available")
	}
}

func runGitLocal(t *testing.T, dir string, args ...string) {
	t.Helper()
	cmd := exec.Command("git", args...)
	if strings.TrimSpace(dir) != "" {
		cmd.Dir = dir
	}
	out, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatalf("git %s failed: %v\n%s", strings.Join(args, " "), err, string(out))
	}
}

func initLocalWikiRemote(t *testing.T, files map[string]string) string {
	t.Helper()
	base := t.TempDir()
	remote := filepath.Join(base, "remote.git")
	seed := filepath.Join(base, "seed")
	runGitLocal(t, "", "init", "--bare", remote)
	runGitLocal(t, "", "clone", remote, seed)
	runGitLocal(t, seed, "config", "user.name", "tester")
	runGitLocal(t, seed, "config", "user.email", "tester@example.com")
	for rel, content := range files {
		abs := filepath.Join(seed, filepath.FromSlash(rel))
		if err := os.MkdirAll(filepath.Dir(abs), 0o755); err != nil {
			t.Fatalf("mkdir for seed file failed: %v", err)
		}
		if err := os.WriteFile(abs, []byte(content), 0o644); err != nil {
			t.Fatalf("write seed file failed: %v", err)
		}
	}
	runGitLocal(t, seed, "add", ".")
	runGitLocal(t, seed, "commit", "-m", "init")
	runGitLocal(t, seed, "push", "origin", "HEAD")
	return remote
}

func TestAPIUsageErrorWithoutEndpoint(t *testing.T) {
	var stdout, stderr bytes.Buffer
	code := Run([]string{"api"}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit, stderr=%q", stderr.String())
	}
}

func TestAPICommandServerError(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Error(w, "bad request", http.StatusBadRequest)
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL)
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"api", "/x"}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit, stderr=%q", stderr.String())
	}
}

func TestRepoListRequiresWorkspace(t *testing.T) {
	var stdout, stderr bytes.Buffer
	code := Run([]string{"repo", "list"}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit, stderr=%q", stderr.String())
	}
}

func TestRepoListInvalidRowData(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte(`{"values":["bad"]}`))
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL)
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"repo", "list", "--workspace", "acme"}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit, stderr=%q", stderr.String())
	}
}

func TestAPIFailsWhenTokenMissing(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"api", "/repositories/x"}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit, stderr=%q", stderr.String())
	}
}
