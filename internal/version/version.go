package version

import "strings"

var (
	// Version should be a SemVer string. Overridden via ldflags at build time.
	Version = "0.0.1"
	// Commit is the git commit hash. Overridden via ldflags at build time.
	Commit = "unknown"
	// BuildDate is an RFC3339 UTC timestamp. Overridden via ldflags at build time.
	BuildDate = "unknown"
)

// ShortCommit returns a stable short hash for user-facing output.
func ShortCommit() string {
	c := strings.TrimSpace(Commit)
	if c == "" || c == "unknown" {
		return "unknown"
	}
	if len(c) > 7 {
		return c[:7]
	}
	return c
}

// DisplayVersion returns the semantic version and short commit build metadata.
func DisplayVersion() string {
	v := strings.TrimSpace(Version)
	if v == "" {
		v = "0.0.1"
	}
	sc := ShortCommit()
	if sc == "unknown" || strings.Contains(v, "+") {
		return v
	}
	return v + "+" + sc
}
