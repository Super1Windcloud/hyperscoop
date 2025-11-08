import requests
import http.client
import json
import os
from pathlib import Path
import base64
import argparse
import random
import string
from publish import create_new_release

def get_version_from_cargo():
    version_toml = r"A:\Rust_Project\hyperscoop\Cargo.toml"
    with open(version_toml, "r", encoding="utf-8") as f:
        data = f.readlines()
        count = 0
        for line in data:
            line = line.strip()
            if line.startswith("version"):
                count += 1
                if count <= 1:
                    continue
                version = line.split("=", 1)[1].strip().strip("\"'")
                return version
    return None


tag_name = get_version_from_cargo()
release_title = "here we go"
owner = "Super1Windcloud"
repo = "hyperscoop"
asset_name_to_update = "hp.exe"
new_asset_path = r"A:\Rust_Project\hyperscoop\target\release\hp.exe"


def join_paths(base_path, *paths):
    path_obj = Path(base_path)
    for p in paths:
        path_obj = path_obj / p
    return str(path_obj)


def read_file(file_path):
    with open(file_path, "rb") as file:
        file_content = file.read()
    return file_content


def get_hp_bin_path():
    current_file_path = Path(__file__).absolute()
    root = current_file_path.parent.parent
    hp_bin = join_paths(root, "target/release/hp.exe")
    return hp_bin


def get_github_release_id():
    url = f"https://api.github.com/repos/{owner}/{repo}/releases/latest"
    access_token = get_github_access_token()
    headers = {
        "Accept": "application/vnd.github.v3+json",
        "X-GitHub-Api-Version": "2022-11-28",
        "Authorization": f"Bearer  {access_token}",
    }
    response = requests.get(url, headers=headers)
    response.raise_for_status()  # 如果请求失败会抛出异常
    release_data = response.json()
    return release_data["id"]


def get_github_asset_id():
    url = f"https://api.github.com/repos/{owner}/{repo}/releases/latest"
    access_token = get_github_access_token()
    headers = {
        "Accept": "application/vnd.github.v3+json",
        "X-GitHub-Api-Version": "2022-11-28",
        "Authorization": f"Bearer  {access_token}",
    }
    try:
        response = requests.get(url, headers=headers)
        response.raise_for_status()  # 如果请求失败会抛出异常
        release_data = response.json()
        assets = release_data["assets"]
        for asset in assets:
            if asset["name"] == asset_name_to_update:
                return asset["id"]
        return None
    except requests.exceptions.RequestException as e:
        raise Exception(f"获取 release ID 失败: {str(e)}")
    except KeyError:
        raise Exception("解析 release 数据失败，可能是 API 响应格式变化")


def get_github_access_token():
    current_file_path = Path(__file__).absolute()
    root = current_file_path.parent.parent
    hp_bin = join_paths(root, ".github_token")
    with open(hp_bin, "r", encoding="utf-8") as file:
        content = file.read()
    return content.strip()


def init_parser():
    parser = argparse.ArgumentParser(
        description="Publish hp   release to gitee",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""示例用法:
      python publish_release.py -t v1.0.0 -n "新版本" -b "更新说明"
      python publish_release.py -u  # 仅上传附件""",
    )
    optional = parser.add_argument_group("Optionals")
    optional.add_argument("-t", "--tag_name", type=str, help="Tag name")
    optional.add_argument("-n", "--name", type=str, help="Release name")

    optional.add_argument("-v", "--version", type=str, help="Version number")
    optional.add_argument("-b", "--body", type=str, help="Release body")

    flags = parser.add_argument_group("Flags")
    flags.add_argument("-p", "--prerelease", action="store_true", help="Is prerelease")
    flags.add_argument(
        "-u",
        "--only_upload_attach_files",
        action="store_true",
        help="Only upload attach files for release",
    )
    return parser


def upload_github_asset():
    access_token = get_github_access_token()
    headers = {
        "Authorization": f"Bearer  {access_token}",
        "Accept": "application/vnd.github.v3+json",
        "X-GitHub-Api-Version": "2022-11-28",
        "Content-Type": "application/octet-stream",
    }
    release_id = get_github_release_id()
    upload_url = f"https://uploads.github.com/repos/{owner}/{repo}/releases/{release_id}/assets?name={asset_name_to_update}"

    try:
        with open(new_asset_path, "rb") as file:
            response = requests.post(upload_url, headers=headers, data=file)

        response.raise_for_status()
        return response.json()

    except requests.exceptions.RequestException as e:
        raise Exception(f"上传失败: {str(e)}")


def update_release():
    access_token = get_github_access_token()
    headers = {
        "Authorization": f"Bearer  {access_token}",
        "Accept": "application/vnd.github.v3+json",
        "X-GitHub-Api-Version": "2022-11-28",
    }

    release_id = get_github_asset_id()
    print(release_id)
    if release_id is None:
        upload_github_asset()
        return

    url = f"https://api.github.com/repos/{owner}/{repo}/releases/assets/{release_id}"
    response = requests.delete(url, headers=headers)
    upload_github_asset()


def main():
    parser = init_parser()
    args = parser.parse_args()
    if args.only_upload_attach_files:
        update_release()
        return
    create_new_release()


#   upload_hp_to_release(access_token)


if __name__ == "__main__":
    main()
