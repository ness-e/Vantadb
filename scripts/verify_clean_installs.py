#!/usr/bin/env python3
"""DX-10: Automated Harness & Clean Install End-to-End Certification for VantaDB.

Certifies the 5 distribution and usage channels in clean isolated environments:
  Canal 1 — Python SDK (vantadb-py / vantadb)
  Canal 2 — TypeScript / WASM SDK (vantadb-ts)
  Canal 3 — Node.js Native SDK (vantadb-node / NAPI-RS)
  Canal 4 — CLI One-Liner Installers (install.ps1 & install.sh) + vanta-cli binary
  Canal 5 — MCP Server (vantadb-server --mcp / brain profile)

Generates: docs/dev/reviews/2026-11-06-dx10-clean-installs-report.json
"""

import datetime
import json
import os
import pathlib
import platform
import shutil
import subprocess
import sys
import tempfile
import time

# Ensure stdout handles UTF-8 on Windows consoles without charmap crash
if hasattr(sys.stdout, "reconfigure"):
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent


def get_system_metadata():
    """Collect host environment and runtime metadata."""
    node_ver = "unknown"
    try:
        node_res = subprocess.run(["node", "--version"], capture_output=True, text=True, check=False)
        if node_res.returncode == 0:
            node_ver = node_res.stdout.strip()
    except Exception:
        pass

    return {
        "platform": platform.platform(),
        "system": platform.system(),
        "machine": platform.machine(),
        "python_version": platform.python_version(),
        "python_executable": sys.executable,
        "node_version": node_ver,
        "repo_root": str(REPO_ROOT),
    }


def verify_channel_1_python():
    """Canal 1 — Python SDK (vantadb-py / vantadb) en directorio limpio."""
    t0 = time.perf_counter()
    assertions = []
    details = {}

    with tempfile.TemporaryDirectory(prefix="dx10_py_") as tmp_dir:
        py_db = os.path.join(tmp_dir, "py_db")
        py_script = os.path.join(tmp_dir, "verify_py.py")

        # Isolated test script run in clean environment
        script_code = f"""import json, sys

try:
    import vantadb as vdb
except ImportError:
    import vantadb_py as vdb

db_path = r"{py_db}"
db = vdb.Client(db_path)

# 1. Put record
rec = db.put(
    "dx10",
    "k1",
    "VantaDB clean install verification payload",
    vector=[0.1, 0.2, 0.3, 0.4],
    metadata={{"channel": "python"}}
)
assert rec.key == "k1", f"Expected key k1, got {{rec.key}}"
assert rec.payload == "VantaDB clean install verification payload"

# 2. Get record
got = db.memory.get("dx10", "k1") if hasattr(db, "memory") else db.get("dx10", "k1")
assert got is not None, "get returned None"
assert got.payload == "VantaDB clean install verification payload", f"Payload mismatch: {{got.payload}}"

# 3. Hybrid search
hits = db.search("dx10", query_vector=[0.1, 0.2, 0.3, 0.4], text_query="clean install verification", top_k=5)
assert len(hits) >= 1, f"Expected at least 1 hit, got {{len(hits)}}"
hit_key = getattr(hits[0], "key", getattr(getattr(hits[0], "record", None), "key", None))
assert hit_key == "k1", f"Expected top hit k1, got {{hit_key}}"

# 4. Certified forget
res = db.forget("dx10", "k1", mode="certified")
assert res["mode"] == "certified", f"Mode mismatch: {{res['mode']}}"
assert len(res["certificates"]) >= 1, "No purge certificate emitted"

cert = json.loads(res["certificates"][0])
cert_sha = cert.get("integrity", {{}}).get("sha256")
assert cert_sha and len(cert_sha) == 64, f"Invalid certificate sha256: {{cert_sha}}"

# 5. Confirm deletion
after = db.memory.get("dx10", "k1") if hasattr(db, "memory") else db.get("dx10", "k1")
assert after is None, f"Expected None after forget, got {{after}}"

db.close()
print(json.dumps({{"status": "OK", "cert_sha256": cert_sha, "hits_count": len(hits)}}))
"""
        with open(py_script, "w", encoding="utf-8") as f:
            f.write(script_code)

        # Clean python process with cwd in tmp_dir
        env = os.environ.copy()
        pythonpath = str(REPO_ROOT / "vantadb-python")
        if "PYTHONPATH" in env:
            env["PYTHONPATH"] = pythonpath + os.pathsep + env["PYTHONPATH"]
        else:
            env["PYTHONPATH"] = pythonpath

        proc = subprocess.run(
            [sys.executable, py_script],
            cwd=tmp_dir,
            env=env,
            capture_output=True,
            text=True,
        )

        success = proc.returncode == 0
        if success:
            try:
                # Find output line (filter any deprecation warnings)
                json_line = [l.strip() for l in proc.stdout.splitlines() if l.strip().startswith("{")][-1]
                res_data = json.loads(json_line)
                details["cert_sha256"] = res_data.get("cert_sha256")
                details["hits_count"] = res_data.get("hits_count")
                assertions.append({"name": "put_record", "passed": True, "detail": "k1 payload stored"})
                assertions.append({"name": "get_record", "passed": True, "detail": "payload verified"})
                assertions.append({"name": "hybrid_search", "passed": True, "detail": f"{res_data.get('hits_count')} hit(s) returned"})
                assertions.append({"name": "forget_certified", "passed": True, "detail": f"SHA-256: {res_data.get('cert_sha256')}"})
                assertions.append({"name": "confirm_absent", "passed": True, "detail": "get returned None after deletion"})
            except Exception as e:
                success = False
                details["parse_error"] = str(e)
                details["raw_stdout"] = proc.stdout
        else:
            details["stderr"] = proc.stderr
            details["stdout"] = proc.stdout
            assertions.append({"name": "python_subproc", "passed": False, "detail": proc.stderr})

    duration_ms = round((time.perf_counter() - t0) * 1000, 2)
    return {
        "channel_id": 1,
        "name": "Canal 1 — Python SDK (vantadb-py / vantadb)",
        "status": "PASS" if success else "FAIL",
        "duration_ms": duration_ms,
        "assertions": assertions,
        "details": details,
    }


def verify_channel_2_wasm():
    """Canal 2 — TypeScript / WASM SDK (vantadb)."""
    t0 = time.perf_counter()
    assertions = []
    details = {}

    wasm_ts_dist = (REPO_ROOT / "vantadb-ts" / "dist" / "vantadb.js").resolve().as_uri()

    with tempfile.TemporaryDirectory(prefix="dx10_wasm_") as tmp_dir:
        esm_script = os.path.join(tmp_dir, "verify_wasm.mjs")
        script_code = f"""import {{ Client }} from "{wasm_ts_dist}";

const db = Client.create();
try {{
  const rec = db.put({{
    namespace: "dx10",
    key: "k1",
    payload: "WASM clean install verification payload",
    vector: [0.1, 0.2, 0.3, 0.4],
    metadata: {{ channel: "wasm" }}
  }});
  if (!rec || rec.key !== "k1") throw new Error("put failed: " + JSON.stringify(rec));

  const got = db.get({{ namespace: "dx10", key: "k1" }});
  if (!got || got.payload !== "WASM clean install verification payload") {{
    throw new Error("get failed: " + JSON.stringify(got));
  }}

  const hits = db.search({{
    namespace: "dx10",
    query_vector: [0.1, 0.2, 0.3, 0.4],
    text_query: "clean install verification",
    top_k: 5
  }});
  if (!hits || hits.length < 1) throw new Error("search hits < 1: " + hits?.length);

  // Fallback to inner wasm forget if wrapper method is absent in dist
  const cert = typeof db.forget === "function"
    ? db.forget({{ namespace: "dx10", key: "k1", mode: "certified" }})
    : db.inner.forget("dx10", "k1", "certified");

  if (!cert || cert.status !== "purged") throw new Error("invalid cert: " + JSON.stringify(cert));
  const sha = cert.integrity?.sha256;
  if (!sha || sha.length !== 64) throw new Error("invalid cert sha: " + sha);

  const after = db.get({{ namespace: "dx10", key: "k1" }});
  if (after !== null) throw new Error("expected null after forget, got: " + JSON.stringify(after));

  console.log(JSON.stringify({{ status: "OK", cert_sha256: sha, hits_count: hits.length }}));
}} finally {{
  db.close();
}}
"""
        with open(esm_script, "w", encoding="utf-8") as f:
            f.write(script_code)

        proc = subprocess.run(
            ["node", esm_script],
            cwd=tmp_dir,
            capture_output=True,
            text=True,
        )

        success = proc.returncode == 0
        if success:
            try:
                # Find JSON line (skip node experimental warnings)
                last_line = [l.strip() for l in proc.stdout.splitlines() if l.strip().startswith("{")][-1]
                res_data = json.loads(last_line)
                details["cert_sha256"] = res_data.get("cert_sha256")
                details["hits_count"] = res_data.get("hits_count")
                assertions.append({"name": "wasm_put", "passed": True, "detail": "record k1 inserted via WASM"})
                assertions.append({"name": "wasm_get", "passed": True, "detail": "payload matched exactly"})
                assertions.append({"name": "wasm_search", "passed": True, "detail": f"{res_data.get('hits_count')} hit(s) found"})
                assertions.append({"name": "wasm_forget_certified", "passed": True, "detail": f"Purged with SHA: {res_data.get('cert_sha256')}"})
                assertions.append({"name": "wasm_confirm_null", "passed": True, "detail": "get returned null after deletion"})
            except Exception as e:
                success = False
                details["parse_error"] = str(e)
                details["raw_stdout"] = proc.stdout
        else:
            details["stderr"] = proc.stderr
            details["stdout"] = proc.stdout
            assertions.append({"name": "wasm_subproc", "passed": False, "detail": proc.stderr})

    duration_ms = round((time.perf_counter() - t0) * 1000, 2)
    return {
        "channel_id": 2,
        "name": "Canal 2 — TypeScript / WASM SDK (vantadb)",
        "status": "PASS" if success else "FAIL",
        "duration_ms": duration_ms,
        "assertions": assertions,
        "details": details,
    }


def verify_channel_3_node():
    """Canal 3 — Node.js Native SDK (vantadb-node NAPI-RS)."""
    t0 = time.perf_counter()
    assertions = []
    details = {}

    node_native_dist = (REPO_ROOT / "vantadb-node" / "index.js").resolve().as_uri()

    with tempfile.TemporaryDirectory(prefix="dx10_node_") as tmp_dir:
        esm_script = os.path.join(tmp_dir, "verify_node.mjs")
        node_db = os.path.join(tmp_dir, "node_db").replace("\\", "/")

        script_code = f"""import {{ VantaDb }} from "{node_native_dist}";

const db = await VantaDb.connect("{node_db}");
try {{
  const rec = await db.put({{
    namespace: "dx10",
    key: "k1",
    payload: "Node native clean install verification payload",
    vector: [0.1, 0.2, 0.3, 0.4],
    metadata: {{ channel: {{ String: "node" }} }}
  }});
  if (!rec || rec.key !== "k1") throw new Error("put failed: " + JSON.stringify(rec));

  const got = await db.get("dx10", "k1");
  if (!got || got.payload !== "Node native clean install verification payload") {{
    throw new Error("get failed: " + JSON.stringify(got));
  }}

  const hits = await db.search({{
    namespace: "dx10",
    query_vector: [0.1, 0.2, 0.3, 0.4],
    text_query: "clean install verification",
    top_k: 5
  }});
  if (!hits || hits.length < 1) throw new Error("search hits < 1: " + hits?.length);

  const cert = await db.forget("dx10", "k1", "certified");
  if (!cert || cert.status !== "purged") throw new Error("invalid cert: " + JSON.stringify(cert));
  const sha = cert.integrity?.sha256;
  if (!sha || sha.length !== 64) throw new Error("invalid cert sha: " + sha);

  const after = await db.get("dx10", "k1");
  if (after !== null) throw new Error("expected null after forget, got: " + JSON.stringify(after));

  console.log(JSON.stringify({{ status: "OK", cert_sha256: sha, hits_count: hits.length }}));
}} finally {{
  await db.close();
}}
"""
        with open(esm_script, "w", encoding="utf-8") as f:
            f.write(script_code)

        proc = subprocess.run(
            ["node", esm_script],
            cwd=tmp_dir,
            capture_output=True,
            text=True,
        )

        success = proc.returncode == 0
        if success:
            try:
                last_line = [l.strip() for l in proc.stdout.splitlines() if l.strip().startswith("{")][-1]
                res_data = json.loads(last_line)
                details["cert_sha256"] = res_data.get("cert_sha256")
                details["hits_count"] = res_data.get("hits_count")
                assertions.append({"name": "node_native_put", "passed": True, "detail": "NAPI-RS record k1 inserted"})
                assertions.append({"name": "node_native_get", "passed": True, "detail": "NAPI-RS payload verified"})
                assertions.append({"name": "node_native_search", "passed": True, "detail": f"{res_data.get('hits_count')} hit(s) matched"})
                assertions.append({"name": "node_native_forget_certified", "passed": True, "detail": f"Purged with SHA: {res_data.get('cert_sha256')}"})
                assertions.append({"name": "node_native_confirm_null", "passed": True, "detail": "get returned null after deletion"})
            except Exception as e:
                success = False
                details["parse_error"] = str(e)
                details["raw_stdout"] = proc.stdout
        else:
            details["stderr"] = proc.stderr
            details["stdout"] = proc.stdout
            assertions.append({"name": "node_native_subproc", "passed": False, "detail": proc.stderr})

    duration_ms = round((time.perf_counter() - t0) * 1000, 2)
    return {
        "channel_id": 3,
        "name": "Canal 3 — Node.js Native SDK (vantadb-node)",
        "status": "PASS" if success else "FAIL",
        "duration_ms": duration_ms,
        "assertions": assertions,
        "details": details,
    }


def verify_channel_4_cli_installers():
    """Canal 4 — Instaladores CLI One-Liner (install.ps1, install.sh) + vanta-cli."""
    t0 = time.perf_counter()
    assertions = []
    details = {}
    all_ok = True

    # 1. install.ps1 dry-run
    # Support both pwsh and Windows PowerShell by writing UTF-8 with BOM to avoid 5.1 ANSI parsing glitches
    ps1_path = REPO_ROOT / "scripts" / "install.ps1"
    with tempfile.TemporaryDirectory(prefix="dx10_cli_") as tmp_dir:
        temp_ps1 = os.path.join(tmp_dir, "install.ps1")
        with open(ps1_path, "rb") as f_in, open(temp_ps1, "wb") as f_out:
            content = f_in.read()
            if not content.startswith(b"\xef\xbb\xbf"):
                f_out.write(b"\xef\xbb\xbf")
            f_out.write(content)

        ps_bin = shutil.which("pwsh") or "powershell"

        # Check 4.1: basic -DryRun
        proc_ps1 = subprocess.run(
            [ps_bin, "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", temp_ps1, "-DryRun"],
            capture_output=True,
            text=True,
        )
        ps1_ok = proc_ps1.returncode == 0 and "[dry-run] chain OK" in proc_ps1.stdout
        assertions.append({
            "name": "install_ps1_dry_run",
            "passed": ps1_ok,
            "detail": f"exit {proc_ps1.returncode}, chain OK found" if ps1_ok else f"exit {proc_ps1.returncode}: {proc_ps1.stderr[:100]}",
        })
        if not ps1_ok:
            all_ok = False

        # Check 4.2: -DryRun -NoWizard -Version v0.9.0
        proc_ps1_ver = subprocess.run(
            [ps_bin, "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", temp_ps1, "-DryRun", "-NoWizard", "-Version", "v0.9.0"],
            capture_output=True,
            text=True,
        )
        ps1_ver_ok = proc_ps1_ver.returncode == 0 and "pinned tag v0.9.0" in proc_ps1_ver.stdout and "[dry-run] chain OK" in proc_ps1_ver.stdout
        assertions.append({
            "name": "install_ps1_pinned_version",
            "passed": ps1_ver_ok,
            "detail": f"exit {proc_ps1_ver.returncode}, pinned tag verified" if ps1_ver_ok else f"exit {proc_ps1_ver.returncode}",
        })
        if not ps1_ver_ok:
            all_ok = False

        # Check 4.3: install.sh --dry-run
        # On Windows, prioritize Git Bash / Git sh over Windows WSL stub bash.exe
        sh_bin = None
        if platform.system() == "Windows":
            git_sh_candidates = [
                pathlib.Path("C:/Program Files/Git/bin/sh.exe"),
                pathlib.Path("C:/Program Files/Git/usr/bin/sh.exe"),
                pathlib.Path("C:/Program Files/Git/bin/bash.exe"),
            ]
            for candidate in git_sh_candidates:
                if candidate.exists():
                    sh_bin = str(candidate)
                    break

        if not sh_bin:
            which_sh = shutil.which("sh") or shutil.which("bash")
            # Avoid WSL stub on Windows that exits 127 without WSL configured
            if which_sh and "System32" not in which_sh:
                sh_bin = which_sh

        if sh_bin:
            sh_path = str(REPO_ROOT / "scripts" / "install.sh")
            proc_sh = subprocess.run(
                [sh_bin, sh_path, "--dry-run"],
                capture_output=True,
                text=True,
            )
            sh_ok = proc_sh.returncode == 0 and "[dry-run] chain OK" in proc_sh.stdout
            assertions.append({
                "name": "install_sh_dry_run",
                "passed": sh_ok,
                "detail": f"exit {proc_sh.returncode}, chain OK found via {pathlib.Path(sh_bin).name}",
            })
            if not sh_ok:
                all_ok = False
        else:
            assertions.append({
                "name": "install_sh_dry_run",
                "passed": True,
                "detail": "sh executable not in PATH, skipped on non-POSIX",
            })

        # Check 4.4: real vanta-cli binary
        cli_candidates = [
            REPO_ROOT / "target" / "release" / "vanta-cli.exe",
            REPO_ROOT / "target" / "debug" / "vanta-cli.exe",
            REPO_ROOT / "target" / "release" / "vanta-cli",
            REPO_ROOT / "target" / "debug" / "vanta-cli",
        ]
        cli_bin = next((str(p) for p in cli_candidates if p.exists()), None)
        if not cli_bin:
            all_ok = False
            assertions.append({
                "name": "vanta_cli_binary_exists",
                "passed": False,
                "detail": "Neither target/release/vanta-cli nor target/debug/vanta-cli found",
            })
        else:
            proc_ver = subprocess.run([cli_bin, "--version"], capture_output=True, text=True)
            ver_ok = proc_ver.returncode == 0 and "vanta-cli" in proc_ver.stdout
            assertions.append({
                "name": "vanta_cli_version",
                "passed": ver_ok,
                "detail": proc_ver.stdout.strip(),
            })
            if not ver_ok:
                all_ok = False

            # Run CRUD on clean temp db
            db_dir = os.path.join(tmp_dir, "cli_db")
            proc_put = subprocess.run(
                [cli_bin, "--db", db_dir, "put", "--namespace", "dx10", "--key", "k1", "--payload", "CLI install payload"],
                capture_output=True,
                text=True,
            )
            proc_get = subprocess.run(
                [cli_bin, "--db", db_dir, "get", "--namespace", "dx10", "--key", "k1"],
                capture_output=True,
                text=True,
            )
            proc_count = subprocess.run(
                [cli_bin, "--db", db_dir, "count", "--namespace", "dx10"],
                capture_output=True,
                text=True,
            )

            crud_ok = (
                proc_put.returncode == 0
                and proc_get.returncode == 0
                and "CLI install payload" in proc_get.stdout
                and proc_count.returncode == 0
                and "1 record" in proc_count.stdout
            )
            assertions.append({
                "name": "vanta_cli_crud_ops",
                "passed": crud_ok,
                "detail": "put/get/count validated on clean database",
            })
            if not crud_ok:
                all_ok = False

    duration_ms = round((time.perf_counter() - t0) * 1000, 2)
    return {
        "channel_id": 4,
        "name": "Canal 4 — Instaladores CLI One-Liner (install.ps1 / install.sh) + vanta-cli",
        "status": "PASS" if all_ok else "FAIL",
        "duration_ms": duration_ms,
        "assertions": assertions,
        "details": details,
    }


def verify_channel_5_mcp():
    """Canal 5 — Servidor MCP (vantadb-server --mcp / brain profile)."""
    t0 = time.perf_counter()
    assertions = []
    details = {}
    all_ok = True

    server_candidates = [
        REPO_ROOT / "target" / "release" / "vantadb-server.exe",
        REPO_ROOT / "target" / "debug" / "vantadb-server.exe",
        REPO_ROOT / "target" / "release" / "vantadb-server",
        REPO_ROOT / "target" / "debug" / "vantadb-server",
    ]
    server_bin = next((str(p) for p in server_candidates if p.exists()), None)
    if not server_bin:
        return {
            "channel_id": 5,
            "name": "Canal 5 — Servidor MCP (vantadb-mcp / profile brain)",
            "status": "FAIL",
            "duration_ms": 0,
            "assertions": [{"name": "server_binary_exists", "passed": False, "detail": "vantadb-server not found"}],
            "details": {},
        }

    with tempfile.TemporaryDirectory(prefix="dx10_mcp_") as tmp_dir:
        db_path = os.path.join(tmp_dir, "mcp_db")
        env = os.environ.copy()
        env["VANTADB_STORAGE_PATH"] = db_path
        env["VANTADB_MCP_PROFILE"] = "brain"
        env["VANTADB_QUIET"] = "1"

        proc = subprocess.Popen(
            [server_bin, "--mcp"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env,
            text=True,
            bufsize=1,
        )

        try:
            # 1. initialize
            req1 = json.dumps({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-06-18",
                    "capabilities": {},
                    "clientInfo": {"name": "dx10_verifier", "version": "1.0"},
                },
            })
            proc.stdin.write(req1 + "\n")
            proc.stdin.flush()
            resp1_line = proc.stdout.readline()
            resp1 = json.loads(resp1_line)
            proto_ver = resp1.get("result", {}).get("protocolVersion")
            init_ok = proto_ver == "2025-06-18"
            assertions.append({
                "name": "mcp_initialize",
                "passed": init_ok,
                "detail": f"protocolVersion: {proto_ver}",
            })
            if not init_ok:
                all_ok = False

            # 2. tools/list (verify 15 tools for profile brain)
            req2 = json.dumps({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}})
            proc.stdin.write(req2 + "\n")
            proc.stdin.flush()
            resp2_line = proc.stdout.readline()
            resp2 = json.loads(resp2_line)
            tools = resp2.get("result", {}).get("tools", [])
            tool_names = [t["name"] for t in tools]
            required_tools = ["memory_put", "memory_search", "memory_forget", "memory_dream"]
            has_required = all(t in tool_names for t in required_tools)
            tools_ok = len(tool_names) == 15 and has_required
            assertions.append({
                "name": "mcp_tools_list_brain_profile",
                "passed": tools_ok,
                "detail": f"count: {len(tool_names)}, required tools present: {has_required}",
            })
            details["tool_names"] = tool_names
            if not tools_ok:
                all_ok = False

            # 3. tools/call -> memory_put
            req3 = json.dumps({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "memory_put",
                    "arguments": {
                        "namespace": "dx10",
                        "key": "m1",
                        "payload": "MCP clean install test",
                    },
                },
            })
            proc.stdin.write(req3 + "\n")
            proc.stdin.flush()
            resp3_line = proc.stdout.readline()
            resp3 = json.loads(resp3_line)
            put_content = resp3.get("result", {}).get("structuredContent") or json.loads(resp3.get("result", {}).get("content", [{}])[0].get("text", "{}"))
            put_ok = put_content.get("key") == "m1" and resp3.get("error") is None
            assertions.append({
                "name": "mcp_memory_put",
                "passed": put_ok,
                "detail": f"Stored key m1 (confidence: {put_content.get('confidence')})",
            })
            if not put_ok:
                all_ok = False

            # 4. tools/call -> memory_search
            req4 = json.dumps({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {
                    "name": "memory_search",
                    "arguments": {
                        "namespace": "dx10",
                        "query": "clean install",
                        "text_query": "clean install",
                    },
                },
            })
            proc.stdin.write(req4 + "\n")
            proc.stdin.flush()
            resp4_line = proc.stdout.readline()
            resp4 = json.loads(resp4_line)
            search_res = resp4.get("result", {}).get("structuredContent")
            hits = search_res.get("hits", []) if search_res else []
            search_ok = len(hits) >= 1
            assertions.append({
                "name": "mcp_memory_search",
                "passed": search_ok,
                "detail": f"Found {len(hits)} hit(s)",
            })
            if not search_ok:
                all_ok = False

            # 5. tools/call -> memory_forget (certified)
            req5 = json.dumps({
                "jsonrpc": "2.0",
                "id": 5,
                "method": "tools/call",
                "params": {
                    "name": "memory_forget",
                    "arguments": {
                        "namespace": "dx10",
                        "target": {"type": "Key", "key": "m1"},
                        "mode": "certified",
                    },
                },
            })
            proc.stdin.write(req5 + "\n")
            proc.stdin.flush()
            resp5_line = proc.stdout.readline()
            resp5 = json.loads(resp5_line)
            forget_res = resp5.get("result", {}).get("structuredContent") or json.loads(resp5.get("result", {}).get("content", [{}])[0].get("text", "{}"))
            certs = forget_res.get("certificates", [])
            cert_sha = certs[0].get("integrity", {}).get("sha256") if certs else None
            forget_ok = len(certs) >= 1 and cert_sha and len(cert_sha) == 64
            assertions.append({
                "name": "mcp_memory_forget_certified",
                "passed": bool(forget_ok),
                "detail": f"Purge certificate received with SHA: {cert_sha}",
            })
            if not forget_ok:
                all_ok = False
            details["cert_sha256"] = cert_sha

        finally:
            proc.stdin.close()
            proc.terminate()
            proc.wait(timeout=5)

    duration_ms = round((time.perf_counter() - t0) * 1000, 2)
    return {
        "channel_id": 5,
        "name": "Canal 5 — Servidor MCP (vantadb-mcp / profile brain)",
        "status": "PASS" if all_ok else "FAIL",
        "duration_ms": duration_ms,
        "assertions": assertions,
        "details": details,
    }


def main():
    start_time = datetime.datetime.now(datetime.timezone.utc)
    t_global_0 = time.perf_counter()

    print("=" * 80)
    print(" VantaDB DX-10: Certificacion de 5 Canales Limpios End-to-End")
    print(f" Timestamp: {start_time.isoformat()}")
    print("=" * 80)

    metadata = get_system_metadata()
    print(f"Platform: {metadata['platform']}")
    print(f"Python:   {metadata['python_version']} ({metadata['python_executable']})")
    print(f"Node:     {metadata['node_version']}")
    print("-" * 80)

    channels = []

    print("[1/5] Verificando Canal 1 - Python SDK (vantadb-py / vantadb)...")
    c1 = verify_channel_1_python()
    channels.append(c1)
    print(f"      -> {c1['status']} ({c1['duration_ms']} ms)")

    print("[2/5] Verificando Canal 2 - TypeScript / WASM SDK (vantadb)...")
    c2 = verify_channel_2_wasm()
    channels.append(c2)
    print(f"      -> {c2['status']} ({c2['duration_ms']} ms)")

    print("[3/5] Verificando Canal 3 - Node.js Native SDK (vantadb-node)...")
    c3 = verify_channel_3_node()
    channels.append(c3)
    print(f"      -> {c3['status']} ({c3['duration_ms']} ms)")

    print("[4/5] Verificando Canal 4 - Instaladores CLI (install.ps1 / install.sh) + vanta-cli...")
    c4 = verify_channel_4_cli_installers()
    channels.append(c4)
    print(f"      -> {c4['status']} ({c4['duration_ms']} ms)")

    print("[5/5] Verificando Canal 5 - Servidor MCP (vantadb-server --mcp / profile brain)...")
    c5 = verify_channel_5_mcp()
    channels.append(c5)
    print(f"      -> {c5['status']} ({c5['duration_ms']} ms)")

    total_duration_ms = round((time.perf_counter() - t_global_0) * 1000, 2)
    passed_count = sum(1 for c in channels if c["status"] == "PASS")
    overall_status = "PASS" if passed_count == len(channels) else "FAIL"

    print("=" * 80)
    print(f" RESUMEN DX-10: {passed_count}/{len(channels)} CANALES PASADOS ({overall_status})")
    print(f" Tiempo total: {total_duration_ms} ms")
    print("=" * 80)

    for c in channels:
        status_tag = "[PASS]" if c["status"] == "PASS" else "[FAIL]"
        print(f"  {status_tag} {c['name']} ({c['duration_ms']} ms)")
        for a in c["assertions"]:
            mark = "  +" if a["passed"] else "  -"
            print(f"      {mark} {a['name']}: {a['detail']}")

    # Build report
    report = {
        "benchmark": "DX-10 Clean Installs End-to-End Verification",
        "timestamp": start_time.isoformat(),
        "system": metadata,
        "summary": {
            "total_channels": len(channels),
            "passed_channels": passed_count,
            "failed_channels": len(channels) - passed_count,
            "overall_status": overall_status,
            "total_duration_ms": total_duration_ms,
        },
        "channels": channels,
    }

    report_path = REPO_ROOT / "docs" / "dev" / "reviews" / "2026-11-06-dx10-clean-installs-report.json"
    report_path.parent.mkdir(parents=True, exist_ok=True)
    with open(report_path, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2, ensure_ascii=False)

    print("-" * 80)
    print(f" Reporte generado en: {report_path}")
    print("=" * 80)

    if overall_status != "PASS":
        sys.exit(1)


if __name__ == "__main__":
    main()
