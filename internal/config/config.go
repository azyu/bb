package config

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

const defaultBaseURL = "https://api.bitbucket.org/2.0"

// Profile contains connection settings for one Bitbucket account/context.
type Profile struct {
	BaseURL  string `json:"base_url"`
	Token    string `json:"token"`
	Username string `json:"username,omitempty"`
}

// Config stores all saved profiles and the currently selected profile name.
type Config struct {
	Current  string             `json:"current"`
	Profiles map[string]Profile `json:"profiles"`
}

func (c *Config) normalize() {
	if c.Profiles == nil {
		c.Profiles = map[string]Profile{}
	}
}

// DefaultPath returns the configuration file path.
func DefaultPath() (string, error) {
	if v := explicitConfigPath(); v != "" {
		return v, nil
	}

	home, err := os.UserHomeDir()
	if err != nil {
		return "", fmt.Errorf("get user home dir: %w", err)
	}

	base := strings.TrimSpace(os.Getenv("XDG_CONFIG_HOME"))
	if base == "" {
		base = filepath.Join(home, ".config")
	}

	return filepath.Join(base, "bb", "config.json"), nil
}

// Load reads config from disk. If it does not exist, it returns an empty config.
func Load() (*Config, error) {
	path, err := DefaultPath()
	if err != nil {
		return nil, err
	}

	raw, err := os.ReadFile(path)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			raw, err = readLegacyConfig()
			if err != nil {
				return nil, err
			}
			if len(raw) == 0 {
				return &Config{Profiles: map[string]Profile{}}, nil
			}
		} else {
			return nil, fmt.Errorf("read config: %w", err)
		}
	}

	return decode(raw)
}

func decode(raw []byte) (*Config, error) {
	var cfg Config
	if err := json.Unmarshal(raw, &cfg); err != nil {
		return nil, fmt.Errorf("decode config: %w", err)
	}
	cfg.normalize()
	return &cfg, nil
}

// Save writes config with restrictive file permissions.
func (c *Config) Save() error {
	path, err := DefaultPath()
	if err != nil {
		return err
	}
	c.normalize()

	if err := os.MkdirAll(filepath.Dir(path), 0o700); err != nil {
		return fmt.Errorf("create config directory: %w", err)
	}

	payload, err := json.MarshalIndent(c, "", "  ")
	if err != nil {
		return fmt.Errorf("encode config: %w", err)
	}

	if err := os.WriteFile(path, payload, 0o600); err != nil {
		return fmt.Errorf("write config: %w", err)
	}
	if err := os.Chmod(path, 0o600); err != nil {
		return fmt.Errorf("chmod config: %w", err)
	}

	return nil
}

// SetProfile upserts a profile and makes it current.
func (c *Config) SetProfile(name, token, baseURL string) {
	c.SetProfileWithAuth(name, "", token, baseURL)
}

// SetProfileWithAuth upserts a profile with optional username and makes it current.
func (c *Config) SetProfileWithAuth(name, username, token, baseURL string) {
	c.normalize()
	if name == "" {
		name = "default"
	}
	if baseURL == "" {
		baseURL = defaultBaseURL
	}

	c.Profiles[name] = Profile{
		BaseURL:  baseURL,
		Token:    token,
		Username: strings.TrimSpace(username),
	}
	c.Current = name
}

// RemoveProfile deletes a profile by name.
// It returns the removed profile name and whether removal happened.
func (c *Config) RemoveProfile(name string) (string, bool) {
	c.normalize()
	target := strings.TrimSpace(name)
	if target == "" {
		target = c.Current
	}
	if target == "" {
		return "", false
	}
	if _, ok := c.Profiles[target]; !ok {
		return target, false
	}

	delete(c.Profiles, target)
	if c.Current == target {
		c.Current = firstProfileName(c.Profiles)
	}
	return target, true
}

// ActiveProfile returns the selected profile, optionally overridden by name.
func (c *Config) ActiveProfile(override string) (Profile, string, error) {
	c.normalize()
	name := override
	if name == "" {
		name = c.Current
	}
	if name == "" {
		return Profile{}, "", errors.New("no active profile")
	}
	p, ok := c.Profiles[name]
	if !ok {
		return Profile{}, "", fmt.Errorf("profile %q not found", name)
	}
	if p.BaseURL == "" {
		p.BaseURL = defaultBaseURL
	}
	return p, name, nil
}

func explicitConfigPath() string {
	return strings.TrimSpace(os.Getenv("BB_CONFIG_PATH"))
}

func firstProfileName(profiles map[string]Profile) string {
	if len(profiles) == 0 {
		return ""
	}
	names := make([]string, 0, len(profiles))
	for name := range profiles {
		names = append(names, name)
	}
	sort.Strings(names)
	return names[0]
}

func readLegacyConfig() ([]byte, error) {
	// If a config path is explicitly set, do not attempt fallback paths.
	if explicitConfigPath() != "" {
		return nil, nil
	}

	legacyBase, err := os.UserConfigDir()
	if err != nil {
		return nil, nil
	}
	legacyPath := filepath.Join(legacyBase, "bb", "config.json")

	newPath, err := DefaultPath()
	if err != nil {
		return nil, nil
	}
	if legacyPath == newPath {
		return nil, nil
	}

	raw, err := os.ReadFile(legacyPath)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			return nil, nil
		}
		return nil, fmt.Errorf("read legacy config: %w", err)
	}
	return raw, nil
}
