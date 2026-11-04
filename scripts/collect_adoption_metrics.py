#!/usr/bin/env python3
"""
scripts/collect_adoption_metrics.py

Collects passive adoption metrics across package registries (PyPI, npm, crates.io)
and GitHub repository engagement (stars, forks, releases).
Implements ACT-04.

Usage:
    python scripts/collect_adoption_metrics.py [--dry-run] [--output PATH]
"""

import argparse
import datetime
import json
import os
import sys
import urllib.error
import urllib.request

DEFAULT_OUTPUT_PATH = os.path.join("docs", "dev", "metrics", "adoption.json")
USER_AGENT = "vantadb-metrics-collector/1.0 (+https://github.com/ness-e/Vantadb)"


def http_get_json(url: str, token: str = None) -> dict | None:
    headers = {"User-Agent": USER_AGENT, "Accept": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(url, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            if resp.status == 200:
                return json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as e:
        sys.stderr.write(f"[WARN] HTTP {e.code} for {url}\n")
        return None
    except Exception as e:
        sys.stderr.write(f"[WARN] Failed to fetch {url}: {e}\n")
        return None
    return None


def collect_crates_io() -> dict:
    url = "https://crates.io/api/v1/crates/vantadb"
    data = http_get_json(url)
    if not data or "crate" not in data:
        return {
            "status": "unavailable_or_unreleased",
            "total_downloads": 0,
            "recent_downloads": 0,
            "versions_count": 0,
        }
    cr = data["crate"]
    return {
        "status": "active",
        "name": cr.get("name", "vantadb"),
        "max_version": cr.get("max_version", ""),
        "total_downloads": cr.get("downloads", 0),
        "recent_downloads": cr.get("recent_downloads", 0),
        "created_at": cr.get("created_at", ""),
        "updated_at": cr.get("updated_at", ""),
    }


def collect_pypi() -> dict:
    # First check pypistats for download counts
    stats_url = "https://pypistats.org/api/packages/vantadb/recent"
    stats_data = http_get_json(stats_url)
    pypi_url = "https://pypi.org/pypi/vantadb/json"
    pypi_data = http_get_json(pypi_url)

    if not pypi_data and not stats_data:
        return {
            "status": "unavailable_or_unreleased",
            "last_day": 0,
            "last_week": 0,
            "last_month": 0,
        }

    res = {"status": "active"}
    if stats_data and "data" in stats_data:
        d = stats_data["data"]
        res["last_day"] = d.get("last_day", 0)
        res["last_week"] = d.get("last_week", 0)
        res["last_month"] = d.get("last_month", 0)
    else:
        res["last_day"] = 0
        res["last_week"] = 0
        res["last_month"] = 0

    if pypi_data and "info" in pypi_data:
        res["version"] = pypi_data["info"].get("version", "")
        res["summary"] = pypi_data["info"].get("summary", "")

    return res


def collect_npm() -> dict:
    downloads_url = "https://api.npmjs.org/downloads/point/last-week/@vantadb/client"
    dl_data = http_get_json(downloads_url)
    registry_url = "https://registry.npmjs.org/@vantadb/client"
    reg_data = http_get_json(registry_url)

    if not dl_data and not reg_data:
        return {
            "status": "unavailable_or_unreleased",
            "last_week_downloads": 0,
        }

    res = {"status": "active"}
    if dl_data and "downloads" in dl_data:
        res["last_week_downloads"] = dl_data["downloads"]
    else:
        res["last_week_downloads"] = 0

    if reg_data and "dist-tags" in reg_data:
        res["latest_version"] = reg_data["dist-tags"].get("latest", "")

    return res


def collect_github(repo: str = "ness-e/Vantadb") -> dict:
    token = os.environ.get("GITHUB_TOKEN")
    repo_url = f"https://api.github.com/repos/{repo}"
    data = http_get_json(repo_url, token=token)

    releases_url = f"https://api.github.com/repos/{repo}/releases"
    releases_data = http_get_json(releases_url, token=token)

    if not data:
        return {
            "status": "unavailable_or_rate_limited",
            "stars": 0,
            "forks": 0,
            "watchers": 0,
            "open_issues": 0,
            "releases_total_downloads": 0,
        }

    releases_downloads = 0
    release_count = 0
    if isinstance(releases_data, list):
        release_count = len(releases_data)
        for rel in releases_data:
            for asset in rel.get("assets", []):
                releases_downloads += asset.get("download_count", 0)

    return {
        "status": "active",
        "stars": data.get("stargazers_count", 0),
        "forks": data.get("forks_count", 0),
        "watchers": data.get("subscribers_count", 0),
        "open_issues": data.get("open_issues_count", 0),
        "license": data.get("license", {}).get("spdx_id", "") if data.get("license") else "",
        "releases_count": release_count,
        "releases_total_downloads": releases_downloads,
    }


def main():
    parser = argparse.ArgumentParser(description="Collect VantaDB ecosystem adoption metrics.")
    parser.add_argument("--dry-run", action="store_true", help="Print collected metrics to stdout without writing.")
    parser.add_argument("--output", default=DEFAULT_OUTPUT_PATH, help="Output JSON path.")
    args = parser.parse_args()

    collected_at = datetime.datetime.now(datetime.timezone.utc).isoformat()
    metrics = {
        "schema_version": "1.0.0",
        "collected_at_utc": collected_at,
        "crates_io": collect_crates_io(),
        "pypi": collect_pypi(),
        "npm": collect_npm(),
        "github": collect_github(),
    }

    payload = json.dumps(metrics, indent=2, ensure_ascii=False) + "\n"

    if args.dry_run:
        print(payload)
        return 0

    out_path = os.path.abspath(args.output)
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(payload)

    print(f"[OK] Adoption metrics recorded successfully to {out_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
