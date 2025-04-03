
import http.client
import json
import  os
from  pathlib  import Path
import  base64
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
     current_file_path =Path(__file__).absolute()
     root = current_file_path.parent.parent
     hp_bin = join_paths(root, "target/release/hp.exe")
     return hp_bin
def  upload_hp_to_release(access_token):

      hp_path = get_hp_bin_path()
      hp_buffer = read_file(hp_path)
      form_data = {
          'file': {
              'filename': 'hp.exe',
              'content_type': 'application/octet-stream',  # 根据实际文件类型修改
              'data': base64.b64encode(hp_buffer).decode('utf-8')  # 转换为 base64 字符串
          }
      }
      host = "gitee.com"
      url = "/api/v5/repos/SuperWindcloud/hyperscoop/releases/1/attach_files"
      headers = {
               "Content-Type": "application/json;charset=UTF-8"
                }
      data = {
               "access_token": access_token ,
               "tag_name": "3.3.4",
               "file":  form_data
           }

      json_data = json.dumps(data)
      conn = http.client.HTTPSConnection(host)
      conn.request("POST", url, body=json_data, headers=headers)
      response = conn.getresponse()
      print("Status:", response.status, response.reason)
      print("Response:", response.read().decode())
      conn.close()

def create_new_release(access_token):
       host = "gitee.com"
       url = "/api/v5/repos/SuperWindcloud/hyperscoop/releases"
       headers = {
           "Content-Type": "application/json;charset=UTF-8"
       }

       data = {
           "access_token": access_token ,
           "tag_name": "3.3.4",
           "name": "here we go ",
           "body": "add install shim feature",
           "prerelease": "false",
           "target_commitish": "master"
       }

       json_data = json.dumps(data)

       conn = http.client.HTTPSConnection(host)
       conn.request("POST", url, body=json_data, headers=headers)

       response = conn.getresponse()
       print("Status:", response.status, response.reason)
       print("Response:", response.read().decode())

       conn.close()

def get_access_token():
   current_file_path =Path(__file__).absolute()
   root = current_file_path.parent.parent
   hp_bin = join_paths(root, ".env")
   with open (hp_bin ,"r" ,encoding="utf-8") as  file:
        content = file.read()
   print(content )
   return content
def main() :
  access_token = get_access_token()
  create_new_release(access_token)
#   upload_hp_to_release(access_token)

main()




