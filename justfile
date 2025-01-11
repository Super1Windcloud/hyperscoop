# 使用powershell执行命令

set shell := ["pwsh.exe", "-NoProfile", "-c"]

default:
    cargo build  --release

update_hash:
    python  script/hash.py

push:
    git add -A  && git commit -m ":panda_face:     update " && git push repo   master
