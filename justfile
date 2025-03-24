# 使用powershell执行命令

set shell := ["pwsh.exe", "-NoProfile", "-c"]

default:
    cargo build  --release

update_hash:
    python  script/hash.py

push:
    git add -A  && git commit -m ":panda_face:     update " && git push repo   master
pull   :
    git pull repo master

musicbox :
  daktilo --preset musicbox


chestra :
 daktilo -p default -p musicbox -p drumkit


real_type:
 daktilo --variate-tempo 0.9,0.4 --variate-volume 0.1,0.5



