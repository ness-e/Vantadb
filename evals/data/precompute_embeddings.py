#!/usr/bin/env python3
"""evals/data/precompute_embeddings.py — GPU precompute of the LongMemEval embedding cache (ACT-02 Step 9).

Why this exists
---------------
`evals/eval_longmemeval.py` re-embeds every haystack session for every question: ~157K unique
chunks. On a 15 W CPU that is ~42 h (measured: 0.970 s/chunk). The *protocol* does not need to
change to escape that — only who computes the vectors. This script does the embedding elsewhere
(a CUDA GPU), writes `sha256(text) -> vector` to an `.npz`, and the harness replays it through
`--embed-cache` with **0 misses**.

The one thing that must not diverge
-----------------------------------
The harness keys the cache on `sha256(text)` of the *exact* string it would embed. If this script
builds the texts even slightly differently (a missing `Session {sid}:\\n` prefix, a different
chunking threshold), every key misses and the run silently falls back to 42 h of CPU. So this
script does not re-implement any of that: it imports `session_text` and `chunk_session_text` from
the harness and mirrors `VantaAblationStore.put()` literally:

    text = f"Session {sid}:\\n{session_text(session)}"
    texts = [text] if len(text.split()) <= 300 else chunk_session_text(text, target_words=250, overlap=1)

plus `question["question"]` for the query side. Vectors are produced with `embed()` — the same
method `embed_one()`/`embed_batch()` use — never `query_embed()`/`passage_embed()`, which would
produce different vectors than the harness expects.

Usage (local, CPU — slow, only useful as a smoke/tests)
------------------------------------------------------
    python evals/data/precompute_embeddings.py --provider cpu --questions 1 \\
        --output /tmp/cache_1q.npz

Usage (Colab / GPU)
-------------------
    # 1. GPU runtime: Entorno de ejecución -> Cambiar tipo de entorno -> GPU (T4)
    # 2. OnnxRuntime GPU build (environment only — NOT a project dependency):
    #    pip install onnxruntime-gpu
    # 3. Dataset (277,383,467 B, sha256 d6f21ea9…d5a9678c3a442):
    #    curl -L -o longmemeval_s_cleaned.json \\
    #      https://huggingface.co/datasets/xiaowu0162/longmemeval-cleaned/resolve/98d7416c24c778c2fee6e6f3006e7a073259d48f/longmemeval_s_cleaned.json
    # 4. python evals/data/precompute_embeddings.py --provider cuda \\
    #        --dataset longmemeval_s_cleaned.json \\
    #        --output evals/data/embeddings_bge_small_500q.npz

No new project dependency is introduced: the script needs only `fastembed` (already used by the
harness), `numpy` (a fastembed dependency) and `onnxruntime`. If CUDA is requested and absent it
stops with an actionable message instead of quietly running on CPU for 42 h.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import sys
import time
from typing import Any, Dict, List

HERE = pathlib.Path(__file__).resolve().parent          # evals/data
EVALS_DIR = HERE.parent                                  # evals/
REPO_ROOT = EVALS_DIR.parent                              # repo root
sys.path.insert(0, str(EVALS_DIR))

import eval_longmemeval as harness                       # noqa: E402  (path set above)

DEFAULT_DATASET = REPO_ROOT / "datasets" / "longmemeval" / "longmemeval_s_cleaned.json"
DEFAULT_OUTPUT = HERE / "embeddings_bge_small_500q.npz"
META_FILE = HERE / "longmemeval_s_subset.json.meta.json"


def collect_texts(questions: List[Dict[str, Any]]) -> List[str]:
    """Every distinct text the harness will ask the embedder for, in first-seen order.

    Mirrors `VantaAblationStore.put()` and `.search()` exactly; that identity is the whole contract.
    """
    texts: Dict[str, str] = {}  # sha256 -> text (dedupe, keeps first-seen order)

    def add(text: str) -> None:
        key = hashlib.sha256(text.encode("utf-8")).hexdigest()
        if key not in texts:
            texts[key] = text

    n_sessions = n_chunks = 0
    for q in questions:
        ids = list(q.get("haystack_session_ids", []))
        sessions = list(q.get("haystack_sessions", []))
        for sid, sess in zip(ids, sessions):
            # Byte-identical to put(): the prefix is part of the hashed text, and the
            # 300-word test is applied to the *prefixed* string, not to the bare session.
            text = f"Session {sid}:\n{harness.session_text(sess)}"
            n_sessions += 1
            if len(text.split()) <= 300:
                add(text)
                n_chunks += 1
            else:
                chunks = harness.chunk_session_text(text, target_words=250, overlap=1)
                for chunk in chunks:
                    add(chunk)
                n_chunks += len(chunks)

    for q in questions:  # query side of every dense ablation
        add(q["question"])

    print(
        f"[precompute] {len(questions)} preguntas · {n_sessions} ocurrencias de sesión · "
        f"{n_chunks} chunks emitidos · {len(texts)} textos únicos a embedder"
    )
    return list(texts.values())


def verify_dataset_sha256(path: pathlib.Path) -> None:
    """Refuse to precompute against a different file than the harness will read."""
    try:
        meta = json.loads(META_FILE.read_text(encoding="utf-8"))
        expected = meta["source"]["file_sha256"]
    except Exception as e:  # meta is committed, but never let this block a run
        print(f"[precompute] AVISO: no se pudo leer {META_FILE.name} ({e}); no se verifica el sha256")
        return

    h = hashlib.sha256()
    with path.open("rb") as fh:
        for block in iter(lambda: fh.read(1 << 20), b""):
            h.update(block)
    actual = h.hexdigest()
    if actual != expected:
        raise SystemExit(
            f"[precompute] ERROR: sha256 del dataset no coincide.\n"
            f"  esperado: {expected}\n  obtenido: {actual}\n"
            f"  Un dataset distinto produce textos distintos y un caché 100% inútil."
        )
    print(f"[precompute] sha256 del dataset verificado: {actual[:16]}…")


def build_embedder(provider: str):
    """Return (embedder, providers_actually_active). Fail loudly if CUDA was asked and is absent."""
    import onnxruntime
    from fastembed import TextEmbedding

    available = list(onnxruntime.get_available_providers())
    if provider == "cuda":
        if "CUDAExecutionProvider" not in available:
            raise SystemExit(
                "[precompute] ERROR: --provider cuda pero CUDAExecutionProvider no está disponible.\n"
                f"  providers presentes: {available}\n"
                "  Instalá onnxruntime-gpu (entorno, NO dependencia del repo): pip install onnxruntime-gpu\n"
                "  En Colab: Entorno de ejecución → Cambiar tipo de entorno → GPU (T4).\n"
                "  Para correr igual en CPU (lento, ~42 h): --provider cpu"
            )
        chosen = ["CUDAExecutionProvider"]
    elif provider == "cpu":
        chosen = ["CPUExecutionProvider"]
    else:  # auto
        chosen = ["CUDAExecutionProvider"] if "CUDAExecutionProvider" in available else ["CPUExecutionProvider"]

    embedder = TextEmbedding(model_name=harness.EMBED_MODEL, providers=chosen)
    try:
        active = list(embedder.model.model.get_providers())
    except Exception:
        active = chosen
    if provider == "cuda" and "CUDAExecutionProvider" not in active:
        raise SystemExit(f"[precompute] ERROR: se pidió CUDA pero la sesión ONNX usa {active}")
    return embedder, active


def main() -> int:
    parser = argparse.ArgumentParser(description="Precalcula el caché de embeddings de LongMemEval-S (ACT-02 Step 9).")
    parser.add_argument("--dataset", default=str(DEFAULT_DATASET), help="Ruta a longmemeval_s_cleaned.json.")
    parser.add_argument("--output", default=str(DEFAULT_OUTPUT), help="Ruta del .npz a escribir.")
    parser.add_argument("--provider", choices=["auto", "cuda", "cpu"], default="auto", help="ExecutionProvider de ONNX Runtime (default: auto).")
    parser.add_argument("--questions", type=int, default=0, help="Limita a N preguntas (0 = las 500). Usá 1 para probar alineación.")
    parser.add_argument("--batch-size", type=int, default=32, help="Textos por lote (default: 32). Un chunk real ronda 400 tokens: con 256 el arena de atención de ONNX pide ~3.2 GB y revienta por OOM. Subilo solo si tenés VRAM/RAM de sobra.")
    parser.add_argument("--no-verify-sha256", action="store_true", help="Salta la verificación del sha256 del dataset.")
    args = parser.parse_args()

    import numpy as np

    data_path = pathlib.Path(args.dataset)
    if not data_path.is_file():
        raise SystemExit(f"[precompute] ERROR: dataset no encontrado en {data_path}")
    if not args.no_verify_sha256:
        verify_dataset_sha256(data_path)

    raw = json.loads(data_path.read_text(encoding="utf-8"))
    questions: List[Dict[str, Any]] = raw if isinstance(raw, list) else raw.get("questions", [])
    if args.questions > 0:
        questions = questions[: args.questions]

    texts = collect_texts(questions)
    if not texts:
        raise SystemExit("[precompute] ERROR: no hay textos que embedder.")

    embedder, active = build_embedder(args.provider)
    print(f"[precompute] modelo {harness.EMBED_MODEL} ({harness.EMBED_DIMS}d) · providers activos: {active}")

    keys = [hashlib.sha256(t.encode("utf-8")).hexdigest() for t in texts]
    vecs = np.zeros((len(texts), harness.EMBED_DIMS), dtype=np.float32)

    t0 = time.perf_counter()
    done = 0
    bs = max(1, args.batch_size)
    for start in range(0, len(texts), bs):
        batch = texts[start : start + bs]
        out = list(embedder.embed(batch, batch_size=bs))
        if len(out) != len(batch):
            raise SystemExit(f"[precompute] ERROR: embed() devolvió {len(out)} vectores para {len(batch)} textos")
        for i, emb in enumerate(out):
            vecs[start + i] = np.asarray(emb, dtype=np.float32)
        done += len(batch)
        elapsed = time.perf_counter() - t0
        rate = done / elapsed if elapsed > 0 else 0.0
        eta = (len(texts) - done) / rate if rate > 0 else 0.0
        print(
            f"\r[precompute] {done}/{len(texts)} ({100.0 * done / len(texts):5.1f}%) · "
            f"{rate:7.1f} textos/s · ETA {eta / 60:5.1f} min",
            end="",
            flush=True,
        )
    print()

    out_path = pathlib.Path(args.output)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    np.savez(out_path, keys=np.array(keys), vecs=vecs)

    print(
        f"[precompute] escrito {out_path} · {len(keys)} vectores float32 {vecs.shape} "
        f"({out_path.stat().st_size / 1e6:.0f} MB) en {(time.perf_counter() - t0) / 60:.1f} min de embed"
    )
    print(
        "[precompute] verificá con: python evals/eval_longmemeval.py --questions "
        f"{len(questions)} --strict-embeddings --embed-cache {out_path} ... → misses debe ser 0"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
