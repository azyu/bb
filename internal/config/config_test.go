package config

import (
	"os"
	"path/filepath"
	"runtime"
	"testing"
)

func TestLoadMissingConfigReturnsEmpty(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", filepath.Join(t.TempDir(), "config.json"))

	cfg, err := Load()
	if err != nil {
		t.Fatalf("Load returned error: %v", err)
	}
	if cfg == nil {
		t.Fatal("Load returned nil config")
	}
	if cfg.Current != "" {
		t.Fatalf("expected empty current profile, got %q", cfg.Current)
	}
	if len(cfg.Profiles) != 0 {
		t.Fatalf("expected no profiles, got %d", len(cfg.Profiles))
	}
}

func TestSaveAndLoadRoundTrip(t *testing.T) {
	configPath := filepath.Join(t.TempDir(), "nested", "config.json")
	t.Setenv("BB_CONFIG_PATH", configPath)

	cfg := &Config{}
	cfg.SetProfile("default", "token-123", "https://api.bitbucket.org/2.0")
	if err := cfg.Save(); err != nil {
		t.Fatalf("Save returned error: %v", err)
	}

	loaded, err := Load()
	if err != nil {
		t.Fatalf("Load returned error: %v", err)
	}

	profile, name, err := loaded.ActiveProfile("")
	if err != nil {
		t.Fatalf("ActiveProfile returned error: %v", err)
	}
	if name != "default" {
		t.Fatalf("expected profile name default, got %q", name)
	}
	if profile.Token != "token-123" {
		t.Fatalf("unexpected token: %q", profile.Token)
	}
	if profile.BaseURL != "https://api.bitbucket.org/2.0" {
		t.Fatalf("unexpected base URL: %q", profile.BaseURL)
	}

	if runtime.GOOS != "windows" {
		info, err := os.Stat(configPath)
		if err != nil {
			t.Fatalf("stat failed: %v", err)
		}
		mode := info.Mode().Perm()
		if mode != 0o600 {
			t.Fatalf("expected file mode 0600, got %o", mode)
		}
	}
}

func TestActiveProfileOverride(t *testing.T) {
	cfg := &Config{}
	cfg.SetProfile("default", "token-a", "https://api.bitbucket.org/2.0")
	cfg.SetProfile("team", "token-b", "https://api.bitbucket.org/2.0")

	profile, name, err := cfg.ActiveProfile("team")
	if err != nil {
		t.Fatalf("ActiveProfile returned error: %v", err)
	}
	if name != "team" {
		t.Fatalf("expected team profile, got %q", name)
	}
	if profile.Token != "token-b" {
		t.Fatalf("expected team token, got %q", profile.Token)
	}
}

func TestActiveProfileErrors(t *testing.T) {
	cfg := &Config{}
	if _, _, err := cfg.ActiveProfile(""); err == nil {
		t.Fatal("expected error for missing active profile")
	}

	cfg.SetProfile("default", "token-a", "")
	if _, _, err := cfg.ActiveProfile("missing"); err == nil {
		t.Fatal("expected error for missing profile override")
	}
}

func TestSetProfileDefaults(t *testing.T) {
	cfg := &Config{}
	cfg.SetProfile("", "token-a", "")

	p, name, err := cfg.ActiveProfile("")
	if err != nil {
		t.Fatalf("ActiveProfile returned error: %v", err)
	}
	if name != "default" {
		t.Fatalf("expected default profile name, got %q", name)
	}
	if p.BaseURL != "https://api.bitbucket.org/2.0" {
		t.Fatalf("unexpected base URL: %q", p.BaseURL)
	}
}

func TestDefaultPathFallback(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", "")
	p, err := DefaultPath()
	if err != nil {
		t.Fatalf("DefaultPath returned error: %v", err)
	}
	if p == "" {
		t.Fatal("expected non-empty default path")
	}
	if filepath.Base(p) != "config.json" {
		t.Fatalf("expected config.json filename, got %q", p)
	}
}
