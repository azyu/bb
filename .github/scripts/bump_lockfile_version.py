"""Set the recorded version of the workspace's own packages in rust/Cargo.lock.

Runs in the release workflow next to the Cargo.toml bump. Editing the lockfile
directly keeps the job free of a Rust toolchain and of registry access, which
`cargo update` needs even with --offline.
"""

import pathlib
import re
import sys

PACKAGES = ("bb-cli", "bb-core")
LOCKFILE = pathlib.Path("rust/Cargo.lock")


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: bump_lockfile_version.py <version>")
    version = sys.argv[1]

    content = LOCKFILE.read_text()
    for package in PACKAGES:
        pattern = rf'(\[\[package\]\]\nname = "{re.escape(package)}"\nversion = )"[^"]*"'
        content, count = re.subn(pattern, rf'\g<1>"{version}"', content)
        if count != 1:
            raise SystemExit(
                f"expected exactly one {package} entry in {LOCKFILE}, found {count}"
            )
    LOCKFILE.write_text(content)
    print(f"{LOCKFILE}: set {', '.join(PACKAGES)} to {version}")


if __name__ == "__main__":
    main()
