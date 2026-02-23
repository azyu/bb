package api

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
)

const defaultUserAgent = "bb-cli/dev"

// Client wraps HTTP calls to the Bitbucket Cloud REST API.
type Client struct {
	baseURL    string
	token      string
	username   string
	userAgent  string
	httpClient *http.Client
}

// APIError carries status code and short response body context.
type APIError struct {
	StatusCode int
	Body       string
}

func (e *APIError) Error() string {
	if e.Body == "" {
		return fmt.Sprintf("api request failed: status %d", e.StatusCode)
	}
	return fmt.Sprintf("api request failed: status %d: %s", e.StatusCode, e.Body)
}

type listResponse struct {
	Values []json.RawMessage `json:"values"`
	Next   string            `json:"next"`
}

// NewClient creates a Bitbucket Cloud API client.
func NewClient(baseURL, token string, httpClient *http.Client) *Client {
	return NewClientWithUser(baseURL, "", token, httpClient)
}

// NewClientWithUser creates a Bitbucket Cloud API client with optional basic-auth username.
func NewClientWithUser(baseURL, username, token string, httpClient *http.Client) *Client {
	if strings.TrimSpace(baseURL) == "" {
		baseURL = "https://api.bitbucket.org/2.0"
	}
	if httpClient == nil {
		httpClient = http.DefaultClient
	}
	return &Client{
		baseURL:    strings.TrimRight(baseURL, "/"),
		token:      token,
		username:   strings.TrimSpace(username),
		userAgent:  defaultUserAgent,
		httpClient: httpClient,
	}
}

// Request performs a raw HTTP request against either a relative API path or absolute URL.
func (c *Client) Request(ctx context.Context, method, path string, query url.Values, body io.Reader) (*http.Response, error) {
	target, err := c.buildURL(path, query)
	if err != nil {
		return nil, err
	}

	req, err := http.NewRequestWithContext(ctx, method, target, body)
	if err != nil {
		return nil, fmt.Errorf("build request: %w", err)
	}
	req.Header.Set("Accept", "application/json")
	req.Header.Set("User-Agent", c.userAgent)
	if c.token != "" {
		if c.username != "" {
			req.SetBasicAuth(c.username, c.token)
		} else {
			req.Header.Set("Authorization", "Bearer "+c.token)
		}
	}
	if body != nil {
		req.Header.Set("Content-Type", "application/json")
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("execute request: %w", err)
	}
	return resp, nil
}

// DoJSON performs a request and decodes a JSON response body into out.
func (c *Client) DoJSON(ctx context.Context, method, path string, query url.Values, body io.Reader, out any) error {
	resp, err := c.Request(ctx, method, path, query, body)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode >= http.StatusBadRequest {
		limited, _ := io.ReadAll(io.LimitReader(resp.Body, 4*1024))
		return &APIError{StatusCode: resp.StatusCode, Body: strings.TrimSpace(string(limited))}
	}
	if out == nil {
		io.Copy(io.Discard, resp.Body)
		return nil
	}
	if err := json.NewDecoder(resp.Body).Decode(out); err != nil {
		return fmt.Errorf("decode response: %w", err)
	}
	return nil
}

// GetAllValues follows Bitbucket pagination and concatenates values from all pages.
func (c *Client) GetAllValues(ctx context.Context, path string, query url.Values) ([]json.RawMessage, error) {
	next := path
	currentQuery := query
	var all []json.RawMessage

	for next != "" {
		var page listResponse
		if err := c.DoJSON(ctx, http.MethodGet, next, currentQuery, nil, &page); err != nil {
			return nil, err
		}
		all = append(all, page.Values...)
		next = page.Next
		currentQuery = nil
	}

	return all, nil
}

func (c *Client) buildURL(path string, query url.Values) (string, error) {
	if strings.HasPrefix(path, "http://") || strings.HasPrefix(path, "https://") {
		u, err := url.Parse(path)
		if err != nil {
			return "", fmt.Errorf("parse absolute URL: %w", err)
		}
		if len(query) > 0 {
			q := u.Query()
			for k, vv := range query {
				for _, v := range vv {
					q.Add(k, v)
				}
			}
			u.RawQuery = q.Encode()
		}
		return u.String(), nil
	}

	base := strings.TrimRight(c.baseURL, "/")
	trimmed := strings.TrimSpace(path)
	if !strings.HasPrefix(trimmed, "/") {
		trimmed = "/" + trimmed
	}

	u, err := url.Parse(base + trimmed)
	if err != nil {
		return "", fmt.Errorf("parse URL: %w", err)
	}

	if len(query) == 0 {
		return u.String(), nil
	}

	q := u.Query()
	for k, vv := range query {
		for _, v := range vv {
			q.Add(k, v)
		}
	}
	u.RawQuery = q.Encode()
	return u.String(), nil
}
