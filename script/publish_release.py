
import http.client
import json
import  os
from  pathlib  import Path

def get_hp_bin_path():
     current_file_path =Path(__file__).absolute()
     root = current_file_path.parent.parent
     hp_bin = root+"target/release/hp.exe"
     return hp_bin
def  upload_hp_to_release(access_token):

      hp_path = get_hp_bin_path()
      host = "gitee.com"
      url = "/api/v5/repos/SuperWindcloud/hyperscoop/releases"
      headers = {
               "Content-Type": "application/json;charset=UTF-8"
                }
      data = {
               "access_token": "1fba69da2f34d7b0b42c6812153d6d12",
               "release_id"  :"",
               "file":  hp
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
           "access_token": "1fba69da2f34d7b0b42c6812153d6d12",
           "tag_name": "3.3.3",
           "name": "here we go ",
           "body": "start to enjoy it ",
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


def main() :
  access_token="1fba69da2f34d7b0b42c6812153d6d12"
  create_new_release(access_token)
  upload_hp_to_release(access_token)



get_hp_bin_path()




