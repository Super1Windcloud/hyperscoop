import hashlib
import json
import os
import re
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CARGO_TOML = ROOT / "Cargo.toml"
MANIFEST_PATH = ROOT / "hyperscoop_source_bucket" / "bucket" / "hp.json"
RELEASE_BASE_URL = "https://github.com/Super1Windcloud/hyperscoop/releases/download"


def get_version_from_cargo():
    content = CARGO_TOML.read_text(encoding="utf-8")
    match = re.search(r'(?m)^version\s*=\s*"([^"]+)"', content)
    if match:
        return match.group(1)
    return None


def fetch_hash_from_url(url):
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "hyperscoop-hash-updater"})
        with urllib.request.urlopen(req, timeout=15) as resp:
            text = resp.read().decode("utf-8").strip()
            # format is usually "<hash>  filename" or just "<hash>"
            parts = text.split()
            if parts:
                return parts[0].strip().lower()
    except Exception as e:
        print(f"Warning: Failed to fetch hash from {url}: {e}")
    return None


def calculate_local_hash(file_path):
    p = Path(file_path)
    if not p.is_file():
        return None
    with open(p, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest().lower()


def get_hashes(version):
    # Try fetching remote sha256 assets first
    x64_url = f"{RELEASE_BASE_URL}/{version}/hp.exe.sha256"
    x86_url = f"{RELEASE_BASE_URL}/{version}/hp-x86-{version}.exe.sha256"
    arm64_url = f"{RELEASE_BASE_URL}/{version}/hp-arm64-{version}.exe.sha256"

    print(f"Fetching release hashes for version {version} from GitHub...")
    x64 = fetch_hash_from_url(x64_url)
    x86 = fetch_hash_from_url(x86_url)
    arm64 = fetch_hash_from_url(arm64_url)

    # Fallback to local files if remote fails
    if not x64:
        local_x64 = ROOT / "target" / "x86_64-pc-windows-msvc" / "release" / "hp.exe"
        if not local_x64.exists():
            local_x64 = ROOT / "target" / "release" / "hp.exe"
        x64 = calculate_local_hash(local_x64)

    if not x86:
        local_x86 = ROOT / "target" / "i686-pc-windows-msvc" / "release" / "hp.exe"
        x86 = calculate_local_hash(local_x86)

    if not arm64:
        local_arm64 = ROOT / "target" / "aarch64-pc-windows-msvc" / "release" / "hp.exe"
        arm64 = calculate_local_hash(local_arm64)

    return x64, x86, arm64


def update_manifest():
    version = get_version_from_cargo()
    if not version:
        raise SystemExit("Error: Could not determine version from Cargo.toml")

    print(f"Updating manifest for version {version}...")
    x64, x86, arm64 = get_hashes(version)
    print(f"x64:   {x64}")
    print(f"x86:   {x86}")
    print(f"arm64: {arm64}")

    if not x64:
        raise SystemExit("Error: Could not obtain x64 hash.")

    if not MANIFEST_PATH.exists():
        raise SystemExit(f"Error: Manifest path {MANIFEST_PATH} does not exist.")

    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    manifest["version"] = version
    manifest["url"] = f"{RELEASE_BASE_URL}/{version}/hp.exe"
    manifest["hash"] = x64

    arch = manifest.setdefault("architecture", {})
    if x64:
        arch.setdefault("64bit", {})["url"] = f"{RELEASE_BASE_URL}/{version}/hp.exe"
        arch["64bit"]["hash"] = x64
    if x86:
        arch.setdefault("32bit", {})["url"] = f"{RELEASE_BASE_URL}/{version}/hp-x86-{version}.exe#/hp.exe"
        arch["32bit"]["hash"] = x86
    if arm64:
        arch.setdefault("arm64", {})["url"] = f"{RELEASE_BASE_URL}/{version}/hp-arm64-{version}.exe#/hp.exe"
        arch["arm64"]["hash"] = arm64

    MANIFEST_PATH.write_text(json.dumps(manifest, ensure_ascii=False, indent=4) + "\n", encoding="utf-8")
    print(f"Successfully updated {MANIFEST_PATH}")

    # Also update local scoop bucket if present
    local_scoop_hp = Path(r"A:\Scoop\buckets\hp\bucket\hp.json")
    if local_scoop_hp.is_file():
        try:
            local_scoop_hp.write_text(json.dumps(manifest, ensure_ascii=False, indent=4) + "\n", encoding="utf-8")
            print(f"Updated local Scoop bucket at {local_scoop_hp}")
        except Exception as e:
            print(f"Warning: Failed to update local scoop bucket: {e}")


if __name__ == "__main__":
    update_manifest()
