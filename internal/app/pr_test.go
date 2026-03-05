package app

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"path/filepath"
	"strings"
	"testing"

	"bitbucket-cli/internal/config"
)

// ---------- pr view ----------

func TestPRView(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet || r.URL.Path != "/2.0/repositories/acme/app/pullrequests/42" {
			http.NotFound(w, r)
			return
		}
		_ = json.NewEncoder(w).Encode(map[string]any{
			"id":          42,
			"title":       "Add feature",
			"description": "Some description",
			"state":       "OPEN",
			"created_on":  "2026-03-01T10:00:00+00:00",
			"updated_on":  "2026-03-02T12:00:00+00:00",
			"author":      map[string]any{"display_name": "Alice"},
			"source":      map[string]any{"branch": map[string]any{"name": "feature"}},
			"destination": map[string]any{"branch": map[string]any{"name": "main"}},
			"participants": []map[string]any{
				{"user": map[string]any{"display_name": "Bob"}, "role": "REVIEWER", "approved": true},
				{"user": map[string]any{"display_name": "Carol"}, "role": "REVIEWER", "approved": false},
			},
			"comment_count": 3,
			"links":         map[string]any{"html": map[string]any{"href": "https://bitbucket.org/acme/app/pull-requests/42"}},
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
	code := Run([]string{"pr", "view", "--workspace", "acme", "--repo", "app", "--id", "42"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	out := stdout.String()
	if !strings.Contains(out, "Title:        Add feature") {
		t.Fatalf("missing title in output: %q", out)
	}
	if !strings.Contains(out, "Author:       Alice") {
		t.Fatalf("missing author in output: %q", out)
	}
	if !strings.Contains(out, "feature → main") {
		t.Fatalf("missing source → dest in output: %q", out)
	}
	if !strings.Contains(out, "Bob (approved)") {
		t.Fatalf("missing reviewer in output: %q", out)
	}
	if !strings.Contains(out, "Carol (pending)") {
		t.Fatalf("missing reviewer in output: %q", out)
	}
	if !strings.Contains(out, "Some description") {
		t.Fatalf("missing description in output: %q", out)
	}
}

func TestPRViewJSON(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]any{
			"id":    42,
			"title": "Add feature",
			"state": "OPEN",
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
	code := Run([]string{"pr", "view", "--workspace", "acme", "--repo", "app", "--id", "42", "--output", "json"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), `"title": "Add feature"`) {
		t.Fatalf("expected JSON output, got %q", stdout.String())
	}
}

func TestPRViewRequiredID(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "view", "--workspace", "acme", "--repo", "app"}, &stdout, &stderr)
	if code != 1 {
		t.Fatalf("expected exit 1, got %d", code)
	}
	if !strings.Contains(stderr.String(), "--id is required") {
		t.Fatalf("expected --id required error, got %q", stderr.String())
	}
}

// ---------- pr approve ----------

func TestPRApprove(t *testing.T) {
	var gotMethod, gotPath string
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotMethod = r.Method
		gotPath = r.URL.Path
		_ = json.NewEncoder(w).Encode(map[string]any{
			"user":     map[string]any{"display_name": "Alice"},
			"role":     "REVIEWER",
			"approved": true,
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
	code := Run([]string{"pr", "approve", "--workspace", "acme", "--repo", "app", "--id", "42"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if gotMethod != http.MethodPost {
		t.Fatalf("expected POST, got %q", gotMethod)
	}
	if gotPath != "/2.0/repositories/acme/app/pullrequests/42/approve" {
		t.Fatalf("unexpected path: %q", gotPath)
	}
	if !strings.Contains(stdout.String(), "Approved PR #42 in acme/app") {
		t.Fatalf("unexpected output: %q", stdout.String())
	}
}

func TestPRApproveJSON(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]any{
			"user":     map[string]any{"display_name": "Alice"},
			"role":     "REVIEWER",
			"approved": true,
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
	code := Run([]string{"pr", "approve", "--workspace", "acme", "--repo", "app", "--id", "42", "--output", "json"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), `"approved": true`) {
		t.Fatalf("expected JSON output, got %q", stdout.String())
	}
}

func TestPRApproveRequiredID(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "approve", "--workspace", "acme", "--repo", "app"}, &stdout, &stderr)
	if code != 1 {
		t.Fatalf("expected exit 1, got %d", code)
	}
	if !strings.Contains(stderr.String(), "--id is required") {
		t.Fatalf("expected --id required error, got %q", stderr.String())
	}
}

// ---------- pr decline ----------

func TestPRDecline(t *testing.T) {
	var gotMethod, gotPath string
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotMethod = r.Method
		gotPath = r.URL.Path
		_ = json.NewEncoder(w).Encode(map[string]any{
			"id":    42,
			"title": "Add feature",
			"state": "DECLINED",
			"links": map[string]any{"html": map[string]any{"href": "https://bitbucket.org/acme/app/pull-requests/42"}},
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
	code := Run([]string{"pr", "decline", "--workspace", "acme", "--repo", "app", "--id", "42"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if gotMethod != http.MethodPost {
		t.Fatalf("expected POST, got %q", gotMethod)
	}
	if gotPath != "/2.0/repositories/acme/app/pullrequests/42/decline" {
		t.Fatalf("unexpected path: %q", gotPath)
	}
	if !strings.Contains(stdout.String(), "Declined PR #42 (DECLINED)") {
		t.Fatalf("unexpected output: %q", stdout.String())
	}
}

func TestPRDeclineJSON(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]any{
			"id":    42,
			"title": "Add feature",
			"state": "DECLINED",
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
	code := Run([]string{"pr", "decline", "--workspace", "acme", "--repo", "app", "--id", "42", "--output", "json"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), `"state": "DECLINED"`) {
		t.Fatalf("expected JSON output, got %q", stdout.String())
	}
}

func TestPRDeclineRequiredID(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "decline", "--workspace", "acme", "--repo", "app"}, &stdout, &stderr)
	if code != 1 {
		t.Fatalf("expected exit 1, got %d", code)
	}
	if !strings.Contains(stderr.String(), "--id is required") {
		t.Fatalf("expected --id required error, got %q", stderr.String())
	}
}

// ---------- pr edit ----------

func TestPREdit(t *testing.T) {
	var gotMethod, gotPath string
	var gotBody map[string]any
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotMethod = r.Method
		gotPath = r.URL.Path
		_ = json.NewDecoder(r.Body).Decode(&gotBody)
		_ = json.NewEncoder(w).Encode(map[string]any{
			"id":    42,
			"title": "New title",
			"state": "OPEN",
			"links": map[string]any{"html": map[string]any{"href": "https://bitbucket.org/acme/app/pull-requests/42"}},
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
		"pr", "edit",
		"--workspace", "acme",
		"--repo", "app",
		"--id", "42",
		"--title", "New title",
		"--description", "Updated desc",
	}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if gotMethod != http.MethodPut {
		t.Fatalf("expected PUT, got %q", gotMethod)
	}
	if gotPath != "/2.0/repositories/acme/app/pullrequests/42" {
		t.Fatalf("unexpected path: %q", gotPath)
	}
	if gotBody["title"] != "New title" {
		t.Fatalf("unexpected title: %v", gotBody["title"])
	}
	if gotBody["description"] != "Updated desc" {
		t.Fatalf("unexpected description: %v", gotBody["description"])
	}
	if !strings.Contains(stdout.String(), "Updated PR #42 (OPEN): New title") {
		t.Fatalf("unexpected output: %q", stdout.String())
	}
}

func TestPREditJSON(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]any{
			"id":    42,
			"title": "New title",
			"state": "OPEN",
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
	code := Run([]string{"pr", "edit", "--workspace", "acme", "--repo", "app", "--id", "42", "--title", "New title", "--output", "json"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), `"title": "New title"`) {
		t.Fatalf("expected JSON output, got %q", stdout.String())
	}
}

func TestPREditRequiredID(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "edit", "--workspace", "acme", "--repo", "app", "--title", "new"}, &stdout, &stderr)
	if code != 1 {
		t.Fatalf("expected exit 1, got %d", code)
	}
	if !strings.Contains(stderr.String(), "--id is required") {
		t.Fatalf("expected --id required error, got %q", stderr.String())
	}
}

func TestPREditRejectsEmptyUpdate(t *testing.T) {
	hits := 0
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		hits++
		_ = json.NewEncoder(w).Encode(map[string]any{"id": 42})
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL+"/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "edit", "--workspace", "acme", "--repo", "app", "--id", "42"}, &stdout, &stderr)
	if code != 1 {
		t.Fatalf("expected exit 1, got %d", code)
	}
	if !strings.Contains(stderr.String(), "at least one field") {
		t.Fatalf("expected empty update error, got %q", stderr.String())
	}
	if hits != 0 {
		t.Fatalf("expected no API call, got %d hits", hits)
	}
}

// ---------- pr comment ----------

func TestPRComment(t *testing.T) {
	var gotMethod, gotPath string
	var gotBody map[string]any
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotMethod = r.Method
		gotPath = r.URL.Path
		_ = json.NewDecoder(r.Body).Decode(&gotBody)
		_ = json.NewEncoder(w).Encode(map[string]any{
			"id":      5,
			"content": map[string]any{"raw": "LGTM"},
			"user":    map[string]any{"display_name": "Alice"},
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
		"pr", "comment",
		"--workspace", "acme",
		"--repo", "app",
		"--id", "42",
		"--body", "LGTM",
	}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if gotMethod != http.MethodPost {
		t.Fatalf("expected POST, got %q", gotMethod)
	}
	if gotPath != "/2.0/repositories/acme/app/pullrequests/42/comments" {
		t.Fatalf("unexpected path: %q", gotPath)
	}
	content, ok := gotBody["content"].(map[string]any)
	if !ok || content["raw"] != "LGTM" {
		t.Fatalf("unexpected body: %v", gotBody)
	}
	if !strings.Contains(stdout.String(), "Added comment #5 to PR #42") {
		t.Fatalf("unexpected output: %q", stdout.String())
	}
}

func TestPRCommentJSON(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]any{
			"id":      5,
			"content": map[string]any{"raw": "LGTM"},
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
	code := Run([]string{"pr", "comment", "--workspace", "acme", "--repo", "app", "--id", "42", "--body", "LGTM", "--output", "json"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), `"raw": "LGTM"`) {
		t.Fatalf("expected JSON output, got %q", stdout.String())
	}
}

func TestPRCommentRequiredFlags(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	// missing --id
	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "comment", "--workspace", "acme", "--repo", "app", "--body", "LGTM"}, &stdout, &stderr)
	if code != 1 {
		t.Fatalf("expected exit 1, got %d", code)
	}
	if !strings.Contains(stderr.String(), "--id is required") {
		t.Fatalf("expected --id required error, got %q", stderr.String())
	}

	// missing --body
	stdout.Reset()
	stderr.Reset()
	code = Run([]string{"pr", "comment", "--workspace", "acme", "--repo", "app", "--id", "42"}, &stdout, &stderr)
	if code != 1 {
		t.Fatalf("expected exit 1, got %d", code)
	}
	if !strings.Contains(stderr.String(), "--body is required") {
		t.Fatalf("expected --body required error, got %q", stderr.String())
	}
}

// ---------- pr diff ----------

func TestPRDiff(t *testing.T) {
	diffContent := "diff --git a/file.txt b/file.txt\n--- a/file.txt\n+++ b/file.txt\n@@ -1 +1 @@\n-old\n+new\n"
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet || r.URL.Path != "/2.0/repositories/acme/app/pullrequests/42/diff" {
			http.NotFound(w, r)
			return
		}
		w.Header().Set("Content-Type", "text/plain")
		w.Write([]byte(diffContent))
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL+"/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "diff", "--workspace", "acme", "--repo", "app", "--id", "42"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if stdout.String() != diffContent {
		t.Fatalf("expected diff content, got %q", stdout.String())
	}
}

func TestPRDiffError(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Error(w, "PR not found", http.StatusNotFound)
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL+"/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "diff", "--workspace", "acme", "--repo", "app", "--id", "999"}, &stdout, &stderr)
	if code != 1 {
		t.Fatalf("expected exit 1, got %d", code)
	}
	if !strings.Contains(stderr.String(), "status 404") {
		t.Fatalf("expected 404 error, got %q", stderr.String())
	}
}

func TestPRDiffRequiredID(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "diff", "--workspace", "acme", "--repo", "app"}, &stdout, &stderr)
	if code != 1 {
		t.Fatalf("expected exit 1, got %d", code)
	}
	if !strings.Contains(stderr.String(), "--id is required") {
		t.Fatalf("expected --id required error, got %q", stderr.String())
	}
}

// ---------- pr statuses ----------

func TestPRStatuses(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet || r.URL.Path != "/2.0/repositories/acme/app/pullrequests/42/statuses" {
			http.NotFound(w, r)
			return
		}
		_ = json.NewEncoder(w).Encode(map[string]any{
			"values": []map[string]any{
				{"state": "SUCCESSFUL", "name": "build/pipeline", "key": "build", "url": "https://ci.example.com/1"},
				{"state": "INPROGRESS", "name": "", "key": "deploy", "url": "https://ci.example.com/2"},
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
	code := Run([]string{"pr", "statuses", "--workspace", "acme", "--repo", "app", "--id", "42"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	out := stdout.String()
	if !strings.Contains(out, "STATE") || !strings.Contains(out, "NAME") {
		t.Fatalf("missing table headers: %q", out)
	}
	if !strings.Contains(out, "SUCCESSFUL") || !strings.Contains(out, "build/pipeline") {
		t.Fatalf("missing status row: %q", out)
	}
	if !strings.Contains(out, "INPROGRESS") || !strings.Contains(out, "deploy") {
		t.Fatalf("missing status row with key fallback: %q", out)
	}
}

func TestPRStatusesJSON(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]any{
			"values": []map[string]any{
				{"state": "SUCCESSFUL", "name": "build", "key": "build"},
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
	code := Run([]string{"pr", "statuses", "--workspace", "acme", "--repo", "app", "--id", "42", "--output", "json"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), `"state": "SUCCESSFUL"`) {
		t.Fatalf("expected JSON output, got %q", stdout.String())
	}
}

func TestPRStatusesRequiredID(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "statuses", "--workspace", "acme", "--repo", "app"}, &stdout, &stderr)
	if code != 1 {
		t.Fatalf("expected exit 1, got %d", code)
	}
	if !strings.Contains(stderr.String(), "--id is required") {
		t.Fatalf("expected --id required error, got %q", stderr.String())
	}
}

// ---------- pr unapprove ----------

func TestPRUnapprove(t *testing.T) {
	var gotMethod, gotPath string
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotMethod = r.Method
		gotPath = r.URL.Path
		w.WriteHeader(http.StatusNoContent)
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL+"/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "unapprove", "--workspace", "acme", "--repo", "app", "--id", "42"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if gotMethod != http.MethodDelete {
		t.Fatalf("expected DELETE, got %q", gotMethod)
	}
	if gotPath != "/2.0/repositories/acme/app/pullrequests/42/approve" {
		t.Fatalf("unexpected path: %q", gotPath)
	}
	if !strings.Contains(stdout.String(), "Unapproved PR #42 in acme/app") {
		t.Fatalf("unexpected output: %q", stdout.String())
	}
}

func TestPRUnapproveJSON(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNoContent)
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL+"/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "unapprove", "--workspace", "acme", "--repo", "app", "--id", "42", "--output", "json"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), `"status": "unapproved"`) {
		t.Fatalf("expected JSON output, got %q", stdout.String())
	}
}

func TestPRUnapproveRequiredID(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "unapprove", "--workspace", "acme", "--repo", "app"}, &stdout, &stderr)
	if code != 1 {
		t.Fatalf("expected exit 1, got %d", code)
	}
	if !strings.Contains(stderr.String(), "--id is required") {
		t.Fatalf("expected --id required error, got %q", stderr.String())
	}
}

// ---------- pr request-changes ----------

func TestPRRequestChanges(t *testing.T) {
	var gotMethod, gotPath string
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotMethod = r.Method
		gotPath = r.URL.Path
		_ = json.NewEncoder(w).Encode(map[string]any{
			"user":     map[string]any{"display_name": "Alice"},
			"role":     "REVIEWER",
			"approved": false,
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
	code := Run([]string{"pr", "request-changes", "--workspace", "acme", "--repo", "app", "--id", "42"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if gotMethod != http.MethodPost {
		t.Fatalf("expected POST, got %q", gotMethod)
	}
	if gotPath != "/2.0/repositories/acme/app/pullrequests/42/request-changes" {
		t.Fatalf("unexpected path: %q", gotPath)
	}
	if !strings.Contains(stdout.String(), "Requested changes on PR #42 in acme/app") {
		t.Fatalf("unexpected output: %q", stdout.String())
	}
}

func TestPRRequestChangesJSON(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]any{
			"user":     map[string]any{"display_name": "Alice"},
			"role":     "REVIEWER",
			"approved": false,
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
	code := Run([]string{"pr", "request-changes", "--workspace", "acme", "--repo", "app", "--id", "42", "--output", "json"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), `"approved": false`) {
		t.Fatalf("expected JSON output, got %q", stdout.String())
	}
}

func TestPRRequestChangesRequiredID(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "request-changes", "--workspace", "acme", "--repo", "app"}, &stdout, &stderr)
	if code != 1 {
		t.Fatalf("expected exit 1, got %d", code)
	}
	if !strings.Contains(stderr.String(), "--id is required") {
		t.Fatalf("expected --id required error, got %q", stderr.String())
	}
}

// ---------- pr checkout ----------

func TestPRCheckout(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]any{
			"id": 42,
			"source": map[string]any{
				"branch":     map[string]any{"name": "feature/cool"},
				"repository": map[string]any{"full_name": "acme/app"},
			},
			"destination": map[string]any{
				"branch":     map[string]any{"name": "main"},
				"repository": map[string]any{"full_name": "acme/app"},
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

	var gitCalls [][]string
	origRunner := gitCommandRunner
	gitCommandRunner = func(ctx context.Context, dir string, args ...string) ([]byte, error) {
		gitCalls = append(gitCalls, args)
		return []byte("ok"), nil
	}
	defer func() { gitCommandRunner = origRunner }()

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "checkout", "--workspace", "acme", "--repo", "app", "--id", "42"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if len(gitCalls) != 2 {
		t.Fatalf("expected 2 git calls, got %d", len(gitCalls))
	}
	if gitCalls[0][0] != "fetch" || gitCalls[0][1] != "origin" || gitCalls[0][2] != "feature/cool" {
		t.Fatalf("unexpected fetch args: %v", gitCalls[0])
	}
	if gitCalls[1][0] != "checkout" || gitCalls[1][1] != "feature/cool" {
		t.Fatalf("unexpected checkout args: %v", gitCalls[1])
	}
	if !strings.Contains(stdout.String(), "Switched to branch 'feature/cool' from PR #42") {
		t.Fatalf("unexpected output: %q", stdout.String())
	}
}

func TestPRCheckoutFork(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]any{
			"id": 42,
			"source": map[string]any{
				"branch": map[string]any{"name": "feature/cool"},
				"repository": map[string]any{
					"full_name": "contributor/app",
					"links":     map[string]any{"html": map[string]any{"href": "https://bitbucket.org/contributor/app"}},
				},
			},
			"destination": map[string]any{
				"branch":     map[string]any{"name": "main"},
				"repository": map[string]any{"full_name": "acme/app"},
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

	var gitCalls [][]string
	origRunner := gitCommandRunner
	gitCommandRunner = func(ctx context.Context, dir string, args ...string) ([]byte, error) {
		gitCalls = append(gitCalls, args)
		return []byte("ok"), nil
	}
	defer func() { gitCommandRunner = origRunner }()

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "checkout", "--workspace", "acme", "--repo", "app", "--id", "42"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if len(gitCalls) != 2 {
		t.Fatalf("expected 2 git calls, got %d", len(gitCalls))
	}
	// fork: should fetch from fork URL, not origin
	if gitCalls[0][0] != "fetch" || gitCalls[0][1] != "https://bitbucket.org/contributor/app" || gitCalls[0][2] != "feature/cool:feature/cool" {
		t.Fatalf("unexpected fork fetch args: %v", gitCalls[0])
	}
	if gitCalls[1][0] != "checkout" || gitCalls[1][1] != "feature/cool" {
		t.Fatalf("unexpected checkout args: %v", gitCalls[1])
	}
	if !strings.Contains(stdout.String(), "Switched to branch 'feature/cool' from PR #42") {
		t.Fatalf("unexpected output: %q", stdout.String())
	}
}

func TestPRCheckoutRequiredID(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "checkout", "--workspace", "acme", "--repo", "app"}, &stdout, &stderr)
	if code != 1 {
		t.Fatalf("expected exit 1, got %d", code)
	}
	if !strings.Contains(stderr.String(), "--id is required") {
		t.Fatalf("expected --id required error, got %q", stderr.String())
	}
}

func TestPRCheckoutGitFetchFails(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]any{
			"id":     42,
			"source": map[string]any{"branch": map[string]any{"name": "feature/cool"}},
		})
	}))
	defer server.Close()

	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", server.URL+"/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	origRunner := gitCommandRunner
	gitCommandRunner = func(ctx context.Context, dir string, args ...string) ([]byte, error) {
		return nil, fmt.Errorf("remote not found")
	}
	defer func() { gitCommandRunner = origRunner }()

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "checkout", "--workspace", "acme", "--repo", "app", "--id", "42"}, &stdout, &stderr)
	if code != 1 {
		t.Fatalf("expected exit 1, got %d", code)
	}
	if !strings.Contains(stderr.String(), "git fetch failed") {
		t.Fatalf("expected git fetch error, got %q", stderr.String())
	}
}

// ---------- pr activity ----------

func TestPRActivity(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet || r.URL.Path != "/2.0/repositories/acme/app/pullrequests/42/activity" {
			http.NotFound(w, r)
			return
		}
		_ = json.NewEncoder(w).Encode(map[string]any{
			"values": []map[string]any{
				{"update": map[string]any{"state": "OPEN", "date": "2026-03-01T10:00:00+00:00", "author": map[string]any{"display_name": "Alice"}}},
				{"approval": map[string]any{"date": "2026-03-02T12:00:00+00:00", "user": map[string]any{"display_name": "Bob"}}},
				{"comment": map[string]any{"created_on": "2026-03-03T08:00:00+00:00", "user": map[string]any{"display_name": "Carol"}, "content": map[string]any{"raw": "Looks good"}}},
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
	code := Run([]string{"pr", "activity", "--workspace", "acme", "--repo", "app", "--id", "42"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	out := stdout.String()
	if !strings.Contains(out, "TYPE") || !strings.Contains(out, "AUTHOR") {
		t.Fatalf("missing table headers: %q", out)
	}
	if !strings.Contains(out, "update") || !strings.Contains(out, "Alice") {
		t.Fatalf("missing update row: %q", out)
	}
	if !strings.Contains(out, "approval") || !strings.Contains(out, "Bob") {
		t.Fatalf("missing approval row: %q", out)
	}
	if !strings.Contains(out, "comment") || !strings.Contains(out, "Carol") {
		t.Fatalf("missing comment row: %q", out)
	}
}

func TestPRActivityJSON(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]any{
			"values": []map[string]any{
				{"update": map[string]any{"state": "OPEN"}},
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
	code := Run([]string{"pr", "activity", "--workspace", "acme", "--repo", "app", "--id", "42", "--output", "json"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), `"state": "OPEN"`) {
		t.Fatalf("expected JSON output, got %q", stdout.String())
	}
}

func TestPRActivityRequiredID(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "activity", "--workspace", "acme", "--repo", "app"}, &stdout, &stderr)
	if code != 1 {
		t.Fatalf("expected exit 1, got %d", code)
	}
	if !strings.Contains(stderr.String(), "--id is required") {
		t.Fatalf("expected --id required error, got %q", stderr.String())
	}
}

// ---------- pr comments ----------

func TestPRComments(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet || r.URL.Path != "/2.0/repositories/acme/app/pullrequests/42/comments" {
			http.NotFound(w, r)
			return
		}
		_ = json.NewEncoder(w).Encode(map[string]any{
			"values": []map[string]any{
				{"id": 1, "user": map[string]any{"display_name": "Alice"}, "created_on": "2026-03-01T10:00:00+00:00", "content": map[string]any{"raw": "Nice work"}},
				{"id": 2, "user": map[string]any{"display_name": "Bob"}, "created_on": "2026-03-02T12:00:00+00:00", "content": map[string]any{"raw": "LGTM"}},
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
	code := Run([]string{"pr", "comments", "--workspace", "acme", "--repo", "app", "--id", "42"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	out := stdout.String()
	if !strings.Contains(out, "ID") || !strings.Contains(out, "AUTHOR") || !strings.Contains(out, "BODY") {
		t.Fatalf("missing table headers: %q", out)
	}
	if !strings.Contains(out, "Alice") || !strings.Contains(out, "Nice work") {
		t.Fatalf("missing comment row: %q", out)
	}
	if !strings.Contains(out, "Bob") || !strings.Contains(out, "LGTM") {
		t.Fatalf("missing comment row: %q", out)
	}
}

func TestPRCommentsJSON(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]any{
			"values": []map[string]any{
				{"id": 1, "content": map[string]any{"raw": "Nice work"}},
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
	code := Run([]string{"pr", "comments", "--workspace", "acme", "--repo", "app", "--id", "42", "--output", "json"}, &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), `"raw": "Nice work"`) {
		t.Fatalf("expected JSON output, got %q", stdout.String())
	}
}

func TestPRCommentsRequiredID(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	var stdout, stderr bytes.Buffer
	code := Run([]string{"pr", "comments", "--workspace", "acme", "--repo", "app"}, &stdout, &stderr)
	if code != 1 {
		t.Fatalf("expected exit 1, got %d", code)
	}
	if !strings.Contains(stderr.String(), "--id is required") {
		t.Fatalf("expected --id required error, got %q", stderr.String())
	}
}

// ---------- pr merge (non-numeric ID) ----------

func TestPRRejectsNonNumericID(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))
	cfg := &config.Config{}
	cfg.SetProfile("default", "token-123", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("save config failed: %v", err)
	}

	cmds := []string{"view", "approve", "decline", "edit", "comment", "diff", "statuses", "unapprove", "request-changes", "checkout", "activity", "comments"}
	for _, cmd := range cmds {
		t.Run(cmd, func(t *testing.T) {
			args := []string{"pr", cmd, "--workspace", "acme", "--repo", "app", "--id", "abc"}
			if cmd == "edit" {
				args = append(args, "--title", "test")
			}
			if cmd == "comment" {
				args = append(args, "--body", "test")
			}
			var stdout, stderr bytes.Buffer
			code := Run(args, &stdout, &stderr)
			if code != 1 {
				t.Fatalf("expected exit 1, got %d", code)
			}
			if !strings.Contains(stderr.String(), "--id must be a number") {
				t.Fatalf("expected numeric ID error, got %q", stderr.String())
			}
		})
	}
}
