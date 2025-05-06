
import   hashlib
import  os
import http.client
import json
from  publish import  get_access_token
github_owner = "Super1Windcloud"
github_repo = "hyperscoop"
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
     return release_info
  except json.JSONDecodeError:
     print("解析 JSON 响应失败")
     print("原始响应:", data)



def  update_body_title():
     release_info = get_latest_release()
     if not release_info:
             print("无法获取最新 release 信息")
             return None
     release_id = release_info['id']
     old_body = release_info['body']
     access_token = get_access_token()
     conn = http.client.HTTPSConnection("api.github.com")
     headers = {
         "Authorization": f"token {access_token}",
         "User-Agent": "Python-Script",
         "Accept": "application/vnd.github.v3+json",
         "X-GitHub-Api-Version": "2022-11-28",
         "Content-Type": "application/json"
     }
     new_body = old_body + "\n" + """fuck you"""
     # 构建只更新 body 的请求数据
     update_data = {
         "body":  new_body
     }

     url = f"/repos/{github_owner}/{github_repo}/releases/{release_id}"
     conn.request("PATCH", url, body=json.dumps(update_data), headers=headers)
     response = conn.getresponse()
     data = response.read().decode()
     conn.close()

     if response.status == 200:
         return json.loads(data)
     else:
         print(f"更新 release body 失败，状态码: {response.status}")
         print("响应内容:", data)
         return None


def  download_file_then_compute_hash():
     release_info = get_latest_release()
     if not release_info:
             print("无法获取最新 release 信息")
             return None
     assets = release_info['assets']
     hashs =[]
     for asset in assets:
          if asset['name'] != "hp.exe":
               download_url = asset['browser_download_url']
               hash = download(download_url)
               hashs.append(hash)
     return hashs

def  download(download_url):
    try:
         import urllib.request
         with urllib.request.urlopen(download_url) as response, open('temp.exe', 'wb') as out_file:
             while True:
                 chunk = response.read(8192)
                 if not chunk:
                     break
                 out_file.write(chunk)
         print("文件下载成功")
         hash = compute_hash()
         print(f"文件的哈希值: {hash}")
         return hash
    except Exception as e:
         print(f"下载文件时出错: {e}")


def  compute_hash():
     with open('temp.exe', 'rb') as f:
        return hashlib.sha256(f.read()).hexdigest()

if __name__ == "__main__":
     download_file_then_compute_hash()

