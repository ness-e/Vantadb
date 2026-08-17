#!/usr/bin/env python3
"""Functional test for the VantaDB MCP server over stdio JSON-RPC.

Spawns ONE server process and drives the MCP handshake sequentially:
initialize -> tools/list -> resources/list -> prompts/list.

Exit 0 if every request returns a valid JSON-RPC result; exit 1 otherwise.

The server binary is resolved from (in order):
  1. argv[1] or the VANTADB_MCP_BIN env var (explicit path)
  2. `vanta-cli` on PATH (must support `server --mcp`)
  3. `target/debug/vanta-cli.exe` (Windows dev builds)
  4. `vantadb-server` on PATH (supports `--mcp` directly)
  5. `target/debug/vantadb-server.exe`
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile

# Windows consoles default to cp1252, which cannot encode emoji output.
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")


def _candidate_paths(explicit):
    yield explicit
    yield shutil.which("vanta-cli")
    yield os.path.join("target", "debug", "vanta-cli.exe")
    yield os.path.join("target", "release", "vanta-cli.exe")
    yield shutil.which("vantadb-server")
    yield os.path.join("target", "debug", "vantadb-server.exe")
    yield os.path.join("target", "release", "vantadb-server.exe")


def _is_vanta_cli(bin_path):
    return subprocess.run(
        [bin_path, "server", "--help"], capture_output=True, timeout=10
    ).returncode == 0


def resolve_server(explicit):
    """Return (binary, kind) where kind is 'vanta-cli' or 'vantadb-server'."""
    for cand in _candidate_paths(explicit):
        if not cand or not os.path.isfile(cand):
            continue
        name = os.path.basename(cand).lower()
        if "vanta-cli" in name:
            # Old vanta-cli builds lack the `server` subcommand — skip them
            # so a stale PATH install does not shadow a working binary.
            if _is_vanta_cli(cand):
                return cand, "vanta-cli"
            continue
        if "vantadb-server" in name:
            return cand, "vantadb-server"
        # Unknown binary: accept only if it answers --help (best effort).
        if subprocess.run([cand, "--help"], capture_output=True, timeout=10).returncode == 0:
            return cand, "vantadb-server"
    raise RuntimeError(
        "No usable VantaDB MCP server binary found. "
        "Pass one via argv or the VANTADB_MCP_BIN env var."
    )


class McpSession:
    """One stdio MCP server process; all requests share it."""

    def __init__(self, binary, kind):
        self._db = tempfile.mkdtemp(prefix="vantadb-test-")
        if kind == "vanta-cli":
            cmd = [binary, "server", "--mcp", "--db", self._db]
            env = None
        else:
            cmd = [binary, "--mcp"]
            env = dict(os.environ, VANTADB_STORAGE_PATH=self._db)
        self._proc = subprocess.Popen(
            cmd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
            env=env,
        )

    def request(self, method, _id, params=None):
        req = {"jsonrpc": "2.0", "id": _id, "method": method}
        if params is not None:
            req["params"] = params
        self._proc.stdin.write(json.dumps(req) + "\n")
        self._proc.stdin.flush()
        line = self._proc.stdout.readline()
        if not line:
            stderr = self._proc.stderr.read() if self._proc.stderr else ""
            raise RuntimeError(
                f"Server closed stdout before responding to {method}. stderr: {stderr}"
            )
        return json.loads(line)

    def close(self):
        self._proc.stdin.close()
        self._proc.wait(timeout=15)


def main():
    explicit = sys.argv[1] if len(sys.argv) > 1 else os.environ.get("VANTADB_MCP_BIN")
    binary, kind = resolve_server(explicit)
    print(f"🧪 Testing VantaDB MCP server: {binary} ({kind})")
    print("=" * 50)

    session = McpSession(binary, kind)
    passed = 0
    try:
        tests = [
            (
                "initialize",
                "initialize",
                {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "test-client", "version": "1.0.0"},
                },
            ),
            ("tools/list", "tools/list", None),
            ("resources/list", "resources/list", None),
            ("prompts/list", "prompts/list", None),
        ]
        for label, method, params in tests:
            print(f"🔍 Testing {label}...")
            resp = session.request(method, passed + 1, params)
            if "error" in resp:
                print(f"   ❌ {label} failed: server error: {resp['error']}")
                continue
            if "result" not in resp:
                print(f"   ❌ {label} failed: no result in response: {resp}")
                continue
            result = resp["result"]
            if label == "initialize":
                info = result.get("serverInfo", {})
                print(
                    f"   ✅ Server: {info.get('name', '?')} "
                    f"{info.get('version', '?')} (protocol {result.get('protocolVersion', '?')})"
                )
            elif label == "tools/list":
                tools = result.get("tools", [])
                print(f"   ✅ Found {len(tools)} tools")
                for tool in tools[:5]:
                    print(f"      - {tool['name']}")
                if len(tools) > 5:
                    print(f"      ... and {len(tools) - 5} more")
            elif label == "resources/list":
                resources = result.get("resources", [])
                print(f"   ✅ Found {len(resources)} resources")
            elif label == "prompts/list":
                prompts = result.get("prompts", [])
                print(f"   ✅ Found {len(prompts)} prompts")
            passed += 1
    finally:
        session.close()

    print("\n" + "=" * 50)
    print(f"📊 Results: {passed}/4 passed")
    return 0 if passed == 4 else 1


if __name__ == "__main__":
    sys.exit(main())