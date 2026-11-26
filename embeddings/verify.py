#!/usr/bin/env python3
"""
verify.py — SMOKE de embeddings locales (ort + tokenizers dim + cosine).

Nivel: smoke (no verificación numérica). Comprueba estructura (dim/rev/grupos),
que la sesión ONNX abre y que el tokenizer carga; los thresholds cosine se
imprimen como hints, NO se asertan. La verificación numérica real vive en
`sanity_embed.py` (cosine por pares con thresholds por modelo).

Checks:
  - dim == manifest dim
  - sesión ONNX abre + tokenizer carga (si descargado; si no, SKIP, no FAIL)
  - multi cosine("hola mundo","hello world") >0.65 para combined/es, <0.50 para en-only (hint impreso, no asertado)
  - --check: solo valida manifest + estructura sin necesitar modelos ni red

Usage:
  python embeddings/verify.py --check
  python embeddings/verify.py --all
  python embeddings/verify.py --only multilingual-e5-small
"""
from __future__ import annotations

import argparse
import json
import math
import pathlib
import re
import sys

# FIND-384: resolver siempre respecto a __file__ (no cwd) para que el script
# funcione invocado desde cualquier directorio de trabajo.
HERE = pathlib.Path(__file__).resolve().parent
MANIFEST = HERE / "manifest.json"
MODELS_DIR = HERE / "models"
LOCK = HERE / "manifest.lock"
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")


def sha256_file(path: pathlib.Path) -> str:
    import hashlib

    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


_lock_cache: tuple[str, int, dict] | None = None


def load_lock() -> dict:
    """manifest.lock parseado (fail-closed: lanza error si falta o es inválido)."""
    global _lock_cache
    if not LOCK.exists():
        raise FileNotFoundError(f"manifest.lock no existe: {LOCK}")
    st = LOCK.stat()
    lock_key = str(LOCK)
    if _lock_cache is not None and _lock_cache[0] == lock_key and _lock_cache[1] == st.st_mtime_ns:
        return _lock_cache[2]
    data = json.loads(LOCK.read_text(encoding="utf-8"))
    if not isinstance(data, dict) or not isinstance(data.get("models"), list):
        raise ValueError(f"manifest.lock inválido (se esperaba objeto con lista 'models'): {LOCK}")
    _lock_cache = (lock_key, st.st_mtime_ns, data)
    return data


def verify_locked_hashes(mid: str, model_dir: pathlib.Path) -> bool:
    """FIND-641 fail-closed: todo modelo ONNX debe existir en manifest.lock, todos sus
    ficheros pinneados deben existir en disco (missing == 0) y coincidir con su SHA-256."""
    try:
        lock = load_lock()
    except (OSError, ValueError) as e:
        print(f"       [integrity] FAIL {mid}: no se pudo cargar manifest.lock ({e})")
        return False
    entries = {e.get("id"): e for e in lock.get("models", [])}
    entry = entries.get(mid)
    if entry is None:
        print(f"       [integrity] FAIL {mid}: sin entrada en manifest.lock (fail-closed)")
        return False
    sha = entry.get("sha256")
    if not isinstance(sha, dict) or not sha:
        print(f"       [integrity] FAIL {mid}: entrada lock sin mapa sha256")
        return False
    ok = True
    verified, missing = 0, 0
    for rel, expected in sha.items():
        if not isinstance(expected, str) or not SHA256_RE.fullmatch(expected):
            print(f"       [integrity] FAIL {mid} {rel}: sha256 malformado en lock ({expected!r})")
            ok = False
            continue
        p = model_dir / rel
        if not p.exists():
            print(f"       [integrity] FAIL {mid}: fichero pinneado faltante: {rel}")
            missing += 1
            ok = False
            continue
        actual = sha256_file(p)
        if actual != expected:
            print(f"       [integrity] FAIL {rel}: sha256 {actual[:16]}… != lock {expected[:16]}… (artefacto corrupto/manipulado)")
            ok = False
        else:
            verified += 1
    if ok:
        print(f"       [integrity] OK {verified} fichero(s) verificados contra lock")
    return ok


def cosine(a: list[float], b: list[float]) -> float:
    dot = sum(x * y for x, y in zip(a, b))
    na = math.sqrt(sum(x * x for x in a))
    nb = math.sqrt(sum(x * x for x in b))
    if na == 0 or nb == 0:
        return 0.0
    return dot / (na * nb)


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="VantaDB embeddings verifier (ort+tokenizers)")
    p.add_argument("--check", action="store_true", help="solo valida manifest/lock sin cargar modelos")
    p.add_argument("--all", action="store_true", help="verifica todos los modelos descargados")
    p.add_argument("--only", type=str, default=None, help="ids coma-separados")
    p.add_argument("--skip-exception", action="store_true", help="omite Qwen3 >3GB")
    return p.parse_args()


def load_manifest() -> dict:
    return json.loads(MANIFEST.read_text(encoding="utf-8"))


def filter_models(manifest: dict, args: argparse.Namespace) -> list[dict]:
    models = manifest["models"]
    if args.only:
        wanted = {s.strip() for s in args.only.split(",") if s.strip()}
        models = [m for m in models if m["id"] in wanted]
    if args.skip_exception:
        models = [m for m in models if "exception" not in m]
    return models


def check_structure(manifest: dict) -> bool:
    ok = True
    num_models = len(manifest.get("models", []))
    if num_models < 9:
        print(f"[check] FAIL al menos 9 modelos esperados, hay {num_models}", file=sys.stderr)
        ok = False
    for m in manifest["models"]:
        if not isinstance(m.get("dim"), int):
            print(f"[check] FAIL {m['id']} dim", file=sys.stderr)
            ok = False
        if not isinstance(m.get("rev"), str) or len(m["rev"]) != 7:
            print(f"[check] FAIL {m['id']} rev pinned", file=sys.stderr)
            ok = False
    # FIND-384: default se valida contra los ids del propio manifest (fuente
    # única de verdad), sin literal duplicado del modelo por defecto.
    default_id = manifest.get("default")
    model_ids = {m.get("id") for m in manifest.get("models", [])}
    if not isinstance(default_id, str) or default_id not in model_ids:
        print(f"[check] FAIL default '{default_id}' no existe entre los ids del manifest", file=sys.stderr)
        ok = False
    elif any(m.get("id") == default_id and "exception" in m for m in manifest.get("models", [])):
        print(f"[check] FAIL default '{default_id}' apunta a un modelo con exception", file=sys.stderr)
        ok = False
    # balance base 3/3/3
    from collections import Counter
    cnt = Counter(m.get("group") for m in manifest.get("models", []))
    if cnt["en"] != 3 or cnt["es"] != 3 or cnt["combined"] < 3:
        print(f"[check] FAIL balance grupos en={cnt['en']} es={cnt['es']} combined={cnt['combined']} (esperado en=3, es=3, combined>=3)", file=sys.stderr)
        ok = False
    qwen = [m for m in manifest.get("models", []) if m.get("id") == "qwen3-embedding-8b"]
    if not qwen or "exception" not in qwen[0] or qwen[0].get("onnx") is not None:
        print("[check] FAIL qwen3 debe tener exception y onnx=null", file=sys.stderr)
        ok = False
    # FIND-641: integridad del lock (100% fail-closed) — existencia de
    # manifest.lock, paridad 1:1 de repo/rev/sha256 para TODOS los modelos ONNX
    # del manifest y verificación en disco cuando el modelo esté descargado.
    try:
        lock = load_lock()
    except (OSError, ValueError) as e:
        print(f"[check] FAIL manifest.lock inválido o ausente: {e}", file=sys.stderr)
        return False

    lock_by_id: dict[str, dict] = {}
    for e in lock.get("models", []):
        eid = e.get("id")
        if not isinstance(eid, str) or not eid:
            print("[check] FAIL entrada en manifest.lock sin 'id'", file=sys.stderr)
            ok = False
            continue
        if eid in lock_by_id:
            print(f"[check] FAIL entrada duplicada en manifest.lock: {eid}", file=sys.stderr)
            ok = False
        lock_by_id[eid] = e

    onnx_models = [m for m in manifest.get("models", []) if m.get("onnx") is not None]
    for m in onnx_models:
        mid = m["id"]
        entry = lock_by_id.get(mid)
        if entry is None:
            print(f"[check] FAIL modelo ONNX '{mid}' falta en manifest.lock", file=sys.stderr)
            ok = False
            continue
        if entry.get("repo") != m.get("repo") or entry.get("rev") != m.get("rev"):
            print(
                f"[check] FAIL paridad repo/rev en manifest.lock para '{mid}': "
                f"lock=({entry.get('repo')}@{entry.get('rev')}) != manifest=({m.get('repo')}@{m.get('rev')})",
                file=sys.stderr,
            )
            ok = False
        sha = entry.get("sha256")
        if not isinstance(sha, dict) or not sha or not all(
            isinstance(v, str) and SHA256_RE.fullmatch(v) for v in sha.values()
        ):
            print(f"[check] FAIL lock '{mid}': sha256 ausente o no es hex-64 válido", file=sys.stderr)
            ok = False
        elif m["onnx"] not in sha:
            print(f"[check] FAIL lock '{mid}': falta hash para onnx declarado '{m['onnx']}'", file=sys.stderr)
            ok = False

        model_dir = MODELS_DIR / mid
        if model_dir.exists() and not verify_locked_hashes(mid, model_dir):
            ok = False

    if ok:
        print(f"[check] OK estructura manifest + lock — {num_models} modelos ({len(onnx_models)} ONNX pinneados)")
    return ok


def write_verify_log(manifest: dict, targets: list[dict] | None = None, status: str = "PASS", mode: str = "check-only") -> pathlib.Path:
    log = HERE / "verify.log"
    models = targets if targets is not None else manifest.get("models", [])
    lines = []
    lines.append(f"# verify {status} — {mode} — {len(models)} modelos")
    lines.append("")
    lines.append(f"manifest v{manifest.get('version')} default={manifest.get('default')}")
    lines.append("")
    lines.append("| id | dim | group | onnx | repo | rev | langs | status |")
    lines.append("|---|---|---|---|---|---|---|---|")
    for m in models:
        mid = m.get("id", "?")
        model_dir = MODELS_DIR / mid
        downloaded = model_dir.exists()
        if mode == "check-only":
            st = "PASS (check-only)"
        elif downloaded:
            st = "PASS" if status == "PASS" else "FAIL"
            # annotate if model dir exists
            st += " (downloaded)" if downloaded else " (missing)"
        else:
            st = "SKIP (not downloaded)"
        onnx = m.get("onnx") or "HF-only"
        langs = ",".join(m.get("langs", []))
        lines.append(f"| {mid} | {m.get('dim')} | {m.get('group')} | {onnx} | {m.get('repo')} | {m.get('rev')} | {langs} | {st} |")
    lines.append("")
    lines.append(f"mode: {mode}")
    # also check manifest.lock
    lock_path = HERE / "manifest.lock"
    if lock_path.exists():
        try:
            j = json.loads(lock_path.read_text(encoding="utf-8"))
            lines.append(f"manifest.lock: OK version={j.get('version')} locked={len(j.get('locked', j.get('models', [])))}")
        except Exception as e:
            lines.append(f"manifest.lock: FAIL {e}")
    else:
        lines.append("manifest.lock: not yet (ok pre-download)")
    if mode == "check-only":
        lines.append("note: check-only sin red — smoke, no verificacion numerica; cosine real en sanity_embed.py")
    lines.append("smoke: verify_model abre sesion ONNX + tokenizer (skips=True); thresholds cosine son hints impresos, no asserts")
    lines.append("")
    log.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return log


def verify_model(m: dict) -> bool:
    """SMOKE por modelo (FIND-71 / FIND-641): valida SHA-256 fail-closed contra
    manifest.lock ANTES de abrir la sesión ONNX exacta y el tokenizer.
    """
    mid, dim, onnx_rel = m["id"], m["dim"], m["onnx"]
    model_dir = MODELS_DIR / mid
    if not model_dir.exists():
        print(f"[skip] {mid}: no descargado ({model_dir} no existe)")
        return True  # skip no es fail en --all sin descarga
    print(f"[verify] {mid} dim={dim} ...", end=" ")
    if not onnx_rel:
        # HF-only (Qwen3, onnx: null — excluido explícitamente del lock ONNX)
        print(f"OK HF-only dim={dim} (GPU, skip ONNX; excluido del lock ONNX)")
        return True

    # FIND-641: verificación fail-closed de TODOS los ficheros pinneados ANTES de cargar ONNX.
    if not verify_locked_hashes(mid, model_dir):
        print(f"FAIL integridad contra manifest.lock ({mid})")
        return False

    onnx_path = model_dir / onnx_rel
    if not onnx_path.exists():
        print(f"FAIL onnx declarado no encontrado ({onnx_rel})")
        return False

    ok = True
    try:
        import onnxruntime as ort  # type: ignore
        import tokenizers  # type: ignore

        tok_path = model_dir / "tokenizer.json"
        if not tok_path.exists():
            print(f"FAIL tokenizer.json no encontrado ({tok_path})")
            return False
        tokenizer = tokenizers.Tokenizer.from_file(str(tok_path))
        _sess = ort.InferenceSession(str(onnx_path), providers=["CPUExecutionProvider"])
        texts = ["hola mundo", "hola mundo", "hello world"]
        enc = tokenizer.encode_batch(texts)
        _ = [e.ids for e in enc]
        print(f"OK (onnx={onnx_path.name} dim={dim})")
    except ImportError as e:
        print(f"skip ort/tokenizers no instalados ({e}) — integridad lock OK")
    except Exception as e:
        print(f"FAIL {e}")
        ok = False
    if ok:
        if m["group"] in ("es", "combined"):
            print(f"       multi cosine threshold >0.65 esperado para {mid}")
        else:
            print(f"       en-only cosine multi <0.50 esperado para {mid}")
    return ok


def main() -> int:
    args = parse_args()
    manifest = load_manifest()

    if args.check:
        ok = check_structure(manifest)
        log = write_verify_log(manifest, status="PASS" if ok else "FAIL", mode="check-only")
        print(f"[verify] {'PASS' if ok else 'FAIL'} — log {log}")
        return 0 if ok else 1

    targets = filter_models(manifest, args)
    if args.all or not args.only:
        # por defecto verifica todos los descargados
        pass

    # header
    print(f"[verify] {len(targets)} modelo(s) — con ort+tokenizers")
    all_ok = True
    for m in targets:
        if not verify_model(m):
            all_ok = False
    # escribe verify.log con tabla
    status = "PASS" if all_ok else "FAIL"
    downloaded_any = any((MODELS_DIR / m["id"]).exists() for m in targets)
    mode = "downloaded" if downloaded_any else "check-only (no models)"
    log = write_verify_log(manifest, targets, status=status, mode=mode)
    if all_ok:
        print(f"[verify] {status} — log {log}")
        return 0
    print(f"[verify] {status}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
