#!/usr/bin/env python3
"""GOV-A4 — Doc snippet harness.

Extracts ```python blocks from docs/tutorials/*.md + docs/QUICKSTART.md and
executes each against a temp VantaDB (vantadb-py). Reports PASS/FAIL/SKIP per
snippet; exit code 1 if any FAIL.

NOTE: on the initial run FAILs are the EXPECTED outcome (known doc breakage,
e.g. graph_bfs("doc1","doc3") with wrong signature — fixed later in GOV-B3).

Skip rules:
- block contains `# vanta-skip`  -> SKIP
- third-party import not installed in this venv -> SKIP (missing dep)
- comment-only block -> SKIP

Usage: python dev-tools/validate_doc_snippets.py [path-substring-filter]
"""
import importlib.util
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = sorted((ROOT / "docs" / "tutorials").glob("*.md")) + [ROOT / "docs" / "QUICKSTART.md"]
TIMEOUT = 30  # seconds per snippet

# Local modules provided by the repo's venv, never "missing deps".
LOCAL_MODULES = {"vantadb", "vantadb_py"}

HEADER = """\
import os as _os, tempfile as _tf
_d = _tf.mkdtemp(prefix="vanta_snip_")
from vantadb import VantaDB as _VantaDB
db = _VantaDB(_os.path.join(_d, "db"))
"""

IMPORT_RE = re.compile(r"^\s*(?:import|from)\s+([A-Za-z_][\w.]*)")


def extract_blocks(md_path: Path):
    """Yield (start_line, code) for every ```python fenced block."""
    lines = md_path.read_text(encoding="utf-8").splitlines()
    blocks, i = [], 0
    while i < len(lines):
        if lines[i].strip() == "```python":
            start = i + 1
            code = []
            i += 1
            while i < len(lines) and lines[i].strip() != "```":
                code.append(lines[i])
                i += 1
            blocks.append((start, "\n".join(code)))
        i += 1
    return blocks


def missing_dep(code: str):
    """First top-level import whose module is not installed, or None."""
    for line in code.splitlines():
        m = IMPORT_RE.match(line)
        if m:
            mod = m.group(1).split(".")[0]
            if mod in LOCAL_MODULES:
                continue
            if importlib.util.find_spec(mod) is None:
                return mod
    return None


def has_code(code: str):
    return any(l.strip() and not l.strip().startswith("#") for l in code.splitlines())


def run_snippet(code: str):
    """Run one snippet; returns None on success or an error string."""
    with tempfile.TemporaryDirectory(prefix="vanta_snip_cwd_") as cwd:
        script = Path(cwd) / "snippet.py"
        script.write_text(HEADER + "\n" + code + "\n", encoding="utf-8")
        try:
            proc = subprocess.run(
                [sys.executable, str(script)],
                capture_output=True,
                text=True,
                timeout=TIMEOUT,
                cwd=cwd,  # relative paths from snippets land in tmp
            )
        except subprocess.TimeoutExpired:
            return f"TIMEOUT after {TIMEOUT}s"
        if proc.returncode != 0:
            tail = [l for l in proc.stderr.strip().splitlines() if l][-5:]
            return "\n".join(tail) or f"exit {proc.returncode}"
    return None


def main():
    flt = sys.argv[1] if len(sys.argv) > 1 else ""
    results = []  # (status, source, error)
    for md in DOCS:
        for start, code in extract_blocks(md):
            src = f"{md.relative_to(ROOT)}:{start}"
            if flt and flt not in str(src):
                continue
            if "# vanta-skip" in code:
                results.append(("SKIP", src, "directive # vanta-skip"))
            elif not has_code(code):
                results.append(("SKIP", src, "comment-only block"))
            elif dep := missing_dep(code):
                results.append(("SKIP", src, f"missing dependency: {dep}"))
            else:
                err = run_snippet(code)
                results.append(("FAIL" if err else "PASS", src, err))
    counts = {"PASS": 0, "FAIL": 0, "SKIP": 0}
    print(f"\n{'STATUS':<6} {'SOURCE':<58} DETAIL")
    print("-" * 100)
    for status, src, detail in results:
        counts[status] += 1
        mark = "" if status == "PASS" else f"  {detail.splitlines()[0] if detail else ''}"
        print(f"{status:<6} {src:<58}{mark}")
    print("-" * 100)
    for status, src, detail in results:
        if status == "FAIL":
            print(f"\n[FAIL] {src}\n{detail}")
    print(
        f"\nSummary: {counts['PASS']} PASS, {counts['FAIL']} FAIL, {counts['SKIP']} SKIP"
    )
    if counts["FAIL"]:
        print("NOTE: FAILs on the initial run are EXPECTED — doc fixes are GOV-B3.")
    return 1 if counts["FAIL"] else 0


if __name__ == "__main__":
    sys.exit(main())
