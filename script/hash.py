import   hashlib
import  os
import  json

#  更新文件的hash version 和 下载URL

def  update_version_and_url() :
      version_toml= r'A:\Rust_Project\hyperscoop\Cargo.toml'
      with open(version_toml, "r" ,encoding="utf-8") as f:
          data =  f.readlines()
          for line in data:
              if  line.startswith("version") :
                  version =  line.split("=")[1].strip()
                  break
      manifest_path1 = os.path.join(os.path.dirname(os.path.dirname(__file__)),  r"hyperscoop_source_bucket/bucket/hp.json")
      manifest_path2 = r'A:\Scoop\buckets\hp\bucket\hp.json'
      with open(manifest_path1, "r" ,encoding="utf-8") as f:
          data =  json.load(f)
          old_version = data["version"]
          print(data["url"])
          data["version"] = version.replace('"', '')
          data["url"]  ="https://gitee.com/SuperWindcloud/hyperscoop/releases/download/"+version.replace('"', '') +"/hp.exe"
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
        # 使用 sha256 算法计算哈希值
        return hashlib.sha256(f.read()).hexdigest()
def  write_to_manifest(hash_value):
    """Write the hash value to the manifest file"""
    manifest_path = os.path.join(os.path.dirname(os.path.dirname(__file__)),
    r"hyperscoop_source_bucket/bucket/hp.json")
    if not os.path.isfile(manifest_path):
        return None
    with open(manifest_path, "r" ,encoding="utf-8") as f:
        data =  json.load(f)
        data["hash"] = hash_value
        with open(manifest_path, "w", encoding="utf-8") as writer :
                 json.dump(data, writer,  ensure_ascii=False, indent=4)


def  write_scoop_bucket(   hash_value   ) :
     """Write the hash value to the scoop bucket"""
     hyperscoop_bucekt = r'A:\Scoop\buckets\hp\bucket\hp.json'
     if not os.path.isfile(hyperscoop_bucekt):
         return None
     with open(hyperscoop_bucekt, "r" ,encoding="utf-8") as f:
         data =  json.load(f) ; print(data["hash"])
         data["hash"] = hash_value
         with open(hyperscoop_bucekt, "w", encoding="utf-8") as writer :
                 json.dump(data, writer,  ensure_ascii=False, indent=4)



if __name__ == '__main__':
    release_file_path = r"A:\Rust_Project\hyperscoop\target\release\hp.exe"
    result =calculate_hash(release_file_path)
    write_to_manifest(result)  # 将哈希值写入 manifest 文件
    write_scoop_bucket ( result )  # 将哈希值写入 scoop bucket
    update_version_and_url()  # 更新版本号和下载URL

