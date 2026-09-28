import argparse
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def run_cmd(cmd, cwd=ROOT, check=True):
    cmd_str = " ".join(cmd) if isinstance(cmd, list) else cmd
    print(f">> Running: {cmd_str}")
    return subprocess.run(cmd, cwd=cwd, shell=isinstance(cmd, str), check=check)


def main():
    parser = argparse.ArgumentParser(
        description="Bump version, commit, tag, and trigger GitHub Action release"
    )
    parser.add_argument(
        "part",
        nargs="?",
        default="patch",
        choices=("major", "minor", "patch"),
        help="Version component to bump (default: patch)",
    )
    args = parser.parse_args()

    # 1. Run bump_version.py
    print(f"Bumping version ({args.part})...")
    res = subprocess.run(
        [sys.executable, str(ROOT / "script" / "bump_version.py"), args.part],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    new_version = res.stdout.strip().splitlines()[-1]
    print(f"New version bumped: {new_version}")

    # 2. Update Cargo.lock
    run_cmd(["cargo", "check", "--workspace"])

    # 3. Update hyperscoop_source_bucket if it's a git repo
    bucket_dir = ROOT / "hyperscoop_source_bucket"
    if (bucket_dir / ".git").exists() or (bucket_dir / "bucket" / "hp.json").exists():
        try:
            run_cmd(["git", "-C", str(bucket_dir), "add", "-A"], check=False)
            run_cmd(
                [
                    "git",
                    "-C",
                    str(bucket_dir),
                    "commit",
                    "-m",
                    f":panda_face: update hp.json to {new_version}",
                ],
                check=False,
            )
            run_cmd(
                ["git", "-C", str(bucket_dir), "push", "origin", "master"],
                check=False,
            )
        except Exception as e:
            print(f"Warning: Failed to push submodule: {e}")

    # 4. Stage and commit changes in root repository
    run_cmd(["git", "add", "-A"])
    commit_msg = f":panda_face:  publish hp {new_version}"
    run_cmd(["git", "commit", "-m", commit_msg])

    # 5. Tag and push to origin
    tag_name = new_version
    print(f"Creating git tag: {tag_name}")
    run_cmd(["git", "tag", "-f", tag_name])

    print("Pushing to origin main and main:dev...")
    run_cmd(["git", "push", "origin", "main"])
    run_cmd(["git", "push", "origin", "main:dev"])

    print(f"Pushing tag {tag_name} to origin...")
    run_cmd(["git", "push", "origin", tag_name, "-f"])

    print(f"\n✅ Successfully bumped version to {new_version} and pushed tag {tag_name}!")
    print("🚀 GitHub Actions 'Hp Release Pipeline' has been triggered to build and publish release.")

    # 6. Wait for GitHub Actions release build to finish and update bucket hashes
    print("\n⏳ Waiting for GitHub Actions build to complete and release assets to be ready...")
    print("   (Will automatically fetch new sha256 hashes and update hyperscoop_source_bucket)")

    import time
    import urllib.request
    from hash import update_manifest

    check_url = f"https://github.com/Super1Windcloud/hyperscoop/releases/download/{tag_name}/hp.exe.sha256"
    max_wait = 600  # 10 minutes max
    start_time = time.time()
    ready = False

    while time.time() - start_time < max_wait:
        try:
            req = urllib.request.Request(
                check_url, headers={"User-Agent": "hyperscoop-release-waiter"}
            )
            with urllib.request.urlopen(req, timeout=10) as resp:
                if resp.status == 200:
                    ready = True
                    break
        except Exception:
            pass
        time.sleep(10)
        elapsed = int(time.time() - start_time)
        print(f"   Waiting for release assets... ({elapsed}s)", end="\r", flush=True)

    if ready:
        print("\n🎉 Release assets found! Updating manifest hashes...")
        update_manifest()
        if (bucket_dir / ".git").exists() or (bucket_dir / "bucket" / "hp.json").exists():
            run_cmd(["git", "-C", str(bucket_dir), "add", "-A"], check=False)
            run_cmd(
                [
                    "git",
                    "-C",
                    str(bucket_dir),
                    "commit",
                    "-m",
                    f":panda_face: update hp.json hashes for {new_version}",
                ],
                check=False,
            )
            run_cmd(
                ["git", "-C", str(bucket_dir), "push", "origin", "master"],
                check=False,
            )
        run_cmd(["git", "add", "-A"], check=False)
        run_cmd(
            [
                "git",
                "commit",
                "-m",
                f":panda_face: update bucket hash pointer for {new_version}",
            ],
            check=False,
        )
        run_cmd(["git", "push", "origin", "main"], check=False)
        run_cmd(["git", "push", "origin", "main:dev"], check=False)
        print("\n✅ Bucket hashes have been automatically updated and pushed to master!")
    else:
        print(
            "\n⚠️ Timed out waiting for release assets. Please run 'just update_hash' once CI finishes."
        )


if __name__ == "__main__":
    main()
