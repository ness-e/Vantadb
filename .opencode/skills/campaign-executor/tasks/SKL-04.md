# SKL-04: Gate P2-01 — revisión read-only de SKL-01/02/03

## Metadata
- **Plan file:** `docs/plans/2026-08-17-skills-vantadb.md` (wave SKL, W3)
- **Fuente:** veredicto de review requerido tras SKL-01/02/03 (Regla P2-01: review por agente DISTINTO al implementador)
- **Esfuerzo:** 🟢
- **Prioridad:** 🔴
- **Tipo:** Review read-only — NO implementa
- **Turns estimados:** 1-2
- **Creado:** 2026-08-17
- **Estado:** ✅ COMPLETED (2026-08-17 — veredicto CHANGES-REQUIRED con 1 falla; fix aplicado por SKL-02 y re-verificado: test-mcp.py 4/4 exit 0, "Found 2 resources". Wave SKL cerrada)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 2 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `skills/vantadb/SKILL.md` (SKL-01), `skills/vantadb-mcp/SKILL.md` + `references/*.md` (SKL-02), `skills/vantadb-mcp/scripts/*` + `assets/*.json` (SKL-03) |
| Callees | contrato mecánico de cada task file (SKL-01: 4 checks, SKL-02: 5 checks, SKL-03: 5 checks + ejecución test-mcp.py) |
| Implicaciones | El veredicto habilita el cierre de la wave. Review-only: sin cambios de código |

## Contrato
"Veredicto emitido por agente distinto (vanta-review) en este task file: APPROVE o CHANGES-REQUIRED con evidencia (check que falla + archivo:línea). Verificación mecánica: ejecutar los checks de contrato de SKL-01/02/03 contra el estado final del working tree."

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **NO implementar fixes** — solo verificar y emitir veredicto. Si se encuentran fallas, se reportan al lead (que delega de vuelta al implementador original).
  2. NO tocar `skills/`, `docs/`, `src/` — review read-only (puede escribir SOLO este task file con el veredicto).
  3. Verificar contra el CÓDIGO real, no contra lo que dice la skill: tools de `vantadb-mcp/src/handlers/tools.rs`, flags de `vantadb-server/src/main.rs`, firmas de `vantadb_py.pyi`.
- **Comandos de verificación:** los checks de contrato de SKL-01/02/03 (rg + Test-Path + ejecutar test-mcp.py).

## Steps (Plan → Act → Verify)

1. **📝 VERIFICAR** — cuando SKL-01/02/03 estén completos, ejecutar los checks de contrato de los 3 task files contra el estado final:
   - SKL-01: `rg "0.1.4|1.70|3\.8\+|docs/BENCHMARKS|docs/adr|packages/langchain|langchain_rag" skills/vantadb/SKILL.md` → 0 matches; versiones 0.5.0/1.94.1/≥3.11; installer real.
   - SKL-02: `rg "query_lisp|vanta-server|--path" skills/vantadb-mcp/` → 0 matches; 14 tools reales; command correcto; OpenCode block.
   - SKL-03: `rg "query_lisp|--path|vanta-server|VANTADB_PATH" skills/vantadb-mcp/scripts skills/vantadb-mcp/assets` → 0 matches; `python skills/vantadb-mcp/scripts/test-mcp.py` exit 0; asset OpenCode presente.
   - Cross-check de API: comparar tools documentados vs `rg -c '"name": "' vantadb-mcp/src/handlers/tools.rs` (14).
2. **📝 VEREDICTO** — escribir en este task file: APPROVE / CHANGES-REQUIRED (con evidencia `archivo:línea` por cada falla). Devolver al lead.

## Dependencias
- SKL-01, SKL-02, SKL-03 (todos ✅ antes de empezar).

## Fases explícitas — SECURITY | PERFORMANCE

- [ ] **SECURITY** — NO aplica: review de docs/scripts.
- [ ] **PERFORMANCE** — NO aplica: review.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-review (este agente, contexto fresco — no participó en SKL-01/02/03).
- **Enfoque:** ejecutar los contratos mecánicos de SKL-01/02/03 contra el working tree + coherencia skill↔código real (tools.rs, resources.rs, vantadb_py.pyi, docs/api/MCP.md). Read-only; no se implementó nada.
- **Cómo se probó:** `rg` de los patrones de contrato (SKL-01/02/03), `Test-Path` de archivos, comparación tool-a-tool contra `vantadb-mcp/src/handlers/tools.rs` (Compare-Object), spot-check de firmas contra `vantadb_py.pyi`, ejecución real de `python skills/vantadb-mcp/scripts/test-mcp.py "target\debug\vanta-cli.exe"`, diff del asset OpenCode vs `docs/api/MCP.md`.
- **Veredicto:** ❌ **CHANGES-REQUIRED** — 1 hallazgo de coherencia doc↔código (recuento de resources). Todos los checks de contrato enumerados pasan.

### Evidencia por check

**SKL-01 (vantadb SKILL.md + install-vantadb.sh):**
- ✅ `rg "0.1.4|1\.70|3\.8\+|docs/BENCHMARKS|docs/adr|packages/langchain|langchain_rag" skills/vantadb/SKILL.md` → 0 matches (EXIT False). Referencias correctas a `docs/operations/BENCHMARKS.md` y `docs/architecture/adr/` (SKILL.md:633, 635).
- ✅ `skills/vantadb/scripts/install-vantadb.sh` = 2109 bytes > 30; sin "test" (HAS_TEST: no).
- ✅ Versiones: `vantadb = "0.5.0"` (installer:49), Python >= 3.11 enforced (`PY_MIN_MAJOR=3`, `PY_MIN_MINOR=11`, installer:9-10,32-33), Rust min 1.94.1 en SKILL.md:775 (installer instala wheel pre-built, no necesita rustc).

**SKL-02 (vantadb-mcp SKILL.md + references):**
- ✅ `rg "query_lisp|vanta-server|--path"` → 2 matches, ambos **notas explícitas correctivas** permitidas por contrato: SKILL.md:30 (aplica VANTADB_STORAGE_PATH) y SKILL.md:36 (`vanta-server` no es binario real, usar `vanta-cli server --mcp --db`). Cero en `references/`.
- ✅ **15 tools reales**: `rg -c '"name": "' vantadb-mcp/src/handlers/tools.rs` = 15; SKILL.md documenta exactamente el mismo set (Compare-Object: IDENTICAL 15/15).
- ✅ Command correcto: `vanta-cli server --mcp --db ~/.vantadb` (SKILL.md:27, 47, 67, 246).
- ✅ `docs/api/MCP.md` enlazado (SKILL.md:220, 265); `docs/MCP.md` NO existe (Test-Path False).
- ✅ Bloque OpenCode presente (SKILL.md:220, 225).

**SKL-03 (scripts + assets):**
- ✅ `rg "query_lisp|--path|vanta-server|VANTADB_PATH" skills/vantadb-mcp/scripts skills/vantadb-mcp/assets` → 0 matches (EXIT False).
- ✅ `python skills/vantadb-mcp/scripts/test-mcp.py "target\debug\vanta-cli.exe"` → **4/4 passed, EXIT 0** (ejecutado por este revisor; server reporta vantadb 0.5.0, 15 tools, 4 prompts).
- ✅ `assets/opencode-config.json` existe y su contenido **coincide exactamente** con la sección OpenCode de `docs/api/MCP.md:185-199` (type local, command array `["vanta-cli","server","--mcp","--db","~/.vantadb"]`, enabled true).
- ✅ `setup-vantadb.sh`: `VANTADB_VERSION="0.5.0"` (setup-vantadb.sh:8), sin refs a config.json (EXIT False).
- ✅ `create-namespace.py` y `config-template.json` eliminados (Test-Path False); `assets/claude-desktop-config.json` y `cursor-config.json` presentes con command correcto.

**Coherencia skill↔código:**
- ✅ Tools documentados == tools en `vantadb-mcp/src/handlers/tools.rs` (15/15, IDENTICAL).
- ✅ Firmas Python spot-check (5/5) en `vantadb-python/vantadb_py/vantadb_py.pyi`:
  - `rebuild_index()` → pyi:90 `def rebuild_index(self) -> dict`
  - `search_batch(queries, top_k=5)` → pyi:104 `def search_batch(self, vectors: list[Any], top_k: int = 10)`
  - `export_namespace("./backup.jsonl", "agent/session-1")` → pyi:91 `def export_namespace(self, path: str, namespace: str) -> dict`
  - `put_batch(entries=...)` y `put_batch(namespace=..., keys=..., ...)` → pyi:48-58 (ambas formas soportadas)
  - `purge_expired()` → pyi:110 `def purge_expired(self) -> int`

### Hallazgos

- **🔴 Bloqueante:** `skills/vantadb-mcp/SKILL.md:8` — declara *"exposes 15 tools, **4 resources**, and 4 prompt templates"* pero el server real devuelve **2 resources** en `resources/list`: `vantadb-mcp/src/handlers/resources.rs:13-30` (solo `metrics://` y `schema://`) y el test en vivo reporta `✅ Found 2 resources`. Los URIs `memory://` y `namespace://` SÍ se sirven en `resources/read` (resources.rs:55, 85) pero no se anuncian en `/list`, por lo que el recuento "4 resources" del skill no coincide con el contrato MCP observable. Fix sugerido (doc, no código): ajustar SKILL.md:8 y la sección "Available MCP Resources" (SKILL.md:161-166) a "2 resources listados + 2 URIs dinámicos servibles" — o agregar los URIs dinámicos a `resources/list` (fuera de scope de la wave de docs). Delegar a SKL-02.
- **🟡 Mejora:** `skills/vantadb/SKILL.md:181` cita `docs/operations/BENCHMARKS.md §6` para `search_batch` (4.01x) — verificado que el path existe y es el correcto; sin acción requerida.
- **🟢 Nota:** el contrato del task file SKL-04 (línea 38) decía "14 tools" pero el código real tiene 15; la documentación fue corregida a 15, así que el contrato del task file quedó desactualizado (no es falla del implementador).

### Alternativas evaluadas (brainstorm)

- Corregir el recuento en SKILL.md (doc, 1 línea, lazy) vs agregar URIs dinámicos a `resources/list` (código MCP, cambia el contrato del server). Se recomienda la primera: la wave es de docs y los URIs dinámicos ya son funcionales vía `resources/read`.

### Recomendaciones (iterate)

1. Delegar a SKL-02 un fix de 1 línea: SKILL.md:8 — alinear "4 resources" con el observable real (2 listados + 2 servibles).
2. Re-ejecutar test-mcp.py tras el fix (debe seguir 4/4).
3. Si el lead prefiere mantener "4 resources", la alternativa válida es documentar explícitamente que `memory://` y `namespace://` son recursos dinámicos no listados.

---

## RESULTADO

**❌ CHANGES-REQUIRED**

Checks verificados (todos con evidencia):
1. ✅ SKL-01 rg patrones obsoletos → 0 matches (`skills/vantadb/SKILL.md`)
2. ✅ SKL-01 installer: 2109 bytes, sin "test", versiones 0.5.0/3.11 (installer) + 1.94.1 (SKILL.md:775)
3. ✅ SKL-02 rg stale flags → solo 2 notas explícitas correctivas (SKILL.md:30, 36), 0 en references/
4. ✅ SKL-02 15 tools doc == 15 tools `tools.rs` (Compare-Object IDENTICAL)
5. ✅ SKL-02 command `vanta-cli server --mcp --db` (SKILL.md:27, 47, 67, 246)
6. ✅ SKL-02 `docs/api/MCP.md` enlazado; `docs/MCP.md` no existe
7. ✅ SKL-03 rg scripts/assets → 0 matches
8. ✅ SKL-03 `test-mcp.py` → 4/4 passed, EXIT 0 (ejecutado por este revisor)
9. ✅ SKL-03 `assets/opencode-config.json` == sección OpenCode `docs/api/MCP.md:185-199` (coincidencia exacta)
10. ✅ SKL-03 archivos eliminados (create-namespace.py, config-template.json) y assets corregidos presentes
11. ✅ Coherencia firmas Python 5/5 (`vantadb_py.pyi`:48, 90, 91, 104, 110)

Fallas (1, con archivo:línea):
- 🔴 `skills/vantadb-mcp/SKILL.md:8` — recuento "4 resources" no coincide con el server real (`vantadb-mcp/src/handlers/resources.rs:13-30` devuelve 2 en `resources/list`; test-mcp.py reporta "Found 2 resources"). Fix de 1 línea en el skill; delegar a SKL-02.

## Notas
- Este task file lo ejecuta vanta-review (leaf node: nunca implementa). El lead le pasa los 3 RESULTADOS y los archivos finales.
- Si todo pasa: wave SKL cierra, plan → archive, filas Backlog migradas por el lead.