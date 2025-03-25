
set shell := ["pwsh.exe", "-NoProfile", "-c"]

release:
    just  push
    cargo build  --release



update_hash:
    python  script/hash.py
    git add -A  && git commit -m ":panda_face:     update hash " && git push repo   master

push:
    python   ./script/hash.py
    cd  hyperscoop_source_bucket &&   git add   -A   &&  (git commit -m ":panda_face:   update hash " ) &&  (git push   repo   master )
    cd .. &&   git add    -A   && git commit -m ":panda_face:    update hash" && git push repo   master

pull   :
    git pull repo master

musicbox :
  daktilo --preset musicbox


chestra :
 daktilo -p default -p musicbox -p drumkit


real_type:
 daktilo --variate-tempo 0.9,0.4 --variate-volume 0.1,0.5

progress_bar:
   cargo run  --example    pg_bar




