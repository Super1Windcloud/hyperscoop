import argparse
import re
from pathlib import Path

import requests


ROOT = Path(__file__).resolve().parent.parent
OWNER = "Super1Windcloud"
REPO = "hyperscoop"
ASSET_NAME = "hp.exe"
RELEASE_TITLE = "Here We Go!"
WORKSPACE_PACKAGE_RE = re.compile(
    r'(?ms)(?P<prefix>^\[workspace\.package\]\s*)(?P<body>.*?)(?=^\[|\Z)'
)
VERSION_RE = re.compile(r'(?m)^version\s*=\s*"(?P<version>\d+\.\d+\.\d+)"')


def get_version_from_cargo():
    cargo_toml = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    section = WORKSPACE_PACKAGE_RE.search(cargo_toml)
    if section is None:
        return None
    match = VERSION_RE.search(section.group("body"))
    if match is None:
        return None
    return match.group("version")


def get_github_access_token():
    return (ROOT / ".github_token").read_text(encoding="utf-8").strip()


def github_headers(content_type=None):
    headers = {
        "Accept": "application/vnd.github.v3+json",
        "X-GitHub-Api-Version": "2022-11-28",
        "Authorization": f"Bearer {get_github_access_token()}",
    }
    if content_type is not None:
        headers["Content-Type"] = content_type
    return headers


def get_release_by_tag(tag_name):
    url = f"https://api.github.com/repos/{OWNER}/{REPO}/releases/tags/{tag_name}"
    response = requests.get(url, headers=github_headers())
    if response.status_code == 404:
        return None
    response.raise_for_status()
    return response.json()


def create_release(tag_name, name=None, body=None, prerelease=False):
    url = f"https://api.github.com/repos/{OWNER}/{REPO}/releases"
    response = requests.post(
        url,
        headers=github_headers("application/json"),
        json={
            "tag_name": tag_name,
            "name": name or RELEASE_TITLE,
            "body": body or f"Hp '{tag_name}' is published.",
            "draft": False,
            "prerelease": prerelease,
            "target_commitish": "main",
        },
    )
    response.raise_for_status()
    return response.json()


def get_or_create_release(tag_name, name=None, body=None, prerelease=False):
    release = get_release_by_tag(tag_name)
    if release is not None:
        return release
    return create_release(tag_name, name, body, prerelease)


def find_asset(release, asset_name):
    for asset in release.get("assets", []):
        if asset.get("name") == asset_name:
            return asset
    return None


def delete_asset(asset):
    response = requests.delete(asset["url"], headers=github_headers())
    response.raise_for_status()


def upload_asset(release, asset_path, asset_name):
    upload_url = release["upload_url"].split("{", 1)[0]
    response = requests.post(
        f"{upload_url}?name={asset_name}",
        headers=github_headers("application/octet-stream"),
        data=asset_path.read_bytes(),
    )
    response.raise_for_status()
    return response.json()


def init_parser():
    parser = argparse.ArgumentParser(
        description="Publish hp release to GitHub",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""示例用法:
      python publish_release.py -t 1.0.0 -n "新版本" -b "更新说明"
      python publish_release.py -u  # 创建/复用当前版本 release 并上传附件""",
    )
    optional = parser.add_argument_group("Optionals")
    optional.add_argument("-t", "--tag_name", type=str, help="Tag name")
    optional.add_argument("-n", "--name", type=str, help="Release name")
    optional.add_argument("-v", "--version", type=str, help="Version number")
    optional.add_argument("-b", "--body", type=str, help="Release body")
    optional.add_argument(
        "-a",
        "--asset",
        type=Path,
        default=ROOT / "target" / "release" / "hp.exe",
        help="Asset path",
    )

    flags = parser.add_argument_group("Flags")
    flags.add_argument("-p", "--prerelease", action="store_true", help="Is prerelease")
    flags.add_argument(
        "-u",
        "--only_upload_attach_files",
        action="store_true",
        help="Compatibility flag; release is created if missing before upload",
    )
    return parser


def publish(args):
    tag_name = args.tag_name or args.version or get_version_from_cargo()
    if not tag_name:
        raise SystemExit("无法从 Cargo.toml 读取版本号")

    asset_path = args.asset.resolve()
    if not asset_path.exists():
        raise SystemExit(f"附件不存在: {asset_path}")

    release = get_or_create_release(tag_name, args.name, args.body, args.prerelease)
    asset = find_asset(release, ASSET_NAME)
    if asset is not None:
        delete_asset(asset)
    upload_asset(release, asset_path, ASSET_NAME)
    print(f"uploaded {ASSET_NAME} to release {tag_name}")


def main():
    parser = init_parser()
    args = parser.parse_args()
    publish(args)


if __name__ == "__main__":
    main()
