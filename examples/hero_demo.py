"""Hero demo - freeze one real local run into vhs/hero-run.json.

Five stages: CLI doctor, MCP handshake, SDK memory capture, supersession,
CLI stats. Writes frames only after every self-check passes; the renderer
reads the frozen JSON and never re-runs the backend.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

from migrate_from_mem0 import _LocalE5Encoder

ROOT = Path(__file__).resolve().parents[1]
NS = "agent/hero"
_ANSI = re.compile(r"\x1b\[[0-9;]*m")
TMP_MARK = "<temporary demo database>"


def _cli() -> str:
    cand = os.environ.get("VANTADB_CLI_BIN") or str(
        ROOT
        / "target"
        / "release"
        / ("vanta-cli.exe" if os.name == "nt" else "vanta-cli")
    )
    if not Path(cand).is_file():
        raise RuntimeError(f"CLI binary not found: {cand}")
    rc, out, _ = _run([cand, "--version"], dict(os.environ))
    if rc != 0 or "0.9.0" not in out:
        raise RuntimeError(f"CLI binary is not version 0.9.0: {out.strip()!r}")
    return cand


def _env(tmp: str) -> dict:
    e = dict(os.environ)
    e["PATH"] = str(ROOT / "target" / "release") + os.pathsep + e["PATH"]
    e["VANTADB_STORAGE_PATH"] = tmp
    e["VANTADB_MCP_PROFILE"] = "brain"
    e["RUST_LOG"] = "error"
    return e


def _run(
    argv: list[str],
    env: dict,
    *,
    stdin: str | None = None,
    timeout: int = 30,
) -> tuple[int, str, str]:
    p = subprocess.run(
        argv,
        env=env,
        input=stdin,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
        timeout=timeout,
    )
    return p.returncode, p.stdout, p.stderr


def _display(raw: str, tmp: str) -> str:
    return _ANSI.sub("", raw).replace(tmp, TMP_MARK)


_INIT = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "vantadb-demo", "version": "1"},
    },
}
_SCENE_SUMMARY = "Demo formatting preference"
_SCENE_CONTENT = (
    "Current user preference: 2-space indentation with tabs enabled. "
    "Source: application-curated agent/hero/pref-indent-2; replaces pref-indent. "
    "No LLM extraction or automatic consolidation was performed."
)


def _mcp_roundtrip(cli: str, env: dict, tmp: str, calls: list[dict]) -> dict[int, dict]:
    wire = [_INIT, {"jsonrpc": "2.0", "method": "notifications/initialized"}] + calls
    payload = "\n".join(json.dumps(r, separators=(",", ":")) for r in wire) + "\n"
    rc, out, err = _run([cli, "--db", tmp, "server", "--mcp"], env, stdin=payload)
    if rc != 0:
        raise RuntimeError(f"MCP child exited {rc}: {_display(err or out, tmp)[:400]}")
    answers: dict[int, dict] = {}
    for line in out.splitlines():
        try:
            m = json.loads(line)
        except ValueError:
            continue
        if isinstance(m.get("id"), int):
            answers[m["id"]] = m
    return answers


def _scene_projection(cli: str, env: dict, tmp: str, ev: Path | None) -> dict:
    summary = _SCENE_SUMMARY
    content = _SCENE_CONTENT
    write_call = {
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "vanta_scene",
            "arguments": {
                "action": "write",
                "session_key": "hero-session",
                "scene_name": "formatting",
                "summary": summary,
                "content": content,
            },
        },
    }
    try:
        w = _mcp_roundtrip(cli, env, tmp, [write_call])[3]
    except (KeyError, RuntimeError) as e:
        raise RuntimeError(f"L2 write roundtrip failed: {e}") from e
    if w.get("error") or w.get("result", {}).get("isError"):
        raise RuntimeError(f"L2 write returned JSON-RPC error: {json.dumps(w)[:400]}")
    if ev:
        (ev / "L2-write.log").write_text(json.dumps(w, indent=2), encoding="utf-8")

    read_call = {
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "vanta_scene",
            "arguments": {
                "action": "read",
                "session_key": "hero-session",
                "scene_name": "formatting",
            },
        },
    }
    try:
        r = _mcp_roundtrip(cli, env, tmp, [read_call])[3]
    except (KeyError, RuntimeError) as e:
        raise RuntimeError(f"L2 read roundtrip failed: {e}") from e
    if r.get("error") or r.get("result", {}).get("isError"):
        raise RuntimeError(f"L2 read returned JSON-RPC error: {json.dumps(r)[:400]}")
    if ev:
        (ev / "L2-read.log").write_text(json.dumps(r, indent=2), encoding="utf-8")

    texts = [
        c.get("text")
        for c in r.get("result", {}).get("content", [])
        if c.get("type") == "text"
    ]
    if not texts:
        raise RuntimeError("L2 read returned no text content")
    scene = json.loads(texts[0]).get("scene")
    if not isinstance(scene, dict):
        raise TypeError(f"unexpected scene payload shape: {texts[0][:200]}")
    if (
        scene.get("scene_name") != "formatting"
        or scene.get("meta", {}).get("summary") != summary
        or scene.get("content") != content
        or scene.get("deleted") is not False
    ):
        raise RuntimeError("L2 scene round-trip mismatch — not substituting query")
    return scene


def _memory_frames(db, enc) -> tuple[dict, dict]:
    cap = db.memory_capture(
        "hero-session",
        [
            {
                "role": "user",
                "content": "Please remember that this demo user prefers "
                "four-space indentation and no tabs in source files.",
            },
            {
                "role": "assistant",
                "content": "I will keep that formatting preference available "
                "for future work in this demonstration.",
            },
        ],
    )
    assert cap["recorded_count"] > 0
    vec = enc("User prefers four-space indentation and no tabs")
    assert len(vec) == 384
    db.put(
        NS,
        "pref-indent",
        "User prefers four-space indentation and no tabs",
        vector=vec,
        metadata={"entity": "demo-user", "scope": "formatting"},
    )
    db.put(
        NS,
        "standup",
        "Daily standup starts at nine in the morning.",
        vector=enc("Daily standup starts at nine in the morning."),
        metadata={"scope": "planning"},
    )
    curated = len(db.memory.list(NS).records)
    frame_capture = {
        "title": "Memory capture",
        "command": "db.memory_capture(...) + db.put(...)  # local E5 encoder",
        "output": (
            f"captured L0 messages={cap['recorded_count']} "
            f"filtered={cap.get('filtered_messages')} cursor_ms={cap.get('cursor_ms')}\n"
            f"application-curated records={curated}  vector dims: {len(vec)}\n"
            "Application-curated memories; automatic L2 consolidation is not demonstrated."
        ),
    }

    qv = enc.embed_query("indentation preference")
    base = {
        "query_vector": qv,
        "text_query": "indentation",
        "filters": {"scope": "formatting"},
        "top_k": 5,
    }
    initial = db.search(NS, **base)
    assert any(h.key == "pref-indent" for h in initial)

    db.put(
        NS,
        "pref-indent-2",
        "User prefers 2-space indentation with tabs",
        vector=enc("User prefers 2-space indentation with tabs"),
        metadata={"entity": "demo-user", "scope": "formatting"},
    )
    db.memory.supersede(NS, "pref-indent", "pref-indent-2")

    cur = {h.key for h in db.search(NS, **{**base, "exclude_superseded": True})}
    hist = {h.key for h in db.search(NS, **{**base, "exclude_superseded": False})}
    assert "standup" not in cur and "standup" not in hist
    assert cur == {"pref-indent-2"} and hist == {"pref-indent", "pref-indent-2"}

    from vantadb import memory_node_id

    old = db.memory.get(NS, "pref-indent")
    assert old.superseded_by == "pref-indent-2"
    old_id = memory_node_id(NS, "pref-indent")
    new_id = memory_node_id(NS, "pref-indent-2")
    adj = {}
    for key, nid, partner in (
        ("pref-indent", old_id, new_id),
        ("pref-indent-2", new_id, old_id),
    ):
        node = db.graph.get_node(nid)
        assert node is not None
        labels = [e[1] for e in node["edges"] if e[0] == partner]
        adj[key] = sorted(labels)
    assert "superseded_by" in adj["pref-indent"]
    assert "supersedes_edge" in adj["pref-indent-2"]

    frame_supersede = {
        "title": "Hybrid search & supersession",
        "command": "db.search(...) + db.memory.supersede(...)",
        "output": (
            f"initial hybrid RRF score={initial[0].score:.4f} (native score, not a probability)\n"
            f"current={sorted(cur)}  history={sorted(hist)}\n"
            f"adjacency labels: {json.dumps(adj, sort_keys=True)}"
        ),
    }
    return frame_capture, frame_supersede


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--output", default=str(ROOT / "vhs" / "hero-run.json"))
    ap.add_argument("--evidence-dir", default=None)
    ap.add_argument(
        "--print-frames",
        action="store_true",
        help="print all frames to stdout after every check passes",
    )
    a = ap.parse_args()
    ev = Path(a.evidence_dir) if a.evidence_dir else None
    if ev:
        ev.mkdir(parents=True, exist_ok=True)

    try:
        cli = _cli()
    except RuntimeError as e:
        print(e, file=sys.stderr)
        return 1
    _, cli_version_out, _ = _run([cli, "--version"], dict(os.environ))
    frames: list[dict] = []
    enc = _LocalE5Encoder()
    info: dict = {}
    with tempfile.TemporaryDirectory(prefix="vanta-hero-") as tmp:
        env = _env(tmp)

        rc, out, err = _run([cli, "--db", tmp, "config", "doctor"], env)
        if ev:
            (ev / "s1-doctor.log").write_text(out + err, encoding="utf-8")
        if rc != 0:
            print(_display(err or out, tmp), file=sys.stderr)
            return 1
        frames.append(
            {
                "title": "Health check",
                "command": f"vanta-cli --db {TMP_MARK} config doctor",
                "output": _display(out, tmp).strip()
                + "\nThis run uses a separate local E5 ONNX encoder for SDK records.",
            }
        )

        reqs = [
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "vantadb-demo", "version": "1"},
                },
            },
            {"jsonrpc": "2.0", "method": "notifications/initialized"},
            {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
        ]
        payload = "\n".join(json.dumps(r, separators=(",", ":")) for r in reqs) + "\n"
        rc, out, err = _run([cli, "--db", tmp, "server", "--mcp"], env, stdin=payload)
        if ev:
            (ev / "s2-mcp.log").write_text(out + err, encoding="utf-8")
        if rc != 0:
            print(_display(err or out, tmp), file=sys.stderr)
            return 1
        init = res = None
        for line in out.splitlines():
            try:
                m = json.loads(line)
            except ValueError:
                continue
            if m.get("id") == 1:
                init = m
            elif m.get("id") == 2:
                res = m
        info = (init or {}).get("result", {}).get("serverInfo", {})
        if info.get("version") != "0.9.0":
            print(f"MCP server version mismatch: {info!r}", file=sys.stderr)
            return 1
        tools = (res or {}).get("result", {}).get("tools", [])
        names = [t.get("name") for t in tools]
        if not tools or len(set(names)) != len(names):
            print("MCP tools list invalid", file=sys.stderr)
            return 1
        frames.append(
            {
                "title": "MCP handshake",
                "command": f"vanta-cli --db {TMP_MARK} server --mcp  (JSON-RPC over stdio)",
                "output": (
                    f"serverInfo: {json.dumps(info)}\n"
                    f"protocolVersion: {init['result'].get('protocolVersion')}\n"
                    f"tools: {len(tools)}\n" + "\n".join(str(n) for n in names)
                ),
            }
        )

        from vantadb import Client

        db = Client(tmp)
        try:
            fc, fs = _memory_frames(db, enc)
        except AssertionError as e:
            print(f"memory stage invariant failed: {e}", file=sys.stderr)
            return 1
        finally:
            db.close()
        frames += [fc, fs]

        try:
            scene = _scene_projection(cli, env, tmp, ev)
        except RuntimeError as e:
            print(e, file=sys.stderr)
            return 1

        rc, out, err = _run([cli, "--db", tmp, "--json", "stats"], env)
        if ev:
            (ev / "s5-stats.log").write_text(out + err, encoding="utf-8")
        if rc != 0:
            print(_display(err or out, tmp), file=sys.stderr)
            return 1
        try:
            stats = json.loads(out)
        except ValueError:
            print("--json stats did not return valid JSON", file=sys.stderr)
            return 1
        namespaces = stats.get("namespaces", [])
        if "scene/hero-session" not in namespaces:
            print(
                f"scene/hero-session missing from stats namespaces: {namespaces!r}",
                file=sys.stderr,
            )
            return 1
        subset = {
            k: stats.get(k)
            for k in ("total_records", "total_vector_nodes", "node_count", "namespaces")
            if k in stats
        }
        evidence = {"stats": stats, "l2_scene": scene}
        (Path(a.output).parent / "hero-evidence.json").write_text(
            json.dumps(evidence, indent=2, allow_nan=False) + "\n", encoding="utf-8"
        )
        frames.append(
            {
                "title": "L2 scene & engine inspection",
                "command": (
                    f"vanta-cli --db {TMP_MARK} server --mcp  (vanta_scene write/read)\n"
                    f"vanta-cli --db {TMP_MARK} --json stats"
                ),
                "output": (
                    f"L2 scene read-back verified: scene_name={scene['scene_name']} "
                    f"summary={scene['meta']['summary']!r}\n"
                    "Application-authored L2 projection; not automatic consolidation.\n"
                    f"stats subset: {json.dumps(subset, sort_keys=True)}\n"
                    "CLI engine counters; not whole-Python-worker RSS.\n"
                    "physical_rss maps core storage estimates, not process RAM."
                ),
            }
        )

    doc = {
        "schema_version": 1,
        "cli_version": cli_version_out.strip(),
        "mcp_version": info.get("version", ""),
        "frames": frames,
        "disclosure": "Recorded presentation of a verified local run — edited pacing",
    }
    assert len(frames) == 5 and all(
        f["title"] and f["command"] and f["output"] for f in frames
    )
    out_path = Path(a.output)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(
        json.dumps(doc, indent=2, allow_nan=False) + "\n", encoding="utf-8"
    )
    if a.print_frames:
        for i, f in enumerate(frames, 1):
            print(f"--- Frame {i}: {f['title']} ---")
            print(f"$ {f['command']}")
            print(f["output"])
    print(f"wrote {out_path} ({len(frames)} frames)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
