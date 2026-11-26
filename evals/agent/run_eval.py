#!/usr/bin/env python3
"""
evals/agent/run_eval.py — FIND-151 agent-eval (Giskard OSS reservado, judge determinista).

10 casos dorados: disciplina de digest (vanta-research: <=500 palabras + URLs que
resuelven) y veredictos con evidencia (vanta-review: file:line o URL por claim).

Determinista y offline-capable: 0 tokens de LLM-judge. El judge Giskard
(`giskard.agents` ChatWorkflow) queda reservado a `/audit full` futuro (Wave 1).

Uso (contrato = gate L9 de /audit full; regresion = NO-GO):
  python evals/agent/run_eval.py                 # online: verifica URLs (timeout 10s)
  python evals/agent/run_eval.py --offline       # CI sin red: solo forma, reporta modo
  python evals/agent/run_eval.py --cases evals/agent/golden_cases.json
"""
from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys
import urllib.request

try:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except Exception:
    pass

HERE = pathlib.Path(__file__).parent
WORD_LIMIT = 500
URL_TIMEOUT = 10
EVIDENCE_RE = re.compile(r"^(\S+:\d+|https?://\S+)$")
VALID_VERDICTS = {"PASS", "CHANGES-REQUIRED", "FAIL", "NEEDS-FIX"}

# ponytail: urllib stdlib en vez de requests/httpx (rung 3: stdlib antes que dep).


def words(text: str) -> int:
    return len(text.split())


def url_resolves(url: str) -> bool:
    req = urllib.request.Request(url, method="HEAD", headers={"User-Agent": "vantadb-eval/1.0"})
    try:
        with urllib.request.urlopen(req, timeout=URL_TIMEOUT) as r:
            return r.status < 400
    except Exception:
        try:  # HEAD bloqueado -> GET liviano
            with urllib.request.urlopen(url, timeout=URL_TIMEOUT) as r:
                return r.status < 400
        except Exception:
            return False


def check_digest(case: dict, offline: bool) -> list[str]:
    errs, inp = [], case["input"]
    n = words(inp["text"])
    if n > WORD_LIMIT:
        errs.append(f"digest {n} palabras > {WORD_LIMIT}")
    if not inp.get("urls"):
        errs.append("digest sin URLs")
    for u in inp.get("urls", []):
        if not u.startswith("https://"):
            errs.append(f"URL no canónica: {u}")
        elif not offline and not url_resolves(u):
            errs.append(f"URL no resuelve: {u}")
    return errs


def check_verdict(case: dict) -> list[str]:
    errs, inp = [], case["input"]
    if inp.get("verdict") not in VALID_VERDICTS:
        errs.append(f"veredicto inválido: {inp.get('verdict')}")
    ev = inp.get("evidence") or []
    if not ev:
        errs.append("veredicto sin evidencia")
    for e in ev:
        if not EVIDENCE_RE.match(e):
            errs.append(f"evidencia sin file:line ni URL: {e}")
    if not inp.get("claim"):
        errs.append("claim vacío")
    return errs


def run(cases_path: pathlib.Path, offline: bool) -> int:
    suite = json.loads(cases_path.read_text(encoding="utf-8"))
    n_ok, rows = 0, []
    for c in suite["cases"]:
        errs = check_digest(c, offline) if c["kind"] == "digest" else check_verdict(c)
        ok = (not errs) == c["expect_pass"]
        n_ok += ok
        rows.append((c["id"], "PASS" if not errs else "FAIL", "; ".join(errs)))
    # Sondas negativas: el checker debe RECHAZAR violaciones (anti-pase vacuo).
    probes = [
        ("overlong", {"text": "x " * 600, "urls": ["https://example.com"]}, True),
        ("no-evidence", {"verdict": "PASS", "claim": "c", "evidence": []}, False),
    ]
    probe_ok = True
    for name, inp, is_digest in probes:
        fake = {"input": inp}
        errs = check_digest(fake, True) if is_digest else check_verdict(fake)
        if not errs:
            probe_ok = False
            rows.append((f"PROBE-{name}", "VACUO", "checker aceptó violación"))
    total = len(suite["cases"])
    print(f"{'caso':12} {'res':6} detalle")
    for i, r, d in rows:
        print(f"{i:12} {r:6} {d}")
    print(f"\neval: {n_ok}/{total} dorados + sondas {'OK' if probe_ok else 'FALLO'}"
          f" (modo={'offline' if offline else 'online'})")
    return 0 if (n_ok == total and probe_ok) else 1


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--cases", default=str(HERE / "golden_cases.json"))
    ap.add_argument("--offline", action="store_true")
    a = ap.parse_args()
    return run(pathlib.Path(a.cases), a.offline)


if __name__ == "__main__":
    raise SystemExit(main())
