# FIND-77 — conteo 76→79 en comentarios MCP (post-FIND-90)

> Campaign: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0 · Wave4 · Appetite 1h · 🟢 · 🟢
> Ruta: vanta-worker · Plan: docs/dev/plans/2026-09-15-find-correcciones.md (Task 13)
> Estado: ⬜ PENDING → DISCOVERY completo en esta iteración (task file no existía)

## Objetivo

Comentario `tools.rs:1090` dice "Full profile (76 tools)"; GOV-B6 registró 79 (49+30).
Recontar mecánicamente hoy (post-FIND-90) y sincronizar todos los comentarios de conteo.

## Contrato

- `rg "76 tools|Full profile" vantadb-mcp/src/` → 0 hits en comentarios de conteo
  (literales `1_048_576` en config.rs:96,100 NO son conteo — intactos)
- `cargo check -p vantadb-mcp -j 2` → 0 warnings
- `git diff --check` limpio

## Archivos

- Clave: `vantadb-mcp/src/handlers/tools.rs:1090` (+ `:25`, `:35` mismo bloque — ver Hallazgo)
- Verificar: `vantadb-mcp/src/config.rs:11,14`
- Relacionados (solo lectura, conteo): `vantadb-mcp/src/{skills,code,wiki,context,scenes,threads}.rs`
- Prohibidos: `.opencode/`, `completions/`, `desktop/src-tauri/Cargo.lock`,
  archivos FIND-82/83/87/73/71 (no re-tocar)

## SDP

`campaign_discover_skills_v2` phase=BUILD keywords=[mcp-tools-count,comment-sync,cargo-check] →
8: campaign-executor, source-driven-development, incremental-implementation,
test-driven-development, context-engineering, doubt-driven-development,
frontend-ui-engineering, api-and-interface-design.
Carga: base auto-vía-MCP (campaign-executor/progreso/ponytail); resto no cargadas —
fix comentario 1-bloque no requiere workflows TDD/incremental (sin lógica nueva).
SDP: base-only efectivo.

## Impacto mapeado (Regla 0)

- Leídos completos: `tools.rs:1-65` (bloque MCP-38), `:940-1190`
  (handle_tools_list + profile_allowed_tools Full), `config.rs:1-40`;
  `rg '"name": "'` por módulo (definiciones).
- Referencias hacia dentro: ninguna (comentarios, sin símbolos).
- Referencias entrantes: ninguna (ningún código lee comentarios).
- Veredicto: impacto NULO en comportamiento — solo literales en comentarios.
  Gate D: no dispara (2 archivos, sin API pública, sin hot path, contrato claro).

## Recuento mecánico (2026-09-15, post-FIND-90 commit 26e6ebc0)

| Módulo | Comando | N |
|---|---|---|
| base `tools.rs` (`"name": "`) | `rg -c '"name": "' handlers/tools.rs` | 49 |
| skills.rs | rg -c | 6 |
| code.rs | rg -c | 8 |
| wiki.rs | rg -c | 6 |
| context.rs | rg -c | 1 |
| scenes.rs | rg -c | 3 |
| threads.rs | rg -c | 6 |
| extend total | 6+8+6+1+3+6 | 30 |
| **Full** | **49+30** | **79** |

Full allowlist (`profile_allowed_tools` Full: 22 mem + 28 dev-extra − 1 dup
`embed_texts` = 49 base + 30 extendidos) cubre las 79 definiciones → superficie
real Full = 79. GOV-B6 (79) sigue vigente post-FIND-90 (ese fix solo cambió
accesos a getters, sin alta/baja de tools). ✅ No copiado a ciegas — recontado.
- `rg -c readOnlyHint handlers/tools.rs` = 85; resto de src = 0 → total 85
  (comentario `:35` decía 76 — también stale, mismo bloque, se corrige).

## Hallazgo (scope del fix)

`rg -n '76' vantadb-mcp/src/` → 6 hits: 4 de conteo (config 11,14; tools 25,1090)
+ 1 de conteo-hits (tools 35) + 2 literales tamaño (config 96,100 — NO tocar).
Fix = 5 líneas en 2 archivos (mismo bloque de comentarios).

## Steps

- [x] Step 1 — Fix comentarios + verify contrato + review P2-01 + commit `docs:`
  - Fix inicial 5 líneas (76→79 ×4, 76→85 hits ×1) + review P2-01 (vanta-review:
    REQUEST CHANGES — 2 hallazgos reales: "85 across src" falso, real 115;
    matriz :26-:32 sumaba 76 creando contradicción con cabecera 79).
  - Segunda ronda: parser JSON por-tool (79 rows × 4 hints) →
    ro 46/33 · de 11/68 · id 61/18 · ow 2/77 · hits 85 base / 115 src.
    Reescrito bloque :25-:35 completo (11+/11- en 2 archivos).
    Delta 76→79 = 2 MEM-59 (memory_recall, memory_search) + embed_texts
    (confirma `mcp_tests.rs:4569`).
  - Verify: `rg 76` solo literales 1_048_576 ✅ · stale-patterns 0 ✅ ·
    `cargo check -p vantadb-mcp -j 2` Finished 0 warnings ✅ ·
    `cargo test --test mcp_tests test_mcp_tool_profiles` ok ✅ ·
    `test_mcp_tool_annotations_coverage` ok ✅ · `git diff --check` limpio ✅.

## NOTICED BUT NOT TOUCHING

- `mcp_tests.rs:4553` comentario "matrix (45) and false (31)" stale (real 46/33)
  — el assert (`≥40`) pasa; tocar tests fuera de scope (Regla 0.5).
- Registry `:36-:65` verificado por muestreo (wiki_ingest, skill_create,
  thread_delete hints OK); revisión exhaustiva línea-por-línea no hecha.
