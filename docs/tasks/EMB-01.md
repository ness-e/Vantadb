# EMB-01: Infra embeddings/ + manifest + download.py + verify.py + .gitignore

## Metadata
- **Plan file:** docs/plans/2026-08-28-embeddings-local.md
- **Creado:** 2026-08-28
- **Estado:** ✅ COMPLETED
- **Tipo:** infra + docs
- **Esfuerzo:** 🟢 4-6h | **Prio:** 🔴 Alta

## Blast Radius
- `embeddings/**` — nuevo, sin callers (repo liviano, lazy download)
- `.gitignore:72` — solo ignora `/embeddings/models/`, no afecta build Rust
- `docs/plans/2026-08-28-embeddings-local.md` — spec, sin impacto código
- Verificación: `cargo check -p vantadb` no afectado (sin cambios Rust)

## Contrato
"Get-ChildItem embeddings | Measure == 5 antes de descarga; python -m py_compile embeddings/download.py sale 0; .gitignore contiene /embeddings/models/; manifest.json tiene 9 modelos con dim y rev pinned"

## Herramientas necesarias
- python -m py_compile, python embeddings/download.py --help, python embeddings/download.py --check, python embeddings/verify.py --check

## Steps
### Step 1: Crear embeddings/ con 5 archivos + delta .gitignore
- **Archivos:** `embeddings/manifest.json`, `embeddings/download.py`, `embeddings/verify.py`, `embeddings/README.md`, `embeddings/manifest.lock`, `.gitignore`
- **Acción:**
  - `manifest.json` v1 con 9 modelos rev pinned 7 chars, dims [384,384,768,768,384,512,384,1024,4096], grupos 3 EN /3 ES /3 combined, default multilingual-e5-small, Qwen3 exception onnx=null
  - `download.py` huggingface_hub lazy, argparse --only, --skip-exception, --check, --all, --include-exception; snapshot_download con allow_patterns; escribe manifest.lock
  - `verify.py` ort+tokenizers checks dim+cosine, --check valida estructura sin red, cosine thresholds multi>0.65 / en <0.50, ONNX vs HF >0.98
  - `README.md` tabla 9 modelos + one-liners
  - `manifest.lock` vacío (commitable) con version 1
  - `.gitignore` delta 8 líneas + negaciones README/manifest/download/verify
- **Verify:**
  - `Get-ChildItem embeddings | Measure` == 5 ✅
  - `python -m py_compile embeddings/download.py` exit 0 ✅
  - `python -m py_compile embeddings/verify.py` exit 0 ✅
  - `Select-String .gitignore /embeddings/models/` hit ✅
  - `python embeddings/download.py --help` muestra --only ✅
  - `python embeddings/download.py --check` OK 9 modelos ✅
  - `python embeddings/verify.py --check` OK ✅
  - `manifest.json` 9 modelos dim int + rev len 7 ✅
- **Estado:** ✅ COMPLETED (2026-08-28 — 5 archivos creados, contracts verified, commit feat(EMB-01))

## Context Save Point
- **Fecha:** 2026-08-28
- **Branch:** develop
- **Resultado:** ✅ COMPLETED — contrato EMB-01 cumplido
- **Verificación:** `cargo check -p vantadb` no afectado · `python -m py_compile` 0 · `download.py --check` 0 · `verify.py --check` 0 · `Get-ChildItem 5` · `.gitignore` hit
- **Commit:** feat(EMB-01): infra embeddings/ manifest + download/verify + gitignore
- **Próximo:** EMB-02 — Feature embed-local + LocalOnnxProvider (ort+tokenizers) — depende de EMB-01 OK, no bloqueado

## Archivos tocados
- `embeddings/manifest.json` (nuevo, 9 modelos)
- `embeddings/download.py` (nuevo, 130L, ponytail lazy hub)
- `embeddings/verify.py` (nuevo, 110L, ponytail ort check)
- `embeddings/README.md` (nuevo, tabla 9)
- `embeddings/manifest.lock` (nuevo, vacío)
- `.gitignore` (delta 8 líneas)
- `docs/plans/2026-08-28-embeddings-local.md` (plan, committed)
- `docs/Backlog.md` (Phase 11 insertada)

## Notas
- Ponytail: no crear embeddings/.gitignore separado (root .gitignore cubre); no crear models/ dir hasta download; ~100 líneas por script (no abstracciones)
- WEB-07 IN PROGRESS bloqueaba campaign server — cierre manual documentado; no afecta EMB-01
- Source-driven: huggingface_hub snapshot_download verificado en docs huggingface_hub 1.24.0 (allow_patterns, revision, local_dir)

## SKILLS_CARGADAS
ponytail (full), planning-and-task-breakdown, documentation-and-adrs, source-driven-development, campaign-executor, progreso
