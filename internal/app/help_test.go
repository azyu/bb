package app

import (
	"bytes"
	"strings"
	"testing"
)

func TestGroupHelpFlags(t *testing.T) {
	groups := []struct {
		name    string
		args    []string
		contain string
	}{
		{"auth --help", []string{"auth", "--help"}, "bb auth <command>"},
		{"auth -h", []string{"auth", "-h"}, "bb auth <command>"},
		{"auth help", []string{"auth", "help"}, "bb auth <command>"},
		{"repo --help", []string{"repo", "--help"}, "bb repo <command>"},
		{"repo -h", []string{"repo", "-h"}, "bb repo <command>"},
		{"pr --help", []string{"pr", "--help"}, "bb pr <command>"},
		{"pr help", []string{"pr", "help"}, "bb pr <command>"},
		{"pipeline --help", []string{"pipeline", "--help"}, "bb pipeline <command>"},
		{"issue --help", []string{"issue", "--help"}, "bb issue <command>"},
		{"issue -h", []string{"issue", "-h"}, "bb issue <command>"},
		{"wiki --help", []string{"wiki", "--help"}, "bb wiki <command>"},
		{"wiki help", []string{"wiki", "help"}, "bb wiki <command>"},
		{"completion --help", []string{"completion", "--help"}, "bb completion <shell>"},
		{"completion -h", []string{"completion", "-h"}, "bb completion <shell>"},
	}
	for _, tc := range groups {
		t.Run(tc.name, func(t *testing.T) {
			var stdout, stderr bytes.Buffer
			code := Run(tc.args, &stdout, &stderr)
			if code != 0 {
				t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
			}
			if !strings.Contains(stdout.String(), tc.contain) {
				t.Fatalf("expected stdout to contain %q, got %q", tc.contain, stdout.String())
			}
		})
	}
}

func TestGroupNoArgsShowsHelp(t *testing.T) {
	groups := []struct {
		name    string
		args    []string
		contain string
	}{
		{"auth", []string{"auth"}, "bb auth <command>"},
		{"repo", []string{"repo"}, "bb repo <command>"},
		{"pr", []string{"pr"}, "bb pr <command>"},
		{"pipeline", []string{"pipeline"}, "bb pipeline <command>"},
		{"issue", []string{"issue"}, "bb issue <command>"},
		{"wiki", []string{"wiki"}, "bb wiki <command>"},
		{"completion", []string{"completion"}, "bb completion <shell>"},
	}
	for _, tc := range groups {
		t.Run(tc.name, func(t *testing.T) {
			var stdout, stderr bytes.Buffer
			code := Run(tc.args, &stdout, &stderr)
			if code != 0 {
				t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
			}
			if !strings.Contains(stdout.String(), tc.contain) {
				t.Fatalf("expected stdout to contain %q, got %q", tc.contain, stdout.String())
			}
		})
	}
}

func TestLeafHelpFlags(t *testing.T) {
	leaves := []struct {
		name    string
		args    []string
		contain string
	}{
		{"auth login --help", []string{"auth", "login", "--help"}, "--token"},
		{"auth login -h", []string{"auth", "login", "-h"}, "--profile"},
		{"auth status --help", []string{"auth", "status", "--help"}, "--profile"},
		{"auth logout --help", []string{"auth", "logout", "--help"}, "--profile"},
		{"api --help", []string{"api", "--help"}, "--method"},
		{"api -h", []string{"api", "-h"}, "--paginate"},
		{"repo list --help", []string{"repo", "list", "--help"}, "--workspace"},
		{"repo list -h", []string{"repo", "list", "-h"}, "--output"},
		{"pr list --help", []string{"pr", "list", "--help"}, "--workspace"},
		{"pr list -h", []string{"pr", "list", "-h"}, "--state"},
		{"pr create --help", []string{"pr", "create", "--help"}, "--title"},
		{"pr create -h", []string{"pr", "create", "-h"}, "--source"},
		{"pipeline list --help", []string{"pipeline", "list", "--help"}, "--workspace"},
		{"pipeline run --help", []string{"pipeline", "run", "--help"}, "--branch"},
		{"issue list --help", []string{"issue", "list", "--help"}, "--workspace"},
		{"issue create --help", []string{"issue", "create", "--help"}, "--title"},
		{"issue update --help", []string{"issue", "update", "--help"}, "--id"},
		{"wiki list --help", []string{"wiki", "list", "--help"}, "--workspace"},
		{"wiki get --help", []string{"wiki", "get", "--help"}, "--page"},
		{"wiki put --help", []string{"wiki", "put", "--help"}, "--content"},
	}
	for _, tc := range leaves {
		t.Run(tc.name, func(t *testing.T) {
			var stdout, stderr bytes.Buffer
			code := Run(tc.args, &stdout, &stderr)
			if code != 0 {
				t.Fatalf("expected exit 0, got %d, stderr=%q", code, stderr.String())
			}
			if !strings.Contains(stdout.String(), tc.contain) {
				t.Fatalf("expected stdout to contain %q, got %q", tc.contain, stdout.String())
			}
		})
	}
}

func TestHelpOutputUsesDoubleDash(t *testing.T) {
	leaves := []struct {
		name string
		args []string
	}{
		{"pr list", []string{"pr", "list", "--help"}},
		{"pr create", []string{"pr", "create", "--help"}},
		{"issue create", []string{"issue", "create", "--help"}},
		{"wiki put", []string{"wiki", "put", "--help"}},
	}
	for _, tc := range leaves {
		t.Run(tc.name, func(t *testing.T) {
			var stdout, stderr bytes.Buffer
			code := Run(tc.args, &stdout, &stderr)
			if code != 0 {
				t.Fatalf("expected exit 0, got %d", code)
			}
			for _, line := range strings.Split(stdout.String(), "\n") {
				trimmed := strings.TrimSpace(line)
				if trimmed == "" {
					continue
				}
				if strings.HasPrefix(trimmed, "-") && !strings.HasPrefix(trimmed, "--") {
					t.Fatalf("found single-dash long flag in help output: %q", line)
				}
			}
		})
	}
}

func TestIsHelpArg(t *testing.T) {
	for _, arg := range []string{"-h", "--help", "help"} {
		if !isHelpArg(arg) {
			t.Errorf("expected isHelpArg(%q) to be true", arg)
		}
	}
	for _, arg := range []string{"-help", "h", "--h", "list", ""} {
		if isHelpArg(arg) {
			t.Errorf("expected isHelpArg(%q) to be false", arg)
		}
	}
}
