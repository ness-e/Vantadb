#!/usr/bin/env python3
"""
download.py — descarga lazy de modelos ONNX+HF desde Hugging Face.

Usage:
  python embeddings/download.py --all
  python embeddings/download.py --only multilingual-e5-small,bge-m3
  python embeddings/download.py --only bge-small-en-v1.5 --skip-exception
  python embeddings/download.py --check          # valida manifest sin red (global o --only <id> por modelo + lock)
  python embeddings/download.py --help

Contrato EMB-01: huggingface_hub lazy, --only, --skip-exception, --check,
fallback de rev stale (404 -> resolver main), lock acumulativo.
"""
from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys

# FIND-384: resolver siempre respecto a __file__ (no cwd) para que el script
# funcione invocado desde cualquier directorio de trabajo.
HERE = pathlib.Path(__file__).resolve().parent
MANIFEST = HERE / "manifest.json"
LOCK = HERE / "manifest.lock"
MODELS_DIR = HERE / "models"
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")

# Base recortada FIND-71: sin "*.bin" global (duplicaba "*.safetensors" cuando el
# repo trae ambos → descarga 3-4× lo declarado en README). safetensors es el
# estándar moderno; si un modelo resulta bin-only, añadir entrada explícita
# en MODEL_PATTERNS (lista cerrada por modelo, no re-ampliar el global).
ALLOW_PATTERNS = ["*.json", "*.txt", "tokenizer*", "onnx/*", "*.safetensors"]

# FIND-383: el patron global `onnx/*` arrastra **todas** las variantes ONNX del repo.
# Para `embeddinggemma-2` eso son 30 ficheros (5 variantes x [model + model.onnx_data] + el
# encoder de audio) = 5.888 MB, cuando el encoder de texto solo necesita 2 ficheros
# (`onnx/model.onnx` + `onnx/model.onnx_data` = 1.034 MB). Verificado contra el repo
# `onnx-community/embeddinggemma-2-ONNX` (2026-10-21): 30 ficheros bajo `onnx/`.
# La lista por modelo es cerrada (no se re-amplia el global): cada modelo que lo necesite
# declara aqui sus ficheros exactos.
_GEMMA_PATTERNS = [
    "config.json",
    "config_sentence_transformers.json",
    "preprocessor_config.json",
    "processor_config.json",
    "tokenizer.json",
    "tokenizer_config.json",
    "onnx/model.onnx",
    "onnx/model.onnx_data",
]

MODEL_PATTERNS: dict[str, list[str]] = {
    # embeddinggemma-2: solo el encoder de texto (el de audio no lo usa el motor).
    "embeddinggemma-2": _GEMMA_PATTERNS,
    "embeddinggemma-300m": _GEMMA_PATTERNS,
    "embeddinggemma-2-270m": _GEMMA_PATTERNS,
}

# Aliases de modelos solicitados hacia sus ids canónicos en manifest.json
ALIAS_MAP: dict[str, str] = {
    "embeddinggemma-300m": "embeddinggemma-2",
    "embeddinggemma-2-270m": "embeddinggemma-2",
    "google/embeddinggemma-300m": "embeddinggemma-2",
    "intfloat/multilingual-e5-small": "multilingual-e5-small",
}

# Override explícito por modelo (lista cerrada, no glob amplio).
# Hoy solo embeddinggemma-2 necesita override; el resto expone model.safetensors.


def get_allow_patterns(model_id: str) -> list[str]:
    """Patterns de descarga para un modelo (base recortada salvo override)."""
    return MODEL_PATTERNS.get(model_id, ALLOW_PATTERNS)


def load_manifest() -> dict:
    return json.loads(MANIFEST.read_text(encoding="utf-8"))


# FIND-641: conjunto cerrado de ficheros con hash pinneado en manifest.lock —
# el ONNX declarado (+ sidecar model.onnx_data si existe) y tokenizer.json.
def model_integrity_files(m: dict, model_dir: pathlib.Path) -> list[str]:
    files: list[str] = []
    onnx_rel = m.get("onnx")
    if onnx_rel:
        files.append(onnx_rel)
        data = model_dir / pathlib.PurePosixPath(onnx_rel).parent / "model.onnx_data"
        if data.exists():
            files.append(str(data.relative_to(model_dir)).replace("\\", "/"))
    if (model_dir / "tokenizer.json").exists():
        files.append("tokenizer.json")
    return files


def sha256_file(path: pathlib.Path) -> str:
    import hashlib

    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def hash_model_files(m: dict, model_dir: pathlib.Path) -> dict[str, str]:
    """SHA-256 por fichero del conjunto cerrado (para manifest.lock, FIND-641)."""
    return {
        rel: sha256_file(model_dir / rel)
        for rel in model_integrity_files(m, model_dir)
        if (model_dir / rel).exists()
    }


def write_manifest(manifest: dict) -> None:
    """Escribe manifest.json preservando formato (indent=2 + trailing newline)."""
    MANIFEST.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def load_lock_strict() -> dict:
    """Lee manifest.lock en modo fail-closed (error si falta o JSON inválido)."""
    if not LOCK.exists():
        raise FileNotFoundError(f"manifest.lock no existe: {LOCK}")
    data = json.loads(LOCK.read_text(encoding="utf-8"))
    if not isinstance(data, dict) or not isinstance(data.get("models"), list):
        raise ValueError(f"manifest.lock inválido (falta lista 'models'): {LOCK}")
    return data


def verify_dir_against_lock(
    m: dict,
    local_dir: pathlib.Path,
    lock_entry: dict,
    *,
    delete_on_mismatch: bool = True,
) -> bool:
    """FIND-641 fail-closed: valida todos los hashes pinneados del modelo contra lock."""
    mid = m["id"]
    if lock_entry.get("repo") != m.get("repo") or lock_entry.get("rev") != m.get("rev"):
        print(
            f"[integrity] FAIL {mid}: lock=({lock_entry.get('repo')}@{lock_entry.get('rev')}) "
            f"!= manifest=({m.get('repo')}@{m.get('rev')})",
            file=sys.stderr,
        )
        return False
    sha = lock_entry.get("sha256")
    if not isinstance(sha, dict) or not sha:
        print(f"[integrity] FAIL {mid}: entrada en manifest.lock sin mapa sha256", file=sys.stderr)
        return False
    if m.get("onnx") and m["onnx"] not in sha:
        print(f"[integrity] FAIL {mid}: falta {m['onnx']} en sha256 de manifest.lock", file=sys.stderr)
        return False

    ok = True
    mismatched: list[pathlib.Path] = []
    for rel, expected in sha.items():
        if not isinstance(expected, str) or not SHA256_RE.fullmatch(expected):
            print(f"[integrity] FAIL {mid} {rel}: sha256 malformado en lock ({expected!r})", file=sys.stderr)
            ok = False
            continue
        p = local_dir / rel
        if not p.exists():
            print(f"[integrity] FAIL {mid}: fichero pinneado faltante: {rel}", file=sys.stderr)
            ok = False
            continue
        actual = sha256_file(p)
        if actual != expected:
            print(
                f"[integrity] FAIL {mid} {rel}: sha256 {actual[:16]}… != lock {expected[:16]}…",
                file=sys.stderr,
            )
            mismatched.append(p)
            ok = False

    if not ok and delete_on_mismatch:
        # Eliminar ficheros corruptos/manipulados (y el onnx descargado si falló otro fichero)
        for bad_path in mismatched:
            bad_path.unlink(missing_ok=True)
        if m.get("onnx"):
            (local_dir / m["onnx"]).unlink(missing_ok=True)
    return ok


def merge_lock(new_entries: dict) -> dict:
    """Lock acumulativo: lee lock previo, merge por id (reemplaza si duplicado)."""
    existing: list[dict] = []
    if LOCK.exists():
        try:
            prev = json.loads(LOCK.read_text(encoding="utf-8"))
            existing = prev.get("models", [])
        except Exception:
            existing = []
    by_id = {e["id"]: e for e in existing}
    for e in new_entries["models"]:
        by_id[e["id"]] = e
    return {"version": new_entries["version"], "models": list(by_id.values())}


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="VantaDB embeddings lazy downloader (huggingface_hub)")
    g = p.add_mutually_exclusive_group()
    g.add_argument("--all", action="store_true", help="descarga los 9 modelos (por defecto)")
    p.add_argument("--only", type=str, default=None, help="lista coma-separada de ids (ej: multilingual-e5-small,bge-m3)")
    p.add_argument("--skip-exception", action="store_true", help="omite modelos con exception (Qwen3 16GB)")
    p.add_argument("--check", action="store_true", help="solo valida manifest/lock sin descargar")
    p.add_argument("--dry-run", action="store_true", help="simula descarga (no red) mostrando targets filtrados")
    p.add_argument("--include-exception", action="store_true", help="incluye Qwen3 aunque sea >3GB (alias inverso de --skip-exception)")
    p.add_argument(
        "--update-lock",
        action="store_true",
        help="permite actualizar manifest.lock (y fallback rev stale) tras descargar; por defecto verifica fail-closed contra manifest.lock",
    )
    return p.parse_args()


def filter_models(manifest: dict, args: argparse.Namespace) -> list[dict]:
    models = manifest["models"]
    if args.only:
        raw_wanted = {s.strip() for s in args.only.split(",") if s.strip()}
        wanted = {ALIAS_MAP.get(k, k) for k in raw_wanted}
        models = [m for m in models if m["id"] in wanted]
        missing = wanted - {m["id"] for m in models}
        if missing:
            print(f"[warn] ids no encontrados en manifest: {sorted(missing)}", file=sys.stderr)
    # skip-exception: omite modelos con campo exception (Qwen3)
    skip_exc = args.skip_exception and not args.include_exception
    # por defecto incluir excepción solo si --only la pide o --include-exception; en --all sin flag, incluir? plan dice flag --skip-exception en CI
    # para no romper backward compat, --all incluye todo salvo si --skip-exception
    if skip_exc:
        models = [m for m in models if "exception" not in m]
    # --include-exception fuerza inclusión aunque estuviera filtrado por skip (no-op si ya incluye)
    return models


def check_manifest(manifest: dict) -> bool:
    ok = True
    ids = set()
    for m in manifest.get("models", []):
        for field in ("id", "repo", "rev", "dim", "langs", "license", "group"):
            if field not in m:
                print(f"[check] FAIL {m.get('id','?')}: falta '{field}'", file=sys.stderr)
                ok = False
        if m.get("id") in ids:
            print(f"[check] FAIL duplicado id {m['id']}", file=sys.stderr)
            ok = False
        ids.add(m.get("id"))
        if not isinstance(m.get("dim"), int) or m["dim"] not in (384, 512, 768, 1024, 4096):
            print(f"[check] FAIL {m.get('id')}: dim inválida {m.get('dim')}", file=sys.stderr)
            ok = False
        rev = m.get("rev", "")
        if not isinstance(rev, str) or len(rev) != 7:
            print(f"[check] FAIL {m.get('id')}: rev debe ser 7 chars pinned (got '{rev}')", file=sys.stderr)
            ok = False
        if m.get("onnx") is not None and not isinstance(m["onnx"], str):
            print(f"[check] FAIL {m.get('id')}: onnx debe ser str o null", file=sys.stderr)
            ok = False
    # Al menos 9 modelos esperados
    num_models = len(manifest.get("models", []))
    if num_models < 9:
        print(f"[check] FAIL manifest debe tener al menos 9 modelos, tiene {num_models}", file=sys.stderr)
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
    # balance grupos base (en>=3, es>=3, combined>=3)
    from collections import Counter
    cnt = Counter(m["group"] for m in manifest.get("models", []))
    if cnt["en"] < 3 or cnt["es"] < 3 or cnt["combined"] < 3:
        print(f"[check] FAIL balance grupos en={cnt['en']} es={cnt['es']} combined={cnt['combined']} (esperado en>=3, es>=3, combined>=3)", file=sys.stderr)
        ok = False
    # Qwen3 exception
    qwen = [m for m in manifest.get("models", []) if m["id"] == "qwen3-embedding-8b"]
    if not qwen or "exception" not in qwen[0] or qwen[0]["onnx"] is not None:
        print("[check] FAIL qwen3-embedding-8b debe tener exception y onnx=null", file=sys.stderr)
        ok = False
    if ok:
        print(f"[check] OK manifest v{manifest.get('version')} — {num_models} modelos, rev pinned, dim ok, grupos ok")
    return ok


def main() -> int:
    args = parse_args()
    manifest = load_manifest()

    if args.check:
        ok = check_manifest(manifest)
        # --only: check por modelo, no global (pre-mortem FIND-71)
        if args.only:
            raw_wanted = {s.strip() for s in args.only.split(",") if s.strip()}
            wanted = {ALIAS_MAP.get(k, k) for k in raw_wanted}
            targets = [m for m in manifest.get("models", []) if m["id"] in wanted]
            missing = wanted - {m["id"] for m in targets}
            if missing:
                print(f"[check] FAIL ids no encontrados: {sorted(missing)}", file=sys.stderr)
                ok = False
            else:
                print(f"[check] modelo(s): {sorted(m['id'] for m in targets)}")
            # sizes re-medidos desde manifest.json (Regla 11: la fuente es el manifest)
            for m in targets:
                onnx_mb = m.get("size_onnx_mb") or 0
                hf_mb = m.get("size_hf_mb") or 0
                print(f"[check] {m['id']}: onnx={m.get('size_onnx_mb')} hf={m.get('size_hf_mb')} total={onnx_mb + hf_mb}MB patterns={get_allow_patterns(m['id'])}")
        # lock: fail-closed — manifest.lock obligatorio, 1:1 para todos los modelos ONNX
        try:
            lock = load_lock_strict()
            by_id = {m["id"]: m for m in manifest.get("models", [])}
            lock_by_id = {e.get("id"): e for e in lock.get("models", []) if e.get("id")}
            bad = 0
            for e in lock.get("models", []):
                ref = by_id.get(e.get("id"))
                if ref is None or ref.get("repo") != e.get("repo") or ref.get("rev") != e.get("rev"):
                    print(f"[check] FAIL lock diverge en {e.get('id')}: lock={e} manifest={ref}", file=sys.stderr)
                    bad += 1
                    ok = False
                sha = e.get("sha256")
                if not isinstance(sha, dict) or not sha:
                    print(f"[check] FAIL lock {e.get('id')}: sin mapa sha256", file=sys.stderr)
                    bad += 1
                    ok = False
                else:
                    for rel, h in sha.items():
                        if not isinstance(h, str) or not SHA256_RE.fullmatch(h):
                            print(f"[check] FAIL lock {e.get('id')} sha256[{rel}] no es hex-64: {h!r}", file=sys.stderr)
                            bad += 1
                            ok = False
            for m in manifest.get("models", []):
                if m.get("onnx") is None:
                    continue
                entry = lock_by_id.get(m["id"])
                if entry is None:
                    print(f"[check] FAIL modelo ONNX '{m['id']}' falta en manifest.lock", file=sys.stderr)
                    bad += 1
                    ok = False
                elif m["onnx"] not in (entry.get("sha256") or {}):
                    print(f"[check] FAIL lock '{m['id']}': falta '{m['onnx']}' en sha256", file=sys.stderr)
                    bad += 1
                    ok = False
            if bad == 0:
                scope = "check por modelo" if args.only else "9/9 ONNX completos"
                print(f"[check] lock OK ({LOCK} — {len(lock.get('models', []))} locked, {scope}, repo/rev vs manifest OK)")
        except (OSError, ValueError) as e:
            print(f"[check] FAIL lock inválido o ausente: {e}", file=sys.stderr)
            ok = False
        return 0 if ok else 1

    if args.dry_run:
        targets = filter_models(manifest, args)
        if not targets:
            print("[dry-run] ningún modelo seleccionado", file=sys.stderr)
            return 1
        print(f"[dry-run] {len(targets)} modelo(s) seleccionado(s) — sin descarga (offline ok):")
        for m in targets:
            print(f"  - {m['id']} ({m['repo']}@{m['rev']} dim={m['dim']} onnx={m['onnx']})")
        return 0

    targets = filter_models(manifest, args)
    if not targets:
        print("[download] ningún modelo seleccionado", file=sys.stderr)
        return 1

    lock_by_id: dict[str, dict] = {}
    if not args.update_lock:
        try:
            lock = load_lock_strict()
        except (OSError, ValueError) as e:
            print(f"[error] fail-closed: no se pudo cargar manifest.lock ({e}). Usa --update-lock solo para regenerar.", file=sys.stderr)
            return 1
        lock_by_id = {e["id"]: e for e in lock.get("models", []) if isinstance(e.get("id"), str)}
        for m in targets:
            if m.get("onnx") is None:
                continue
            entry = lock_by_id.get(m["id"])
            if entry is None or entry.get("repo") != m.get("repo") or entry.get("rev") != m.get("rev"):
                print(
                    f"[error] fail-closed: '{m['id']}' ({m.get('repo')}@{m.get('rev')}) "
                    f"sin entrada válida en manifest.lock. Usa --update-lock explícitamente si estás actualizando el lock.",
                    file=sys.stderr,
                )
                return 1

    # lazy import huggingface_hub solo cuando se necesita descargar
    try:
        from huggingface_hub import HfApi, snapshot_download  # type: ignore
    except ImportError:
        print("[error] huggingface_hub no instalado. Instala con: pip install huggingface_hub", file=sys.stderr)
        return 1

    MODELS_DIR.mkdir(parents=True, exist_ok=True)
    for m in targets:
        repo, rev, mid = m["repo"], m["rev"], m["id"]
        local_dir = MODELS_DIR / mid
        # si ya existe en disco, verificar contra lock en lugar de saltar ciegamente
        if local_dir.exists() and m.get("onnx") and (local_dir / m["onnx"]).exists():
            if not args.update_lock:
                if verify_dir_against_lock(m, local_dir, lock_by_id[mid], delete_on_mismatch=False):
                    print(f"[skip] {mid} ya descargado y verificado contra lock en {local_dir}")
                    continue
                print(f"[error] {mid} en {local_dir} falló verificación contra manifest.lock (fail-closed)", file=sys.stderr)
                return 1
            elif any(local_dir.rglob("*.safetensors") or local_dir.rglob("*.bin")):
                print(f"[skip] {mid} ya descargado en {local_dir}")
                continue
        print(f"[download] {mid} <- {repo}@{rev} -> {local_dir}")
        used_rev = rev
        try:
            snapshot_download(
                repo_id=repo,
                revision=rev,
                local_dir=str(local_dir),
                allow_patterns=get_allow_patterns(mid),
            )
        except Exception as e:
            err = str(e)
            # 404 "Revision Not Found": fail-closed salvo --update-lock explícito
            if args.update_lock and ("Revision Not Found" in err or "Invalid rev id" in err or "404" in err):
                try:
                    actual = HfApi().repo_info(repo, repo_type="model").sha
                    actual_short = actual[:7]
                    print(f"[warn] {mid}: rev pinneado '{rev}' stale, fallback a main = {actual_short} (--update-lock)")
                    used_rev = actual_short
                    snapshot_download(
                        repo_id=repo,
                        revision=actual,
                        local_dir=str(local_dir),
                        allow_patterns=get_allow_patterns(mid),
                    )
                    m["rev"] = actual_short
                    write_manifest(manifest)
                    print(f"[manifest] {mid}.rev actualizado a {actual_short}")
                except Exception as e2:
                    print(f"[error] {mid}: fallback falló: {e2}", file=sys.stderr)
                    return 1
            else:
                print(f"[error] {mid}: {e}", file=sys.stderr)
                return 1
        # actualizar rev usado en manifest si difiere del pinneado original (solo --update-lock)
        if args.update_lock:
            m["rev"] = used_rev
        # verificar descarga contra manifest.lock en modo por defecto (fail-closed)
        if not args.update_lock and m.get("onnx") is not None:
            if not verify_dir_against_lock(m, local_dir, lock_by_id[mid], delete_on_mismatch=True):
                print(f"[error] abortando fail-closed: '{mid}' no coincide con manifest.lock", file=sys.stderr)
                return 1
        # si el repo no trae onnx/ y no es excepción, avisar para optimum
        onnx_path = local_dir / (m["onnx"] or "")
        if m["onnx"] and not onnx_path.exists():
            print(f"[warn] {mid}: onnx no encontrado en HF ({m['onnx']}); exporta con: optimum-cli export onnx --model {repo} {local_dir}/onnx/", file=sys.stderr)

    if args.update_lock:
        # manifest.lock acumulativo: solo se escribe cuando se pasa --update-lock.
        lock = merge_lock(
            {
                "version": manifest["version"],
                "models": [
                    {
                        "id": m["id"],
                        "repo": m["repo"],
                        "rev": m["rev"],
                        "sha256": hash_model_files(m, MODELS_DIR / m["id"]),
                    }
                    for m in targets
                    if m.get("onnx") is not None
                ],
            }
        )
        LOCK.write_text(json.dumps(lock, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(f"[lock] escrito {LOCK} ({len(lock['models'])} modelos acumulados)")
    else:
        print(f"[lock] verificado fail-closed contra {LOCK} (sin mutar lock)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
