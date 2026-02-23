package api

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strings"
	"testing"
)

func TestGetAllValuesFollowsNextLinks(t *testing.T) {
	var firstAuth string
	var server *httptest.Server
	server = httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		firstAuth = r.Header.Get("Authorization")
		if r.URL.Path == "/2.0/repositories/ws" && r.URL.Query().Get("page") == "" {
			_ = json.NewEncoder(w).Encode(map[string]any{
				"values": []map[string]any{{"slug": "repo-1"}},
				"next":   server.URL + "/2.0/repositories/ws?page=2",
			})
			return
		}
		if r.URL.Path == "/2.0/repositories/ws" && r.URL.Query().Get("page") == "2" {
			_ = json.NewEncoder(w).Encode(map[string]any{
				"values": []map[string]any{{"slug": "repo-2"}},
			})
			return
		}
		http.NotFound(w, r)
	}))
	defer server.Close()

	client := NewClient(server.URL+"/2.0", "token-123", nil)
	values, err := client.GetAllValues(context.Background(), "/repositories/ws", nil)
	if err != nil {
		t.Fatalf("GetAllValues returned error: %v", err)
	}

	if firstAuth != "Bearer token-123" {
		t.Fatalf("expected bearer auth header, got %q", firstAuth)
	}
	if len(values) != 2 {
		t.Fatalf("expected 2 values, got %d", len(values))
	}
}

func TestDoJSONReturnsErrorOnAPIFailure(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Error(w, "bad request", http.StatusBadRequest)
	}))
	defer server.Close()

	client := NewClient(server.URL, "token-123", nil)
	var out map[string]any
	err := client.DoJSON(context.Background(), http.MethodGet, "/x", nil, nil, &out)
	if err == nil {
		t.Fatal("expected error, got nil")
	}
}

func TestRequestAddsQueryToRelativePath(t *testing.T) {
	var gotQuery string
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotQuery = r.URL.RawQuery
		_ = json.NewEncoder(w).Encode(map[string]any{"ok": true})
	}))
	defer server.Close()

	client := NewClient(server.URL, "token-123", nil)
	resp, err := client.Request(context.Background(), http.MethodGet, "/x", url.Values{"q": []string{"name = \"a\""}}, nil)
	if err != nil {
		t.Fatalf("Request returned error: %v", err)
	}
	_ = resp.Body.Close()

	if gotQuery == "" {
		t.Fatal("expected query string to be set")
	}
}

func TestRequestSupportsAbsoluteURLWithExtraQuery(t *testing.T) {
	var gotQuery url.Values
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotQuery = r.URL.Query()
		_ = json.NewEncoder(w).Encode(map[string]any{"ok": true})
	}))
	defer server.Close()

	client := NewClient(server.URL+"/2.0", "token-123", nil)
	resp, err := client.Request(context.Background(), http.MethodGet, server.URL+"/x?from=next", url.Values{"page": []string{"2"}}, nil)
	if err != nil {
		t.Fatalf("Request returned error: %v", err)
	}
	_ = resp.Body.Close()

	if gotQuery.Get("from") != "next" || gotQuery.Get("page") != "2" {
		t.Fatalf("unexpected merged query: %v", gotQuery.Encode())
	}
}

func TestRequestSetsContentTypeWhenBodyPresent(t *testing.T) {
	var contentType string
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		contentType = r.Header.Get("Content-Type")
		_, _ = w.Write([]byte(`{"ok":true}`))
	}))
	defer server.Close()

	client := NewClient(server.URL, "token-123", nil)
	resp, err := client.Request(context.Background(), http.MethodPost, "/x", nil, strings.NewReader(`{"x":1}`))
	if err != nil {
		t.Fatalf("Request returned error: %v", err)
	}
	_ = resp.Body.Close()

	if contentType != "application/json" {
		t.Fatalf("expected content type application/json, got %q", contentType)
	}
}

func TestDoJSONDecodeError(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte(`{invalid`))
	}))
	defer server.Close()

	client := NewClient(server.URL, "token-123", nil)
	var out map[string]any
	if err := client.DoJSON(context.Background(), http.MethodGet, "/x", nil, nil, &out); err == nil {
		t.Fatal("expected decode error, got nil")
	}
}
