import asyncio
import inspect
import os
import sys
from typing import Dict, List, Tuple

import requests
from googletrans import Translator

from publish import get_access_token


def build_headers() -> Dict[str, str]:
    token = get_access_token() or os.getenv("GITHUB_TOKEN") or os.getenv("GH_TOKEN")
    if token:
        return {"Authorization": f"Bearer {token}"}
    return {}


def get_repos(username: str) -> Tuple[List[str], List[str]]:
    headers = build_headers()
    repos: List[str] = []
    repo_names: List[str] = []
    page = 1

    while True:
        url = f"https://api.github.com/users/{username}/repos"
        params = {"per_page": 100, "page": page}
        resp = requests.get(url, headers=headers, params=params, timeout=30)

        if resp.status_code == 401 and headers:
            print("GitHub rejected the token. Retrying without authentication...")
            headers = {}
            continue

        if resp.status_code != 200:
            print(f"Error fetching data: {resp.status_code}")
            print(resp.text)
            break

        try:
            data = resp.json()
        except ValueError:
            print("Error parsing response as JSON")
            break

        if not isinstance(data, list):
            print("Unexpected response format")
            break

        if not data:
            break

        for repo in data:
            repo_names.append(repo.get("name", ""))
            repos.append(repo.get("description", ""))

        page += 1

    return repos, repo_names


def translate_text(text: str, target_language: str = "zh") -> str:
    translator = Translator()
    try:
        result = translator.translate(text, dest=target_language)
        if inspect.iscoroutine(result):
            result = asyncio.run(result)
        if hasattr(result, "text"):
            return result.text
        raise ValueError("Translation result missing text")
    except Exception as e:
        print(f"Error during translation: {e}")
        return text


def write_output(lines: List[str], filename: str = "repos_output.txt") -> None:
    with open(filename, "w", encoding="utf-8") as file:
        file.write("\n".join(lines))


if __name__ == "__main__":
    username = os.getenv("GITHUB_USERNAME", "Super1WindCloud")
    if len(sys.argv) > 1:
        username = sys.argv[1]

    repos, repos_name = get_repos(username)

    output_lines: List[str] = []
    for desc, name in zip(repos, repos_name):
        if desc:
            translated = translate_text(desc)
            line = f"Repository with description: {name} \n {translated}"
            output_lines.append(line)

        else:
            line = f"!!Repository without description: {name} - {desc}"
            print(line)

    write_output(output_lines)
    print(f"Total repositories fetched: {len(repos)}")
