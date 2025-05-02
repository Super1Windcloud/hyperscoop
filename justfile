
set shell := ["pwsh.exe", "-NoProfile", "-c"]

release:
    cargo  br
    just  update_hash
    just  upload


publish_release:
    just  update_hash
    just  upload


publish:
       git add -A  && git commit -m ":panda_face:    publish hp" && git push repo   master  && git push github  master:dev &&  git  push github master:main
       just upload

upload:
     cd script  &&    uv run  publish_release.py  -u

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
    git add -A  && git commit -m ":panda_face:  publish and update hash " && git push repo   master  && git push github  master:dev  &&  git  push github master:main


no_commit_update_hash:
    python  script/hash.py
    cd  hyperscoop_source_bucket  &&  git push repo   master
    git push repo   master  && git push github  master:dev  &&  git  push github master:main

push:
    git add -A  && git commit -m ":panda_face:   update hash " && git push repo   master  && git push github  master:dev

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

