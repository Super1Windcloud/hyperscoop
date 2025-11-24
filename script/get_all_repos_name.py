import os
import requests


def build_headers():
    token = os.getenv("GITHUB_TOKEN")
    if token:
        return {"Authorization": f"Bearer {token}"}
    return {}


def get_repos(username):
    url = f"https://api.github.com/users/{username}/repos?per_page=100"
    resp = requests.get(url, headers=build_headers())
    data = resp.json()
    return [repo["description"] for repo in data], [repo["name"] for repo in data]


if __name__ == "__main__":
    username = "Super1WindCloud"
    repos, repos_name = get_repos(username)
    for desc, name in zip(repos, repos_name):
        if not desc:
            print(name)
    print(len(repos))
