import http.client
import json
import os
from pathlib import Path
import base64
import argparse
import random
import string
from  get_latest_version import get_version_from_cargo
tag_name =  get_version_from_cargo()
release_title = "here we go"
github_owner = "Super1Windcloud"
github_repo = "hp"

def join_paths(base_path, *paths):
    path_obj = Path(base_path)
    for p in paths:
        path_obj = path_obj / p
    return str(path_obj)

def read_file(file_path):
    with open(file_path, 'rb') as file:
        file_content = file.read()
    return file_content

def get_hp_bin_path():
    current_file_path = Path(__file__).absolute()
    root = current_file_path.parent.parent
    hp_bin = join_paths(root, "target/x86_64-pc-windows-msvc/release/hp.exe")
    return hp_bin




def upload_hp_to_release( ):
    access_token = get_access_token()
    release_id = get_latest_release().get("id")
    if not release_id:
        print("无法获取 release_id，无法上传文件")
        return

    hp_path = get_hp_bin_path()
    if not os.path.exists(hp_path):
        print(f"文件 {hp_path} 不存在")
        return

    with open(hp_path, 'rb') as f:
        hp_buffer = f.read()

    # GitHub requires the asset name in the URL
    asset_name = "hp.exe"
    host = "uploads.github.com"
    url = f"/repos/{github_owner}/{github_repo}/releases/{release_id}/assets?name={asset_name}"

    headers = {
        "Authorization": f"token {access_token}",
        "User-Agent": "Python-Script",
        "Content-Type": "application/octet-stream",
        "Accept": "application/vnd.github.v3+json"
    }

    conn = http.client.HTTPSConnection(host)
    conn.request("POST", url, body=hp_buffer, headers=headers)

    response = conn.getresponse()
    print("Status:", response.status, response.reason)
    print("Response:", response.read().decode())
    conn.close()

def create_new_release( ):
    access_token = get_access_token()
    host = "api.github.com"
    url = f"/repos/{github_owner}/{github_repo}/releases"
    headers = {
        "Authorization": f"token {access_token}",
        "User-Agent": "Python-Script",
        "Content-Type": "application/json",
        "Accept": "application/vnd.github.v3+json"
    }

    data = {
        "tag_name": tag_name,
        "name": release_title,
        "body": "add hp new feature",
        "draft": False,
        "prerelease": False,
        "target_commitish": "main"
    }

    json_data = json.dumps(data)
    conn = http.client.HTTPSConnection(host)
    conn.request("POST", url, body=json_data, headers=headers)

    response = conn.getresponse()
    data = response.read().decode()
    print("Status:", response.status, response.reason)
    print("Response:", data)
    conn.close()

def get_access_token():
    current_file_path = Path(__file__).absolute()
    root = current_file_path.parent.parent
    env_file = join_paths(root, ".github_token")
    with open(env_file, "r", encoding="utf-8") as file:
        content = file.read()
    print(content)
    return content.strip()  # Remove any extra whitespace/newlines

def init_parser():
    parser = argparse.ArgumentParser(
        description='Publish hp release to GitHub',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog='''示例用法:
    python publish_release.py -t v1.0.0 -n "新版本" -b "更新说明"
    python publish_release.py -u  # 仅上传附件'''
    )
    optional = parser.add_argument_group('Optionals')
    optional.add_argument('-t', '--tag_name', type=str, help='Tag name')
    optional.add_argument('-n', '--name', type=str, help='Release name')
    optional.add_argument('-v', '--version', type=str, help='Version number')
    optional.add_argument('-b', '--body', type=str, help='Release body')

    flags = parser.add_argument_group('Flags')
    flags.add_argument('-p', '--prerelease', action='store_true',
                       help='Is prerelease')
    flags.add_argument('-u', '--only_upload_attach_files', action='store_true',
                       help='Only upload attach files for release')
    return parser


def get_latest_release ():
  access_token = get_access_token()
  conn = http.client.HTTPSConnection("api.github.com")
  headers = {
    "Authorization": f"token {access_token}",
    "User-Agent": "Python-Script",
    "Accept": "application/vnd.github.v3+json",
    "X-GitHub-Api-Version": "2022-11-28"
  }
  url =f"/repos/{github_owner}/{github_repo}/releases/latest"
  conn.request("GET", url, headers=headers)
  response = conn.getresponse()
  data = response.read().decode()
  conn.close()
  try:
     # 解析 JSON 数据
     release_info = json.loads(data )
     print(release_info.get("tag_name"))
     return release_info
  except json.JSONDecodeError:
     print("解析 JSON 响应失败")
     print("原始响应:", data)


def main():
    parser = init_parser()
    args = parser.parse_args()
    if args.tag_name:
        global tag_name
        tag_name = args.tag_name
    if args.name:
        global release_title
        release_title = args.name
    if args.only_upload_attach_files:
        upload_hp_to_release( )
        return
    create_new_release( )
    upload_hp_to_release( )

if __name__ == '__main__':
     main()

