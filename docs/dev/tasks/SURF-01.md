---
title: "SURF-01 — Public Surface Manifest"
kind: task
status: active
description: "Task file SURF-01 con slices DOC-22 y REL-03"
tags: [surface, manifest, docs]
---

# SURF-01 — Public Surface Manifest + DOC-22 + REL-03

## Metadata

| Campo | Valor |
|---|---|
| ID | SURF-01 (slices DOC-22 authority map, REL-03 manifest versiones generado) |
| Branch | develop |
| Tipo | docs-only + test (no cambia código productivo, no agrega símbolos públicos) |
| Appetite / Esfuerzo / Prioridad | max 3d / 🟡 / 🔴 P0 |
| Uphill / Downhill | 1 (taxonomía multi-superficie) / 2 |
| Origen | Backlog §SURF-01 + §DOC-22 + §REL-03 (análisis surface-reduction 2026-10-08) |
| Pre-req de | API-10 (plan Task 6), CI-08 |
| SDP | documentation-skill, codebase-memory, progreso (solo lectura) + ponytail full. Grep SKILLS-MANIFEST por superficie/manifest/adr/authority → sin skill dedicada (solo menciones de "design authority"); SDP = base + ponytail. |
| SKILLS_CARGADAS | documentation-skill, codebase-memory, progreso |

## Blast Radius

- Workspace members (`Cargo.toml`): `.` (vantadb), `vantadb-python`, `vantadb-server`, `vantadb-mcp`, `vantadb-wasm`, `vantadb-ffi-core`, `vanta-memory`, `vanta-proxy` (default-members excluye proxy/wasm/ffi-core; providers/ no son miembros).
- Packages npm: `vantadb-ts/package.json` (workspace 0.8.0 ✅), `vantadb-node/package.json` (0.7.0 never-published ✅).
- Python: `vantadb-python/pyproject.toml` dynamic version (maturin).
- Maquinaria de paridad existente: `tests/api/public-api.txt`, `tests/api/openapi_yaml_parity.rs`, `tests/version_coherence.rs`, `tests/sdk_serialization.rs`, `scripts/docs/check-npm-versions.mjs` (exit 0 verificado).
- Check-docs baseline HEAD: exit 1 preexistente (`missingFrontmatter=1`: `docs/dev/historial/borrada-2026-10-10.md` ajeno). Contrato = 0 NUEVOS issues.
- PROHIBIDO: `vantadb-python/**` unstaged (WIP ajeno), `stash@{0..2}`, `.opencode/**`.

## Impacto mapeado (Regla 0)

- Archivos leídos completos: `Cargo.toml` (workspace members/default-members), `tests/version_coherence.rs`, `scripts/docs/check-npm-versions.mjs`, `docs/api/BINDINGS_NAMESPACES.md`, `release-plz.toml`, `docs/dev/workflow/PUBLISH.md` §Tag namespace + §DIST-06.
- Referencias hacia dentro (el cambio cita): BINDINGS_NAMESPACES.md, PUBLISH.md, version_coherence.rs, public-api.txt, openapi_yaml_parity.rs, sdk_serialization.rs — solo links, sin duplicar.
- Referencias entrantes: Backlog SURF-01/DOC-22/REL-03/DOC-23; API-10 y CI-08 consumen el manifest.
- Veredicto: impacto docs + 1 test apéndice. Sin símbolos públicos nuevos → Gate D no disparado. `git add` selectivo (solo archivos propios; WIP ajeno intacto).

## Contrato

- `node scripts/docs/check-npm-versions.mjs` exit 0.
- `node scripts/docs/check-docs.mjs` exit 0 en lo nuevo (0 nuevos issues vs HEAD; baseline exit 1 ajeno documentado).
- El manifest existe (`docs/api/SURFACE_MANIFEST.md`, EN INGLÉS, generado) y lista cada superficie con su estado PUBLIC/INTERNAL/EXPERIMENTAL/FROZEN/RETIRED.
- Authority map: tabla concepto→fuente canónica (dentro del manifest, sin duplicar).
- REL-03: archivo GENERADO con versión por superficie; `version_coherence.rs` extendido (no compite).
- Sin símbolos públicos fuera de `tests/api/public-api.txt` (sin cambio de código → trivialmente ✅).
- Fallback aceptado (precedente DOC-22/DOC-23): si el manifest completo no es generable mecánicamente, scope a 1 dimensión (versiones) + DEFER documentado. Decisión tomada: generador fail-closed — lo derivable se deriva (miembros, versiones, publish flags, overrides release-plz); lo taxonómico vive en UNA tabla de overrides visible en el script (no en N docs).

## Herramientas

codegraph (inventario, ya cubierto por lectura directa de manifests), `rg`, `node scripts/docs/check-*.mjs`, `gen-index.mjs --write/--check`, `git` (lectura + commit local). Sin cargo (no está en PATH; el test .rs nuevo queda verificado por CI).

## Steps

- [x] Step 1 — Task file + generador `scripts/docs/gen-surface-manifest.mjs` (fail-closed, overrides visibles, authority map embebido). Verify: `node scripts/docs/gen-surface-manifest.mjs --write` genera el manifest.
- [x] Step 2 — Extendido `tests/version_coherence.rs` (test de sincronía del manifest) + contrato: check-npm exit 0 ✅; check-docs sin nuevos issues gating (missingFrontmatter=1 preexistente ajeno; orphans 5→7 = los 2 archivos nuevos, categoría reporting-only donde ya viven los task files) ✅; check-links: roto único preexistente en snapshot ajeno ✅; manifest `--check` ✅; `gen-index --write` (docs/index.md, docs/api/index.md, llms.txt regenerados) ✅; `git diff --check` ✅. Test .rs nuevo no compilado local (cargo fuera de PATH; verifica CI).
- [x] Step 3 — Review P2-01 ✅ APPROVE (vanta-review, sesión distinta; re-ejecución de contrato; 2 Optional + 1 nit como follow-up, typo tags aplicado) → commit local `docs: ... (SURF-01)` sin push → RESULTADO.

## Dependencias

Wave 0 (no bloqueada). Next: API-10 (Task 6).

## Notas

- La contradicción fila-Backlog vs ADR vs código (ej. MEMG-05/MEMG-09) NO se resuelve acá: el manifest la hace detectable (tercer punto de comparación; desvíos → fila FIND-*).
- Inventario workflows contaminado por FIND-321: no se inventarían workflows; canales solo desde PUBLISH.md + overrides.
- `docs/index.md` solo vía `gen-index.mjs --write`.

## Context Save Point

Sesión subagente 2026-10-15: Steps 1-2 completos y verificados (contrato verde). Step 3 bloqueado en review P2-01: esta sesión no dispone de subagentes para review por agente distinto; diff + evidencia quedan listos sin commitear (pipeline ordena review→commit). WIP ajeno `vantadb-python/**` intacto (staging selectivo obligatorio al commitear). Test .rs nuevo pendiente de compilación en CI (cargo fuera de PATH). Baselines: check-docs `missingFrontmatter=1` (borrada-2026-10-10.md, ajeno); check-npm exit 0 (workspace 0.8.0; node 0.7.0 never-published).

## Review

- Tier: Fast (docs generados + script docs + test apéndice; `docs/api/SURFACE_MANIFEST.md` no es wire format on-disk — el tier adversarial cubre formatos en `src/`).
- OCR delegation: solo reglas advisory, sin hallazgos Critical/High (output = rule groups; `==` detectado vive en `check-npm-versions.mjs` preexistente, fuera de scope).
- Reviewer: vanta-review, sesión distinta al implementador (veredicto 2026-10-15, re-ejecución de contrato).
- Veredicto: ✅ APPROVE SURF-01 — fail-closed real, sin duplicación, test liveness con estilo del archivo, índices solo vía generador, diff sin prohibidos. 2 Optional (aserción substring débil + nota allowance 0.7.0 → follow-up API-10) + 1 nit (typo tags, aplicado). Compilación del test .rs delegada a CI (cargo ausente local).
