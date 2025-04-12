import   hashlib
import  os
import http.client
import json
def   get_gitee_latest_version () :
      host = "gitee.com"
      url = "/api/v5/repos/SuperWindcloud/hyperscoop/releases/latest"
      headers = {
          "Content-Type": "application/json;charset=UTF-8"
      }

      data = {
          "access_token": "1fba69da2f34d7b0b42c6812153d6d12",
      }

      json_data = json.dumps(data)

      conn = http.client.HTTPSConnection(host)
      conn.request("GET", url, body=json_data, headers=headers)

      response = conn.getresponse()
      print("Status:", response.status, response.reason)
      response = json.loads(response.read().decode())
      conn.close()
      return response.get("tag_name")



def get_version_from_cargo():
      version_toml= r'A:\Rust_Project\hyperscoop\Cargo.toml'
      with open(version_toml, "r" ,encoding="utf-8") as f:
            data =  f.readlines()
            count  =0
            for line in data:
              line = line.strip()
              if  line.startswith("version") :
                     count +=1
                     if  count <=1 :
                            continue
                     version = line.split("=", 1)[1].strip().strip('"\'')
                     return version
      return None
def  update_version_and_url() :
      version=  get_version_from_cargo()
      manifest_path1 = os.path.join(os.path.dirname(os.path.dirname(__file__)),  r"hyperscoop_source_bucket/bucket/hp.json")
      manifest_path2 = r'A:\Scoop\buckets\hp\bucket\hp.json'
      with open(manifest_path1, "r" ,encoding="utf-8") as f:
          data =  json.load(f)
          old_version = data["version"]
          print(data["url"])
          data["version"] = version.replace('"', '')
          data["url"]  = data["url"].replace(old_version,version.replace('"', ''))
          print(data["url"])
          with open(manifest_path2, "w", encoding="utf-8") as writer :
                 json.dump(data, writer , ensure_ascii=False , indent=4)  # 禁用 ASCII 编码以保留非 ASCII 字符（如中文） )



      with open(manifest_path2, "r" ,encoding="utf-8") as f:
              data =  json.load(f)
              old_version = data["version"]
              data["version"] = version.replace('"', '')
              data["url"]  = "https://gitee.com/SuperWindcloud/hyperscoop/releases/download/"+version.replace('"', '') +"/hp.exe"
              with open(manifest_path1, "w", encoding="utf-8") as writer :
                     json.dump(data, writer , ensure_ascii=False , indent=4)  # 禁用 ASCII 编码以保留非 ASCII 字符（如中文） )





def  calculate_hash(file_path):
    """Calculate the hash value of a file"""
    if not os.path.isfile(file_path):
        return None
    with open(file_path, 'rb') as f:
        #  scoop 默认使用sha256 哈希算法
        return hashlib.sha256(f.read()).hexdigest()
def  write_to_manifest(x64 ,arm64,x86  ):
    """Write the hash value to the manifest file"""
    manifest_path = os.path.join(os.path.dirname(os.path.dirname(__file__)),
    r"hyperscoop_source_bucket/bucket/hp.json")
    if not os.path.isfile(manifest_path):
        return None
    with open(manifest_path, "r" ,encoding="utf-8") as f:
        data =  json.load(f)
        data["hash"] = x64
        data["architecture"]["64bit"]["hash"] = x64
        data["architecture"]["arm64"]["hash"] = arm64
        data["architecture"]["32bit"]["hash"] = x86
        data["architecture"]["64bit"]["url"] = data["url"]
        data["architecture"]["arm64"]["url"] = data["url"]
        data["architecture"]["32bit"]["url"] = data["url"]
        with open(manifest_path, "w", encoding="utf-8") as writer :
                 json.dump(data, writer,  ensure_ascii=False, indent=4)


def  write_scoop_bucket(   x64 ,arm64,x86    ) :
     """Write the hash value to the scoop bucket"""
     hyperscoop_bucekt = r'A:\Scoop\buckets\hp\bucket\hp.json'
     if not os.path.isfile(hyperscoop_bucekt):
         return None
     with open(hyperscoop_bucekt, "r" ,encoding="utf-8") as f:
         data =  json.load(f) ; print(data["hash"])
         data["hash"] = x64
         data["architecture"]["64bit"]["hash"] = x64
         data["architecture"]["arm64"]["hash"] = arm64
         data["architecture"]["32bit"]["hash"] = x86
         data["architecture"]["64bit"]["url"] = data["url"]
         data["architecture"]["arm64"]["url"] = data["url"]
         data["architecture"]["32bit"]["url"] = data["url"]
         with open(hyperscoop_bucekt, "w", encoding="utf-8") as writer :
                 json.dump(data, writer,  ensure_ascii=False, indent=4)


def main():
      release_x64 = r"A:\Rust_Project\hyperscoop\target\x86_64-pc-windows-msvc\release\hp.exe"
      release_arm64 = r"A:\Rust_Project\hyperscoop\target\i686-pc-windows-msvc\release\hp.exe"
      release_x86  = r"A:\Rust_Project\hyperscoop\target\aarch64-pc-windows-msvc\release\hp.exe"
      result1 =calculate_hash(release_x64)
      result2 =calculate_hash(release_arm64)
      result3 =calculate_hash(release_x86)
      update_version_and_url()  # 更新版本号和下载URL
      write_to_manifest(result1, result2 ,result3)  # 将哈希值写入 manifest 文件
      write_scoop_bucket ( result1,result2 , result3 )  # 将哈希值写入 scoop bucket


def test():
      version = get_version_from_cargo()
      print(version)


if __name__ == '__main__':
      main()
