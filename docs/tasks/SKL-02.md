# SKL-02: Corregir y modernizar `skills/vantadb-mcp/` — SKILL.md + references

## Metadata
- **Plan file:** `docs/plans/2026-08-17-skills-vantadb.md` (wave SKL)
- **Fuente:** diagnóstico del lead 2026-08-17 (Backlog P21) — skill MCP desactualizada contra el MCP server real
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Tipo:** Docs (skills de proyecto) — no toca `vantadb-mcp/` crate
- **Turns estimados:** 3-5
- **Creado:** 2026-08-17
- **Estado:** ✅ COMPLETED (2026-08-17 — incluye fix P2-01: SKILL.md:8 "4 resources" → "2 resources listados + 2 servibles")
- **Incógnitas (uphill):** 0 — diagnóstico completo
- **Pendientes (downhill):** 3 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `skills/vantadb-mcp/SKILL.md` (223L), `skills/vantadb-mcp/references/*.md` (3 archivos), `skills/vantadb-mcp/assets/*.json` (3 archivos, SKL-03) |
| Callees | MCP server real: `vantadb-mcp/src/handlers/tools.rs` (14 tools), `docs/api/MCP.md` (fuente de verdad del contrato MCP — leer, NO editar), `vantadb-server/src/main.rs` (flags reales del binario) |
| Implicaciones | La skill documenta tools y commands para integrar VantaDB como memoria de agentes → tools/commands falsos propagan configs rotas (p.ej. OpenCode/Cursor). Docs-only: riesgo bajo |

## Hallazgos verificados (lead, 2026-08-17)

1. **🔴 Command roto**: SKILL.md:27 `vanta-server --mcp --path ~/.vantadb`:
   - Binario real: `vanta-cli server --mcp --db <path>` (verificado: `target/debug/vanta-cli.exe server --help` → `--db`, `--mcp`) o `vantadb-server --mcp` (solo `-h/--help/--mcp`; `--path` NO existe — `vantadb-server/src/main.rs:89-91`).
   - `vanta-server` (sin `db`) no es binario del workspace (bins reales: `vanta-cli`, `vantadb-server`, `crash_helper`, `lock_helper`).
   - `--path` NO es flag válido en ninguno de los dos binarios → cualquier agente que copie la skill falla.
2. **🔴 Tool inexistente**: SKILL.md:103-105 `query_lisp` → **renombrado a `query_iql`** (AUD-004; `vantadb-mcp/src/handlers/tools.rs:72`). LISP NO es soportado (solo IQL).
3. **🟡 Tools faltantes** (14 reales en `tools.rs`, la skill documenta ~10):
   - Faltan: `collection_stats`, `collection_list`, `collection_delete`, `rehydrate`.
4. **🟡 Paths muertos** (Test-Path verificado):
   - `docs/EDITOR_INTEGRATIONS.md` → NO existe (skill:178).
   - `docs/MCP.md` → NO existe (skill:223) → real `docs/api/MCP.md`.
5. **🟡 referencias internas**: `references/mcp-protocol.md`, `api-reference.md`, `configuration.md` — verificar contra código real (api-reference.md documenta `VantaEmbedded`, `VantaMemoryInput` — verificar con `src/sdk/api.rs`; probablemente API real difiere).
6. **🟡 Falta OpenCode**: el usuario quiere VantaDB con OpenCode — la skill lista "OpenCode" como editor soportado (skill:183) pero NO hay bloque de config OpenCode ni asset. `docs/api/MCP.md:185-199` SÍ tiene el bloque OpenCode correcto (con `vanta-cli server --mcp`).

## Contrato
"`skills/vantadb-mcp/SKILL.md` + `references/*.md` sin tools ni commands falsos: (1) `rg "query_lisp|vanta-server|--path" skills/vantadb-mcp/` → 0 matches (excepto notas históricas explícitas); (2) los 14 tools reales documentados (incluye `query_iql`, `collection_stats`, `collection_list`, `collection_delete`, `rehydrate`); (3) command correcto `vanta-cli server --mcp --db <path>` (o `vantadb-server --mcp`); (4) paths vivos (`docs/api/MCP.md`, `docs/EDITOR_INTEGRATIONS.md` eliminado o reemplazado por path real); (5) bloque OpenCode con config real (copiar de `docs/api/MCP.md:185-199`, ajustando al proyecto)." Verificación mecánica: 5 checks.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **NO editar `docs/api/MCP.md`** — es la fuente de verdad del contrato MCP; la skill debe ser consistente CON ella, no duplicarla ni contradecirla.
  2. NO inventar tools: la lista real está en `vantadb-mcp/src/handlers/tools.rs` (bloque `"name": "..."`). Si un tool no está en el código, no se documenta.
  3. NO tocar `docs/Backlog.md`, task files, `docs/plans/*` — solo `skills/vantadb-mcp/` (SKILL.md + references).
  4. Los scripts (`scripts/*.py`, `scripts/*.sh`) y assets (`assets/*.json`) son de SKL-03 — NO tocarlos aquí (evitar colisión de waves paralelas W1/W2).
  5. Idioma: skill en inglés — mantener.
- **Comandos de verificación:** los del contrato + `rg -c '"name": "' vantadb-mcp/src/handlers/tools.rs` (14) para cross-check.
- **Deuda pendiente:** ninguna esperada.

## Steps (Plan → Act → Verify)

1. **📝 DISCOVERY** — leer `skills/vantadb-mcp/SKILL.md` completo (223L) + `vantadb-mcp/src/handlers/tools.rs` (lista real de tools, líneas de `"name":`) + `docs/api/MCP.md` (contrato) + `references/*.md` (3 archivos). Confirmar hallazgos con rg. Verify: lista de correcciones con `archivo:línea` objetivo.
2. **📝 EJECUCIÓN** — reescribir `skills/vantadb-mcp/SKILL.md` + corregir `references/*.md`:
   - Command real: `vanta-cli server --mcp --db <path>` como comando canónico (y nota de `vantadb-server --mcp` alternativo con env `VANTADB_STORAGE_PATH`).
   - Tools: los 14 reales, agrupados (Memory CRUD, Search, Graph con `query_iql`, Collection ops, `rehydrate`).
   - Paths: `docs/api/MCP.md`, eliminar `docs/EDITOR_INTEGRATIONS.md` o enlazar a su reemplazo real si existe.
   - Bloque OpenCode: config real copiada/adaptada de `docs/api/MCP.md:185-199`.
   - references: verificar `api-reference.md` contra `src/sdk/api.rs` real (VantaEmbedded? VantaMemoryInput? — usar nombres reales o eliminar secciones inventadas); `configuration.md` contra `src/config.rs` real (env vars `VANTADB_*`); `mcp-protocol.md` contra el protocolo real.
   - Verify: contrato mecánico (5 checks).
3. **📝 CIERRE** — task file actualizado (este archivo) con hallazgos/resultado + bloque RESULTADO (pipeline-full.md §7) para el lead.

## Dependencias
- Ninguna (autónoma). SKL-03 (scripts/assets) corre en paralelo — NO solapar archivos.
- SKL-04 (review) depende de SKL-02.

## Fases explícitas — SECURITY | PERFORMANCE

- [ ] **SECURITY** — NO aplica: docs-only.
- [ ] **PERFORMANCE** — NO aplica: docs-only.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-review (SKL-04, wave W3).
- **Enfoque:** validar 14 tools reales documentados, 0 matches `query_lisp|vanta-server|--path`, command correcto, bloque OpenCode real.
- **Veredicto:** pendiente.

## Notas
- La fuente de verdad de tools es `vantadb-mcp/src/handlers/tools.rs` — la skill documenta tools, el crate los implementa. Si hay drift entre skill y crate, manda el crate.
- El usuario usa OpenCode → el bloque OpenCode es prioridad (no solo Cursor/Claude Desktop).