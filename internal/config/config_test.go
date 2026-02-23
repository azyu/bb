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
	t.Setenv("XDG_CONFIG_HOME", "")
	home := t.TempDir()
	t.Setenv("HOME", home)

	p, err := DefaultPath()
	if err != nil {
		t.Fatalf("DefaultPath returned error: %v", err)
	}
	want := filepath.Join(home, ".config", "bb", "config.json")
	if p != want {
		t.Fatalf("expected %q, got %q", want, p)
	}
}

func TestDefaultPathUsesXDGConfigHome(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", "")
	t.Setenv("HOME", t.TempDir())
	xdg := filepath.Join(t.TempDir(), "xdg-config")
	t.Setenv("XDG_CONFIG_HOME", xdg)

	p, err := DefaultPath()
	if err != nil {
		t.Fatalf("DefaultPath returned error: %v", err)
	}
	want := filepath.Join(xdg, "bb", "config.json")
	if p != want {
		t.Fatalf("expected %q, got %q", want, p)
	}
}

func TestLoadFallbackToLegacyPath(t *testing.T) {
	t.Setenv("BB_CONFIG_PATH", "")
	t.Setenv("XDG_CONFIG_HOME", filepath.Join(t.TempDir(), "new-config"))
	t.Setenv("HOME", t.TempDir())

	legacyBase, err := os.UserConfigDir()
	if err != nil {
		t.Fatalf("UserConfigDir returned error: %v", err)
	}
	legacyPath := filepath.Join(legacyBase, "bb", "config.json")
	if err := os.MkdirAll(filepath.Dir(legacyPath), 0o700); err != nil {
		t.Fatalf("mkdir legacy config dir failed: %v", err)
	}
	payload := []byte(`{"current":"default","profiles":{"default":{"base_url":"https://api.bitbucket.org/2.0","token":"legacy-token"}}}`)
	if err := os.WriteFile(legacyPath, payload, 0o600); err != nil {
		t.Fatalf("write legacy config failed: %v", err)
	}

	cfg, err := Load()
	if err != nil {
		t.Fatalf("Load returned error: %v", err)
	}
	p, name, err := cfg.ActiveProfile("")
	if err != nil {
		t.Fatalf("ActiveProfile returned error: %v", err)
	}
	if name != "default" {
		t.Fatalf("expected profile name default, got %q", name)
	}
	if p.Token != "legacy-token" {
		t.Fatalf("expected legacy token, got %q", p.Token)
	}
}
