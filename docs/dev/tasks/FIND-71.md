# FIND-71 — embeddings peso + verify + sizes (download patterns + smoke)

> **Plan:** `docs/dev/plans/2026-09-15-find-correcciones.md` Task 12 (Wave3) · **Appetite:** 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🟡
> **Branch:** develop · **Commit:** `fix:` · **Ruta:** vanta-worker · **Estado:** ⬜ PENDING → IN PROGRESS
> **Campaign:** 6ab26f3f-cf16-4416-9255-c18cca0bcaf0 · **Previa:** FIND-70 ✅ · **Next:** Wave4
> **SDP:** campaign-executor, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development (discover_skills_v2 BUILD, keywords embeddings-download/allow-patterns/verify-smoke/manifest-lock; frontend-ui-engineering/api-and-interface-design descartadas por no aplicar — sin web/, sin API pública)

## Objetivo

Recortar `ALLOW_PATTERNS` amplios (`*.safetensors` + `*.bin` duplicados descargan 3-4× lo declarado) a base recortada + override por modelo; aclarar `verify.py:135-192` como smoke (dummies/skips, no verificación numérica); re-medir sizes reales en README desde `manifest.json`; manifest.lock verificado; `download --check` verde offline con check por modelo.

## Contrato (ley)

- `ALLOW_PATTERNS` recortado en `embeddings/download.py:26` (sin `*.bin` global) + `MODEL_PATTERNS`/`get_allow_patterns()` por modelo si hace falta.
- `verify`→smoke aclarado en docs (docstring + README + `verify.log` note).
- Sizes reales en `embeddings/README.md` re-medidos desde `manifest.json` (no copiados a ciegas, Regla 11).
- `manifest.lock` verificado (entradas vs manifest repo/rev).
- `python embeddings/download.py --check` verde offline (global + `--only` por modelo), sin red en CI.
- **Stop:** modelo sin pattern claro → lista explícita por modelo, no glob.

## Archivos

- **Clave:** `embeddings/download.py:26`, `embeddings/verify.py:135-192`, `embeddings/README.md`, `embeddings/manifest.lock` (+ `manifest.json` source-of-truth, solo lectura).
- **Relacionados:** modelos referenciados, embeddings config.
- **Prohibidos (NO TOCAR):** `.opencode/`, `completions/`, `desktop/src-tauri/Cargo.lock`, `stash@{0}`, archivos FIND-87 y FIND-73, `Cargo.toml`, WIP ajeno en `git status` (pipeline-state.json, completions, tauri lock).

## Dependencias

Wave3 sin bloqueantes, paralela con FIND-87/FIND-73 disjunta (archivos distintos). Previa FIND-70 ✅. Next Wave4.

## Impacto mapeado (Regla 0)

- **Leídos completos:** `embeddings/download.py` (226L), `embeddings/verify.py` (229L), `embeddings/README.md` (90L), `embeddings/manifest.json` (115L), `embeddings/manifest.lock` (20L), `embeddings/verify.log` (20L), `embeddings/sanity_embed.py` (162L, referencia de verificación real).
- **Referencias hacia dentro (qué usa el slice):** `download.py` → `manifest.json` (load), `manifest.lock` (merge_lock), `huggingface_hub` (lazy, solo en descarga real, no en `--check`); `verify.py` → `manifest.json`, `models/` en disco, `ort`/`tokenizers` (lazy, solo en verify descargado); README → `manifest.json` (tabla sizes), `docs/user/tutorials/05-embedding-integrations.md:126`, `docs/api/EMBEDDINGS.md`, `benchmarks/embed_bench.py`.
- **Referencias entrantes (quién usa lo que toco):** `ALLOW_PATTERNS` solo usado en `download.py:184,199` (2× `snapshot_download`); `verify_model` solo llamado en `verify.py:213`; ningún importador externo (`rg` + codegraph: 0 callers fuera de `embeddings/`; codegraph solo halla `download_hdf5` y `verifyClaim` no relacionados). CI no invoca descarga (solo `--check` offline).
- **Veredicto:** blast radius = 4 archivos en `embeddings/`, 0 callers externos, sin hot path Rust, sin API pública, sin trust boundary nuevo. `get_allow_patterns()` es helper interno del script (no `pub fn`/endpoint/binding) → Gate D no dispara; sin `## Spec` (fix acotado sobre comportamiento existente, SPEC.md no existe por plan). Riesgo principal: recorte rompe modelo bin-only → mitigado con `MODEL_PATTERNS` override + check por modelo.

## Gate D

No dispara: blast 4 archivos <10, sin hot path, sin API pública, contrato no ambiguo (patterns/docstring/sizes/lock), helper nuevo interno (no símbolo público). Sin `question` al usuario.

## Investigación código (DISCOVERY)

- `codegraph_explore "embeddings download verify manifest"` → 11 símbolos en 3 files; `ALLOW_PATTERNS` 2 usos (`main` download + fallback); `verify_model` 1 caller; 0 tests cubriendo; resto (`download_hdf5`, `verifyClaim`) no relacionados.
- `check_index_coverage` → `no_recorded_issue` en los 3 paths (best-effort, fuente leída directa como verdad).
- `download.py:174` skip-if-downloaded chequea `*.onnx` + (`*.safetensors`|`*.bin`); coherente con recorte (safetensors basta).
- `verify.log:18` lock con 3 modelos (subset acumulativo, válido); `verify.py:123` lee `locked`/`models` (ambas claves).

## Investigación problema

- Globs `*.safetensors` + `*.bin` traen ambos frameworks cuando el repo trae ambos → ~2× en pesos HF + `onnx/*` ya cubre ONNX → total real 3-4× lo declarado en tabla (size_hf_mb = 1 formato).
- `verify_model` no hace forward real: tokeniza, abre sesión ort, imprime OK con dim del manifest + thresholds como hints impresos (no asertados); skips (no descargado / sin ort) retornan True. Es smoke, no verify numérico (lo numérico real vive en `sanity_embed.py` con thresholds por modelo).
- Tradeoff glob vs explícita: glob simple pero sobre-descarga; lista explícita precisa pero frágil ante renames upstream. Decisión: base recortada (safetensors, estándar moderno) + override explícito por modelo (escape hatch sin re-ampliar global).

## Investigación internet

[NO VERIFICADA — sin ambigüedad que la exija]: formato safetensors/manifest estándar conocido desde repo (todos los repos citados exponen `model.safetensors`; override cubre el caso bin-only sin necesidad de red). Sin red en CI por contrato.

## Steps

- [x] **Step 1 — RED + DISCOVERY:** gaps reproducidos 4/4 (ALLOW amplio, `--check` sin `--only`, lock sin repo/rev, smoke sin documentar) + task file con Regla 0. Verify: script RED 4/4 OK.
- [x] **Step 2 — GREEN download.py:** `ALLOW_PATTERNS` recortado (sin `*.bin`) + `MODEL_PATTERNS`/`get_allow_patterns()` + `--check --only` por modelo + validación lock repo/rev + sizes por modelo. Verify: `download --check` global OK + 9/9 `--only` verde + tamper-lock FAIL→restore OK + `py_compile` OK + `git diff --check` OK.
- [x] **Step 3 — GREEN verify.py + README + lock:** docstring smoke + `write_verify_log` nota smoke→sanity + README (patterns/sizes re-medidos 2026-09-15/smoke) + `download.py` usage. Lock real intacto (3 entries, repo/rev vs manifest OK). Verify: `verify --check` PASS + `download --check` OK + `py_compile` OK + `git diff --check` OK. GREEN 4/4 gaps cerrados.
- [x] **Step 4 — CIERRE:** review P2-01 approve (vanta-review, 0 bloqueantes) + commit `fix:` (solo propios) + lesson + RESULTADO §7. Backlog sync vía progreso/orquestador (race Wave3 paralelo, precedente FIND-63/88); plan file recitation añadida pero unstaged (compartido con FIND-87/73).

## Verificación contrato

```bash
python embeddings/download.py --check
python embeddings/download.py --check --only <cada uno de los 9 ids>
python embeddings/verify.py --check
python -m py_compile embeddings/download.py embeddings/verify.py
git diff --check
```

Sin red en CI (todo offline/`--check`). `campaign_verify_cmd` con bug exit -1 conocido → bash directa + mención en RESULTADO.

## DoD 3 niveles

1. **Contrato:** patterns recortados, check por modelo verde, docs coherentes (verify→smoke), lock verificado, `download --check` offline OK.
2. **Repo:** solo archivos propios en commit; prohibidos intactos; sin deuda neta; `py_compile` + `diff --check` limpios.
3. **Proceso:** reviewer P2-01 distinto, task file sync, recitation, lesson, RESULTADO §7.

## Context Save Point

- RED 4/4 reproducido 2026-09-15 (bash python -c, output en historial).
- Baseline `--check` global verde pre-fix (download + verify OK, compile OK, diff-check OK salvo warning CRLF ajeno en completions).
- WIP ajeno detectado (`git status`): `.opencode`, `completions/*`, `desktop/src-tauri/Cargo.lock`, `docs/pipeline-state.json` — NO tocar, NO `git add -A`.
