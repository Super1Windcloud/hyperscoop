
set shell := ["pwsh.exe", "-NoProfile", "-c"]

release:
    just  update_hash
    cargo build  --release
    python  script/publish_release.py

publish:
    python  script/publish_release.py

upload_hp:
     python  script/publish_release.py   -u

hp  :
    scoop uninstall hp && scoop install  -u -s -k  hp

cross:
   cargo build --target x86_64-pc-windows-msvc --release
   cargo build --target i686-pc-windows-msvc --release
   just  arm64
arm64 :
   cargo build  -q --color auto  --target aarch64-pc-windows-msvc

# 每条命令都是全新的 SHell 环境
update_hash:
    python  script/hash.py
    cd  hyperscoop_source_bucket  &&  just  update
    git add -A  && git commit -m ":panda_face:     update hash " && git push repo   master

push:
    git add -A  && git commit -m ":panda_face:     update hash " && git push repo   master


pull  :
    git pull repo master

musicbox :
  daktilo --preset musicbox


chestra :
 daktilo -p default -p musicbox -p drumkit


real_type:
 daktilo --variate-tempo 0.9,0.4 --variate-volume 0.1,0.5

progress_bar:
   cargo run  --example    pg_bar


count :
    pwsh script/file_count.ps1

