#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Iterable, List

import requests
import tomllib


ROOT = Path(__file__).resolve().parents[1]
CARGO_TOML = ROOT / "Cargo.toml"
DEFAULT_CHANGELOG = ROOT / "CHANGELOG.md"
TOKEN_FILE = ROOT / ".github_token"
DEFAULT_OWNER = "Super1Windcloud"
DEFAULT_REPO = "hyperscoop"
REQUEST_TIMEOUT = 30
BINARY_ALIASES = {
    "hp": ROOT / "target" / "release" / "hp.exe",
    "hp-upx": ROOT / "target" / "release" / "hp_upx.exe",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Create a GitHub release for the hyperscoop workspace.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument("--tag", help="Release tag (defaults to Cargo.toml version).")
    parser.add_argument("--name", help="Release title.")
    parser.add_argument("--notes", help="Release body text; overrides --notes-file.")
    parser.add_argument(
        "--notes-file",
        default=str(DEFAULT_CHANGELOG),
        help="File to read release notes from.",
    )
    parser.add_argument("--owner", default=DEFAULT_OWNER, help="GitHub owner.")
    parser.add_argument("--repo", default=DEFAULT_REPO, help="GitHub repository.")
    parser.add_argument("--target", default="main", help="Target commitish.")
    parser.add_argument("--draft", action="store_true", help="Create a draft release.")
    parser.add_argument(
        "--prerelease",
        action="store_true",
        help="Mark the release as a prerelease.",
    )
    parser.add_argument(
        "--attach",
        action="append",
        default=[],
        metavar="PATH|hp|hp-upx",
        help="Asset path or alias to upload (repeatable).",
    )
    parser.add_argument(
        "--run-just-release",
        action="store_true",
        help="Run `just release` before calling the GitHub API.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show the payload without calling the API.",
    )
    parser.add_argument(
        "--allow-existing",
        action="store_true",
        help="Skip release creation when the tag already exists.",
    )
    return parser.parse_args()


def read_workspace_version() -> str:
    with CARGO_TOML.open("rb") as fp:
        data = tomllib.load(fp)
    workspace_pkg = data.get("workspace", {}).get("package")
    if not workspace_pkg or "version" not in workspace_pkg:
        raise SystemExit("workspace.package.version not found in Cargo.toml.")
    return str(workspace_pkg["version"])


def read_token() -> str:
    try:
        return TOKEN_FILE.read_text(encoding="utf-8").strip()
    except FileNotFoundError as exc:
        raise SystemExit(f"GitHub token file not found: {TOKEN_FILE}") from exc


def default_tag(version: str) -> str:
    return version if version.startswith("v") else f"v{version}"


def extract_notes(path: Path, tag: str) -> str:
    if not path.exists():
        return ""
    content = path.read_text(encoding="utf-8")
    lines = content.splitlines()
    sections: List[tuple[str, list[str]]] = []
    current_header = ""
    current_lines: list[str] = []
    for line in lines:
        if line.startswith("## "):
            if current_lines:
                sections.append((current_header, current_lines))
                current_lines = []
            current_header = line.strip()
        if current_header:
            current_lines.append(line)
    if current_lines:
        sections.append((current_header, current_lines))
    normalized_tag = tag.lstrip("v")
    markers = {
        f"## {tag}",
        f"## v{normalized_tag}",
        f"## {normalized_tag}",
    }
    for header, section_lines in sections:
        if header and any(header.startswith(marker) for marker in markers):
            return "\n".join(section_lines).strip()
    if sections:
        return "\n".join(sections[0][1]).strip()
    return content.strip()


def run_just_release() -> None:
    if shutil.which("just") is None:
        raise SystemExit("`just` is not available in PATH.")
    try:
        subprocess.run(
            ["just", "release"],
            cwd=ROOT,
            check=True,
        )
    except subprocess.CalledProcessError as exc:
        raise SystemExit(f"`just release` failed with exit code {exc.returncode}.") from exc


def resolve_assets(raw_assets: Iterable[str]) -> list[Path]:
    resolved: list[Path] = []
    for entry in raw_assets:
        if not entry:
            continue
        if entry in BINARY_ALIASES:
            path = BINARY_ALIASES[entry]
        else:
            path = Path(entry)
            if not path.is_absolute():
                path = (ROOT / path).resolve()
        if not path.exists():
            raise SystemExit(f"Asset not found: {path}")
        resolved.append(path)
    return resolved


def github_headers(token: str) -> dict[str, str]:
    return {
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
    }


def fetch_release(owner: str, repo: str, tag: str, token: str) -> dict | None:
    url = f"https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}"
    response = requests.get(url, headers=github_headers(token), timeout=REQUEST_TIMEOUT)
    if response.status_code == 404:
        return None
    response.raise_for_status()
    return response.json()


def create_release(
    owner: str,
    repo: str,
    payload: dict[str, object],
    token: str,
) -> dict:
    url = f"https://api.github.com/repos/{owner}/{repo}/releases"
    response = requests.post(
        url,
        headers=github_headers(token),
        data=json.dumps(payload),
        timeout=REQUEST_TIMEOUT,
    )
    response.raise_for_status()
    return response.json()


def upload_assets(release: dict, assets: Iterable[Path], token: str) -> None:
    base_upload_url = release.get("upload_url", "").split("{", 1)[0]
    if not base_upload_url:
        raise SystemExit("upload_url missing from GitHub response.")
    headers = github_headers(token)
    headers["Content-Type"] = "application/octet-stream"
    for asset in assets:
        with asset.open("rb") as fh:
            response = requests.post(
                f"{base_upload_url}?name={asset.name}",
                headers=headers,
                data=fh.read(),
                timeout=REQUEST_TIMEOUT,
            )
        response.raise_for_status()


def main() -> None:
    args = parse_args()
    version = read_workspace_version()
    tag = args.tag or default_tag(version)
    title = args.name or f"hp {version}"
    notes_path = Path(args.notes_file)
    body = args.notes or extract_notes(notes_path, tag)
    if not body:
        body = f"hp {version} release"
    token = read_token()
    if args.run_just_release:
        run_just_release()
    existing = fetch_release(args.owner, args.repo, tag, token)
    if existing and not args.allow_existing:
        raise SystemExit(
            f"A release for tag {tag} already exists (id={existing.get('id')}). "
            "Use --allow-existing to continue."
        )
    payload = {
        "tag_name": tag,
        "name": title,
        "body": body,
        "draft": bool(args.draft),
        "prerelease": bool(args.prerelease),
        "target_commitish": args.target,
    }
    if args.dry_run:
        json.dump(payload, sys.stdout, indent=2, ensure_ascii=False)
        sys.stdout.write("\n")
        return
    release = create_release(args.owner, args.repo, payload, token)
    assets = resolve_assets(args.attach)
    if assets:
        upload_assets(release, assets, token)
    print(f"Release created: {release.get('html_url')}")
    if assets:
        print(f"Uploaded {len(assets)} asset(s).")


if __name__ == "__main__":
    main()
