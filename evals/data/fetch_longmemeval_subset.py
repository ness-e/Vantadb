#!/usr/bin/env python3
"""Build the committed LongMemEval-S sub-sample (VER-08).

Downloads (or reuses a cached copy of) `longmemeval_s_cleaned.json` from the
pinned Hugging Face revision, verifies its sha256, then extracts a small,
deterministic sub-sample and writes it under `evals/data/`.

Provenance (verified 2026-09-29, see evals/data/README.md):
  dataset : xiaowu0162/longmemeval-cleaned  (replaces the original LongMemEval)
  split   : longmemeval_s_cleaned  ("LongMemEval_S", ~115K tokens / ~40 sessions)
  license : MIT  (HF metadata `license:mit`, tag `license:mit`)
  revision: 98d7416c24c778c2fee6e6f3006e7a073259d48f
  file    : sha256 d6f21ea9d60a0d56f34a05b609c79c88a451d2ae03597821ea3d5a9678c3a442
            (LFS oid of longmemeval_s_cleaned.json, 277,383,467 bytes)

Selection rule (deterministic, dataset order):
  1. Walk questions in file order.
  2. Keep the first `--nonabs` questions whose id does NOT end with `_abs`.
  3. Keep the first `--abs` questions whose id DOES end with `_abs`.
  4. Write non-abstention questions first, then abstention questions.

The output file is a JSON array of UNMODIFIED LongMemEval question objects, so
any LongMemEval loader can read it. A `.meta.json` sidecar records the
provenance + selection so the artifact is self-describing.

Commands:
  python evals/data/fetch_longmemeval_subset.py                      # defaults: 4 non-abs + 1 abs
  python evals/data/fetch_longmemeval_subset.py --nonabs 95 --abs 5  # larger local slice (not committed by default)
  python evals/data/fetch_longmemeval_subset.py --check              # verify cached file sha256 only
"""
from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import shutil
import sys
import urllib.request

try:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except Exception:
    pass

HF_URL = (
    "https://huggingface.co/datasets/xiaowu0162/longmemeval-cleaned/"
    "resolve/98d7416c24c778c2fee6e6f3006e7a073259d48f/longmemeval_s_cleaned.json"
)
REVISION = "98d7416c24c778c2fee6e6f3006e7a073259d48f"
SOURCE_SHA256 = "d6f21ea9d60a0d56f34a05b609c79c88a451d2ae03597821ea3d5a9678c3a442"
SOURCE_BYTES = 277_383_467
LICENSE = "MIT"
DEFAULT_CACHE = pathlib.Path("datasets/longmemeval/longmemeval_s_cleaned.json")
DEFAULT_OUTPUT = pathlib.Path("evals/data/longmemeval_s_subset.json")


def sha256_file(path: pathlib.Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def ensure_cache(cache: pathlib.Path, expect_sha: str) -> None:
    """Download the source file if absent; always verify sha256."""
    if not cache.exists():
        cache.parent.mkdir(parents=True, exist_ok=True)
        print(f"[fetch] downloading {HF_URL}\n[fetch] -> {cache} ({SOURCE_BYTES/1e6:.1f} MB)", file=sys.stderr)
        req = urllib.request.Request(HF_URL, headers={"User-Agent": "vantadb-eval/1.0"})
        with urllib.request.urlopen(req, timeout=3600) as r, cache.open("wb") as f:
            shutil.copyfileobj(r, f, length=1024 * 1024)
    got = sha256_file(cache)
    if got != expect_sha:
        raise SystemExit(
            f"sha256 mismatch for {cache}\n  expected {expect_sha}\n  got      {got}\n"
            "delete the cache file and re-run to re-download."
        )
    print(f"[fetch] cache sha256 OK ({got})", file=sys.stderr)


def select(questions: list, nonabs: int, abs_: int) -> tuple[list, list]:
    out_non, out_abs = [], []
    for q in questions:
        qid = q.get("question_id", "")
        if qid.endswith("_abs"):
            if len(out_abs) < abs_:
                out_abs.append(q)
        elif len(out_non) < nonabs:
            out_non.append(q)
        if len(out_non) >= nonabs and len(out_abs) >= abs_:
            break
    if len(out_non) < nonabs or len(out_abs) < abs_:
        raise SystemExit(
            f"selection shortfall: got {len(out_non)} non-abs (want {nonabs}) + "
            f"{len(out_abs)} abs (want {abs_}) — dataset file may be truncated."
        )
    return out_non, out_abs


def main() -> int:
    ap = argparse.ArgumentParser(description="Extract the committed LongMemEval-S sub-sample (VER-08).")
    ap.add_argument("--cache", default=str(DEFAULT_CACHE), help="local copy of longmemeval_s_cleaned.json")
    ap.add_argument("--output", default=str(DEFAULT_OUTPUT), help="sub-sample output path")
    ap.add_argument("--nonabs", type=int, default=4, help="number of non-abstention questions to keep")
    ap.add_argument("--abs", dest="abs_", type=int, default=1, help="number of abstention (_abs) questions to keep")
    ap.add_argument("--check", action="store_true", help="only verify the cached sha256; no extraction")
    args = ap.parse_args()

    cache = pathlib.Path(args.cache)
    if args.check and not cache.exists():
        raise SystemExit(f"cache missing at {cache} — run without --check to download it first")
    ensure_cache(cache, SOURCE_SHA256)
    if args.check:
        return 0

    questions = json.loads(cache.read_text(encoding="utf-8"))
    if not isinstance(questions, list) or len(questions) < 100:
        raise SystemExit(f"unexpected dataset shape in {cache} (expected a JSON array of ~500 questions)")

    sel_non, sel_abs = select(questions, args.nonabs, args.abs_)
    subset = sel_non + sel_abs

    out = pathlib.Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(subset, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    out_sha = sha256_file(out)

    meta = {
        "artifact": out.name,
        "artifact_sha256": out_sha,
        "artifact_bytes": out.stat().st_size,
        "source": {
            "dataset": "xiaowu0162/longmemeval-cleaned",
            "split": "longmemeval_s_cleaned",
            "revision": REVISION,
            "file": "longmemeval_s_cleaned.json",
            "file_sha256": SOURCE_SHA256,
            "file_bytes": SOURCE_BYTES,
            "url": HF_URL,
            "license": LICENSE,
        },
        "selection": {
            "rule": "first N non-abstention + first M abstention questions in dataset order; output = non-abs then abs",
            "nonabs": args.nonabs,
            "abs": args.abs_,
            "question_ids": [q["question_id"] for q in subset],
        },
        "note": (
            "Questions are object-identical to the source split (deep-equal, no field edits, "
            "no truncation). Not comparable with published LongMemEval-S leaderboard numbers "
            "at this sample size; use the full split (--nonabs 470 --abs 30 or the raw file) "
            "for reporting."
        ),
    }
    meta_path = out.with_suffix(out.suffix + ".meta.json")
    meta_path.write_text(json.dumps(meta, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    print(f"[fetch] wrote {out} ({out.stat().st_size/1e6:.2f} MB, sha256 {out_sha})")
    print(f"[fetch] wrote {meta_path}")
    print(f"[fetch] questions: {', '.join(meta['selection']['question_ids'])}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
