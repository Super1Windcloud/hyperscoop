import argparse
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CARGO_TOML = ROOT / "Cargo.toml"
WORKSPACE_PACKAGE_RE = re.compile(
    r'(?ms)(?P<prefix>^\[workspace\.package\]\s*)(?P<body>.*?)(?=^\[|\Z)'
)
VERSION_RE = re.compile(r'(?m)^version\s*=\s*"(?P<version>\d+\.\d+\.\d+)"')


def parse_args():
    parser = argparse.ArgumentParser(description="Bump workspace package version")
    parser.add_argument(
        "part",
        nargs="?",
        default="patch",
        choices=("major", "minor", "patch"),
        help="Version component to bump",
    )
    return parser.parse_args()


def bump_version(version, part):
    major, minor, patch = (int(item) for item in version.split("."))
    if part == "major":
        return f"{major + 1}.0.0"
    if part == "minor":
        return f"{major}.{minor + 1}.0"
    return f"{major}.{minor}.{patch + 1}"


def main():
    args = parse_args()
    cargo_toml = CARGO_TOML.read_text(encoding="utf-8")
    section = WORKSPACE_PACKAGE_RE.search(cargo_toml)
    if section is None:
        raise SystemExit("Cargo.toml 中没有找到 [workspace.package] 段")

    match = VERSION_RE.search(section.group("body"))
    if match is None:
        raise SystemExit("Cargo.toml 中没有找到 workspace.package version")

    current_version = match.group("version")
    next_version = bump_version(current_version, args.part)
    version_start = section.start("body") + match.start("version")
    version_end = section.start("body") + match.end("version")
    updated = (
        cargo_toml[:version_start]
        + next_version
        + cargo_toml[version_end:]
    )
    CARGO_TOML.write_text(updated, encoding="utf-8", newline="")

    subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=ROOT,
        check=True,
        stdout=subprocess.DEVNULL,
    )
    print(next_version)


if __name__ == "__main__":
    main()
