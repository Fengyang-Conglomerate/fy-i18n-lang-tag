"""Publish bilingual release notes after the matching main CI succeeds."""

import json
import os
from pathlib import Path
import re
import time
import tomllib
from urllib.error import HTTPError
from urllib.parse import urlencode
from urllib.request import Request, urlopen


def version_notes(path, version):
    text = Path(path).read_text(encoding="utf-8")
    sections = re.split(r"(?m)^## ", text)
    matches = [s for s in sections[1:] if re.match(re.escape(version) + r"(?:\s|$)", s)]
    if len(matches) != 1:
        raise RuntimeError(f"Expected exactly one {version} section in {path}")
    section = matches[0].strip()
    if not section.partition("\n")[2].strip():
        raise RuntimeError(f"Empty {version} section in {path}")
    return "## " + section


def release_body(tag):
    if not re.fullmatch(r"v\d+\.\d+\.\d+", tag):
        raise RuntimeError(f"Unsupported release tag: {tag}")
    version = tag[1:]
    package = tomllib.loads(Path("Cargo.toml").read_text(encoding="utf-8"))
    if package["package"]["version"] != version:
        raise RuntimeError("Tag and Cargo.toml versions do not match")
    parts = []
    for language, path in [("简体中文", "CHANGELOG.zh-CN.md"), ("English", "CHANGELOG.md")]:
        parts.append(f"# {language}\n\n{version_notes(path, version)}")
    return "\n\n".join(parts) + "\n"


def api(method, route, data=None):
    request = Request(
        f"{os.environ.get('GITHUB_API_URL', 'https://api.github.com')}/repos/{os.environ['GITHUB_REPOSITORY']}/{route}",
        data=None if data is None else json.dumps(data).encode(),
        method=method,
        headers={
            "Authorization": f"Bearer {os.environ['GH_TOKEN']}",
            "Accept": "application/vnd.github+json",
            "Content-Type": "application/json",
            "X-GitHub-Api-Version": "2022-11-28",
        },
    )
    with urlopen(request, timeout=30) as response:
        return json.load(response)


def wait_for_ci(sha, timeout=1200, interval=15):
    deadline = time.monotonic() + timeout
    query = urlencode({"branch": "main", "event": "push", "head_sha": sha, "per_page": 100})
    while True:
        runs = api("GET", f"actions/workflows/ci.yml/runs?{query}")["workflow_runs"]
        runs = [r for r in runs if r["head_sha"] == sha and r["head_branch"] == "main" and r["event"] == "push"]
        if runs:
            latest = max(runs, key=lambda r: (r["run_number"], r["run_attempt"]))
            if latest["status"] == "completed":
                if latest["conclusion"] != "success":
                    raise RuntimeError(f"Main CI did not succeed: {latest['html_url']}")
                print(f"Verified main CI: {latest['html_url']}", flush=True)
                return
        if time.monotonic() >= deadline:
            raise RuntimeError("Timed out waiting for main CI for the tagged commit")
        print("Waiting for main CI for the tagged commit...", flush=True)
        time.sleep(interval)


def publish(tag, body):
    payload = {"tag_name": tag, "name": f"fy-i18n-lang-tag {tag}", "body": body, "draft": False, "prerelease": False}
    try:
        existing = api("GET", f"releases/tags/{tag}")
    except HTTPError as error:
        if error.code != 404:
            raise
        result = api("POST", "releases", payload)
    else:
        result = api("PATCH", f"releases/{existing['id']}", payload)
    print(f"Published release: {result['html_url']}", flush=True)


def main():
    tag = os.environ["GITHUB_REF_NAME"]
    body = release_body(tag)
    wait_for_ci(os.environ["GITHUB_SHA"])
    publish(tag, body)


if __name__ == "__main__":
    main()
