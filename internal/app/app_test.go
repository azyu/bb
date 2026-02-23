package app

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"path/filepath"
	"strings"
	"testing"

	"bitbucket-cli/internal/config"
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

func TestUnknownCommand(t *testing.T) {
	var stdout, stderr bytes.Buffer
	code := Run([]string{"nope"}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit for unknown command, got %d", code)
	}
}

func TestRootHelp(t *testing.T) {
	var stdout, stderr bytes.Buffer
	code := Run(nil, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d", code)
	}
	if !strings.Contains(stdout.String(), "bb - Bitbucket CLI") {
		t.Fatalf("unexpected help output: %q", stdout.String())
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

func TestAuthLoginRequiresToken(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	t.Setenv("BITBUCKET_TOKEN", "")

	var stdout, stderr bytes.Buffer
	code := Run([]string{"auth", "login", "--profile", "default"}, &stdout, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit, stderr=%q", stderr.String())
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

func TestStubCommandsReturnNonZero(t *testing.T) {
	commands := [][]string{
		{"pr"},
		{"pipeline"},
		{"issue"},
		{"completion"},
	}

	for _, cmd := range commands {
		var stdout, stderr bytes.Buffer
		if code := Run(cmd, &stdout, &stderr); code == 0 {
			t.Fatalf("expected non-zero for %v", cmd)
		}
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
