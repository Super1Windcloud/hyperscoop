import os
def get_version_from_cargo():
      cwd = os.getcwd()
      version_toml=  cwd + r"/Cargo.toml"
      if not os.path.isfile(version_toml):
          print(version_toml)
          return None
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


result  = get_version_from_cargo().strip()
print(result)



