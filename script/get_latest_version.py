
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


get_version_from_cargo()



