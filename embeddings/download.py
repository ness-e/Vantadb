#!/usr/bin/env python3
"""
download.py — descarga lazy de modelos ONNX+HF desde Hugging Face.

Usage:
  python embeddings/download.py --all
  python embeddings/download.py --only multilingual-e5-small,bge-m3
  python embeddings/download.py --only bge-small-en-v1.5 --skip-exception
  python embeddings/download.py --check          # valida manifest sin red
  python embeddings/download.py --help

Contrato EMB-01: huggingface_hub lazy, --only, --skip-exception, --check
"""
from __future__ import annotations

import argparse
import json
import pathlib
import sys

MANIFEST = pathlib.Path(__file__).parent / "manifest.json"
LOCK = pathlib.Path(__file__).parent / "manifest.lock"
MODELS_DIR = pathlib.Path(__file__).parent / "models"

ALLOW_PATTERNS = ["*.json", "*.txt", "tokenizer*", "onnx/*", "*.safetensors", "*.bin"]


def load_manifest() -> dict:
    return json.loads(MANIFEST.read_text(encoding="utf-8"))


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="VantaDB embeddings lazy downloader (huggingface_hub)")
    g = p.add_mutually_exclusive_group()
    g.add_argument("--all", action="store_true", help="descarga los 9 modelos (por defecto)")
    p.add_argument("--only", type=str, default=None, help="lista coma-separada de ids (ej: multilingual-e5-small,bge-m3)")
    p.add_argument("--skip-exception", action="store_true", help="omite modelos con exception (Qwen3 16GB)")
    p.add_argument("--check", action="store_true", help="solo valida manifest/lock sin descargar")
    p.add_argument("--dry-run", action="store_true", help="simula descarga (no red) mostrando targets filtrados")
    p.add_argument("--include-exception", action="store_true", help="incluye Qwen3 aunque sea >3GB (alias inverso de --skip-exception)")
    return p.parse_args()


def filter_models(manifest: dict, args: argparse.Namespace) -> list[dict]:
    models = manifest["models"]
    if args.only:
        wanted = {s.strip() for s in args.only.split(",") if s.strip()}
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
    # 9 modelos esperados
    if len(manifest.get("models", [])) != 9:
        print(f"[check] FAIL manifest debe tener 9 modelos, tiene {len(manifest.get('models', []))}", file=sys.stderr)
        ok = False
    if manifest.get("default") != "multilingual-e5-small":
        print(f"[check] FAIL default debe ser multilingual-e5-small", file=sys.stderr)
        ok = False
    # balance 3/3/3
    from collections import Counter
    cnt = Counter(m["group"] for m in manifest.get("models", []))
    if cnt["en"] != 3 or cnt["es"] != 3 or cnt["combined"] != 3:
        print(f"[check] FAIL balance grupos en={cnt['en']} es={cnt['es']} combined={cnt['combined']} (esperado 3/3/3)", file=sys.stderr)
        ok = False
    # Qwen3 exception
    qwen = [m for m in manifest.get("models", []) if m["id"] == "qwen3-embedding-8b"]
    if not qwen or "exception" not in qwen[0] or qwen[0]["onnx"] is not None:
        print("[check] FAIL qwen3-embedding-8b debe tener exception y onnx=null", file=sys.stderr)
        ok = False
    if ok:
        print(f"[check] OK manifest v{manifest.get('version')} — 9 modelos, rev pinned, dim ok, grupos 3/3/3")
    return ok


def main() -> int:
    args = parse_args()
    manifest = load_manifest()

    if args.check:
        ok = check_manifest(manifest)
        # también valida lock si existe
        if LOCK.exists():
            try:
                json.loads(LOCK.read_text(encoding="utf-8"))
                print(f"[check] lock OK ({LOCK})")
            except Exception as e:
                print(f"[check] lock JSON inválido: {e}", file=sys.stderr)
                ok = False
        else:
            print(f"[check] lock no existe aún ({LOCK}) — ok para EMB-01")
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

    # lazy import huggingface_hub solo cuando se necesita descargar
    try:
        from huggingface_hub import snapshot_download  # type: ignore
    except ImportError:
        print("[error] huggingface_hub no instalado. Instala con: pip install huggingface_hub", file=sys.stderr)
        return 1

    MODELS_DIR.mkdir(parents=True, exist_ok=True)
    for m in targets:
        repo, rev, mid = m["repo"], m["rev"], m["id"]
        local_dir = MODELS_DIR / mid
        print(f"[download] {mid} <- {repo}@{rev} -> {local_dir}")
        try:
            snapshot_download(
                repo_id=repo,
                revision=rev,
                local_dir=str(local_dir),
                allow_patterns=ALLOW_PATTERNS,
            )
            # si el repo no trae onnx/ y no es excepción, avisar para optimum
            onnx_path = local_dir / (m["onnx"] or "")
            if m["onnx"] and not onnx_path.exists():
                print(f"[warn] {mid}: onnx no encontrado en HF ({m['onnx']}); exporta con: optimum-cli export onnx --model {repo} {local_dir}/onnx/", file=sys.stderr)
        except Exception as e:
            print(f"[error] fallo {mid}: {e}", file=sys.stderr)
            return 1

    # escribe manifest.lock con shas resueltos (simplified: rev + repo)
    lock = {"version": manifest["version"], "models": [{"id": m["id"], "repo": m["repo"], "rev": m["rev"]} for m in targets]}
    LOCK.write_text(json.dumps(lock, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"[lock] escrito {LOCK} ({len(targets)} modelos)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
