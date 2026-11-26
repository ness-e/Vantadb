"""Offline gate for server.json (MCP Registry, schema 2025-12-11).

Mirrors the normative constraints of
https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json
(draft-07) using stdlib only: no network, no third-party deps, deterministic.
Full jsonschema validation against the downloaded schema stays a manual
owner step (see docs/dev/tasks/FIND-344.md); this gate encodes exactly the
constraints that file must satisfy so CI fails fast on drift.

Exit 0 = VALID, exit 1 = INVALID (errors printed, one per line).
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SERVER_JSON = ROOT / "server.json"

NAME_RE = re.compile(r"^[a-zA-Z0-9.-]+/[a-zA-Z0-9._-]+$")
SHA256_RE = re.compile(r"^[a-f0-9]{64}$")
VERSION_RANGE_RE = re.compile(r"[\^~>=<xX*]|\blatest\b")


def fail(errors, msg):
    errors.append(msg)


def check_server(srv, errors):
    for field in ("name", "description", "version"):
        if field not in srv:
            fail(errors, f"missing required field: {field}")
    name = srv.get("name", "")
    if not (3 <= len(name) <= 200) or not NAME_RE.match(name):
        fail(errors, f"name must match ^[a-zA-Z0-9.-]+/[a-zA-Z0-9._-]+$ (3..200 chars): {name!r}")
    desc = srv.get("description", "")
    if not (1 <= len(desc) <= 100):
        fail(errors, f"description must be 1..100 chars (got {len(desc)})")
    ver = srv.get("version", "")
    if not (1 <= len(str(ver)) <= 255):
        fail(errors, f"version must be 1..255 chars: {ver!r}")
    elif VERSION_RANGE_RE.search(str(ver)):
        fail(errors, f"version must be pinned, not a range: {ver!r}")
    title = srv.get("title")
    if title is not None and not (1 <= len(title) <= 100):
        fail(errors, f"title must be 1..100 chars (got {len(title)})")
    url = srv.get("websiteUrl")
    if url is not None and not url.startswith(("https://", "http://")):
        fail(errors, f"websiteUrl must be a URI: {url!r}")
    repo = srv.get("repository")
    if repo is not None:
        if "url" not in repo or "source" not in repo:
            fail(errors, "repository requires url + source")
    for i, pkg in enumerate(srv.get("packages", [])):
        for field in ("registryType", "identifier", "transport"):
            if field not in pkg:
                fail(errors, f"packages[{i}] missing required field: {field}")
        if pkg.get("registryType") == "mcpb":
            ident = pkg.get("identifier", "")
            if not ident.startswith("https://"):
                fail(errors, f"packages[{i}] mcpb identifier must be an https URL: {ident!r}")
            if not SHA256_RE.match(pkg.get("fileSha256", "")):
                fail(errors, f"packages[{i}] mcpb requires fileSha256 ^[a-f0-9]{{64}}$")
        t = pkg.get("transport", {})
        if "type" not in t:
            fail(errors, f"packages[{i}].transport missing type")
    for i, rem in enumerate(srv.get("remotes", [])):
        if "type" not in rem or "url" not in rem:
            fail(errors, f"remotes[{i}] requires type + url")


def main():
    try:
        srv = json.loads(SERVER_JSON.read_text(encoding="utf-8"))
    except Exception as exc:
        print(f"INVALID: cannot parse server.json: {exc}")
        return 1
    errors = []
    check_server(srv, errors)
    if errors:
        print("INVALID server.json:")
        for err in errors:
            print(f"  - {err}")
        return 1
    print(f"VALID server.json (description {len(srv['description'])} chars, "
          f"{len(srv.get('packages', []))} packages, {len(srv.get('remotes', []))} remotes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
