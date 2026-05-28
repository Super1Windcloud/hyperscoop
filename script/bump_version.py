import argparse
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CARGO_TOML = ROOT / "Cargo.toml"
HP_MANIFEST = ROOT / "hyperscoop_source_bucket" / "bucket" / "hp.json"
RELEASE_BASE_URL = "https://github.com/Super1Windcloud/hyperscoop/releases/download"
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


def update_hp_manifest(version, manifest_path=HP_MANIFEST):
    if not manifest_path.exists():
        raise SystemExit(f"hp manifest 不存在: {manifest_path}")

    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["version"] = version
    manifest["url"] = f"{RELEASE_BASE_URL}/{version}/hp.exe"

    architecture = manifest.setdefault("architecture", {})
    architecture.setdefault("64bit", {})[
        "url"
    ] = f"{RELEASE_BASE_URL}/{version}/hp.exe"
    architecture.setdefault("32bit", {})[
        "url"
    ] = f"{RELEASE_BASE_URL}/{version}/hp-x86-{version}.exe#/hp.exe"
    architecture.setdefault("arm64", {})[
        "url"
    ] = f"{RELEASE_BASE_URL}/{version}/hp-arm64-{version}.exe#/hp.exe"

    manifest_path.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=4) + "\n",
        encoding="utf-8",
    )


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
    update_hp_manifest(next_version)

    subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=ROOT,
        check=True,
        stdout=subprocess.DEVNULL,
    )
    print(next_version)


if __name__ == "__main__":
    main()
