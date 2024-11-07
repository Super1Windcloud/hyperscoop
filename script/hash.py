import   hashlib
import  os
import  json


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
    r"hyperscoop_source_bucket/bucket/hyperscoop.json")
    if not os.path.isfile(manifest_path):
        return None
    with open(manifest_path, "r" ,encoding="utf-8") as f:
        data =  json.load(f)
        data["hash"] = hash_value
        with open(manifest_path, "w", encoding="utf-8") as writer :
                 json.dump(data, writer,  ensure_ascii=False, indent=4)


def  write_scoop_bucket(   hash_value   ) :
     """Write the hash value to the scoop bucket"""
     hyperscoop_bucekt = r'A:\Scoop\buckets\hyperscoop\bucket\hyperscoop.json'
     if not os.path.isfile(hyperscoop_bucekt):
         return None
     with open(hyperscoop_bucekt, "r" ,encoding="utf-8") as f:
         data =  json.load(f) ; print(data["hash"])
         data["hash"] = hash_value
         with open(hyperscoop_bucekt, "w", encoding="utf-8") as writer :
                 json.dump(data, writer,  ensure_ascii=False, indent=4)



if __name__ == '__main__':
    debug_file_path = r"A:\Rust_Project\hyperscoop\target\debug\hyperscoop.exe"
    release_file_path = r"A:\Rust_Project\hyperscoop\target\release\hyperscoop.exe"
    result =calculate_hash(release_file_path)
    result =  calculate_hash(debug_file_path)  if result is None else result
    write_to_manifest(result)  # 将哈希值写入 manifest 文件
    write_scoop_bucket ( result )  # 将哈希值写入 scoop bucket

