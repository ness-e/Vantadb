---
title: "FIND-177: Bins de `target/` no reemplazables con MCP vivos (Windows) — entrada de troubleshooting + scoping del gate"
kind: task
status: completed
description: "Acceso denegado (os error 5) / LNK1104 when relinking target/debug/*.exe with a live MCP server: documented the holder-by-path diagnosis, the exact Windows commands, and the -p vantadb --lib --features server workaround"
tags: [troubleshooting, windows, build-gate]
---

# FIND-177: Bins de `target/` no reemplazables con MCP vivos (Windows) — entrada de troubleshooting + scoping del gate

## Metadata
- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) §5, Fase 1, wave 1.2
- **Fuente:** Backlog `FIND-177` (origen: WIRE-04 discovery, 2026-09-27; verificado 2026-10-18)
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟡 Media
- **Tipo:** Docs
- **Turns estimados:** 5-10
- **Creado:** 2026-11-24T00:00
- **last-synced:** 2026-11-24T00:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Ninguno en código: `docs/dev/references/troubleshooting.md` es un documento nuevo. Lo consume un humano/agente que se choca con `os error 5`; `.opencode/AGENTS.md` (tabla Reference Files) ya apunta a esa ruta exacta desde antes de que existiera |
| Callees | `docs/user/troubleshooting.md` (hermano user-facing, enlazado), `../tasks/closed/WIRE-04.md` (workaround `--lib`), `vanta-mcp-local.ps1` (nombrado en prosa, no enlazado), `dev-tools/verify.ps1` (citado en prosa/líneas, **no editado**) |
| Implicaciones | 0 cambios de código; 0 cambios de contrato público. Riesgo real: una entrada mal escrita manda a matar el proceso equivocado → el documento distingue por **`Path` del proceso**, no por nombre, y prohíbe matar `opencode.exe`. No cambia el gate: `dev-tools/verify.ps1` es de `FIND-192` (solo se propone el cambio, §Deuda) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `docs/user/troubleshooting.md` (245L), `dev-tools/verify.ps1` (116L), `vanta-mcp-local.ps1` (150L aprox.), `scripts/docs/check-docs.mjs` (273L), `scripts/docs/lib.mjs` (`deriveKind`, `isIndexCandidate`), `.github/markdownlint-cli2.yaml`, `.opencode/task-system/prompts/task.md` (398L), `.opencode/task-system/prompts/pipeline-full.md` (317L), plan §5 y §3 (rutas/ownership)
- **Archivos referenciados hacia dentro:** ninguno (documento nuevo, sin includes)
- **Archivos que referencian a los editados:** `rg -n "dev/references"` → `.opencode/AGENTS.md` (Reference Files: `docs/dev/references/troubleshooting.md`), `docs/dev/Backlog.md:194`, `docs/dev/plans/backlog-release-v0.10.0.md:52`, `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md:160`. Los cuatro esperaban la ruta **y no existía** (`git ls-files docs/dev/references` = vacío): el hueco lo dejó la consolidación de docs de 2026-09-28
- **Veredicto impacto:** bajo en código (0 archivos de código), **medio en operación**: es la primera parada de cualquier sesión Windows que se choque con `os error 5`, así que el texto tiene que ser exacto o desinforma. Nada se rompe si el documento desaparece; lo que se pierde es el desbloqueo

## Contrato

```
rg -n 'os error 5|LNK1104|Acceso denegado' docs/dev/references/troubleshooting.md   # >=1 entrada nueva
rg -n 'vantadb-server.exe --mcp' docs/dev/references/troubleshooting.md             # >=1 (cómo detectarlo)
```
+ tabla de secciones al inicio, síntoma / causa raíz / solución / comando exacto, y scoping `--lib` documentado.

**Verificación obtenida:** ambos `rg` devuelven coincidencias (6 y 1 respectivamente) · `node scripts/docs/check-docs.mjs` → `GATING: all clear` · `node scripts/docs/check-links.mjs` → `broken markdown links: 0` · `npx markdownlint-cli2 --config .github/markdownlint-cli2.yaml "docs/**/*.md"` → 0 issues.

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | ¿Dónde vive la entrada? | (A) crear `docs/dev/references/troubleshooting.md` — ruta que piden plan, Backlog, contrato y `AGENTS.md`, inexistente desde la consolidación · (B) appendear a `docs/user/troubleshooting.md` — fichero vivo, indexado y en `llms.txt`, `kind: howto` correcto, pero el contrato y 4 documentos apuntan a la otra ruta | A | ✅ decidido por evidencia: 4 documentos citan la ruta A y ninguno propone mover la entrada a B; B queda como candidato de consolidación (§Deuda D1) |
| 2 | Idioma de la entrada | inglés (regla AGENTS.md: docs técnicas en inglés) / español (hermano `docs/user/troubleshooting.md` es español) | inglés | ✅ decidido por evidencia: `AGENTS.md` §Doc Language Split; los literales de síntoma (`Acceso denegado (os error 5)`, `LNK1104`) se conservan **verbatim** porque son lo que se grepea |
| 3 | ¿Se aplica el scoping `--lib` al gate? | (A) editar `dev-tools/verify.ps1` — **prohibido**: pertenece a `FIND-192` (plan §3, tabla de ownership) · (B) documentar el comando `-p vantadb --lib --features server` y proponer el probe a `FIND-192` (§Deuda D2) · (C) añadir `--lib` al step `nextest` de `verify.ps1` | B | ✅ decidido por evidencia: `verify.ps1:87` ya es `-p vantadb` y **no** construye `vantadb-server`, así que añadir `--lib` ahí solo debilitaría el fast gate (perdería los integration tests de `vantadb/tests/`) sin desbloquear nada |
| 4 | `kind` del frontmatter | `howto` (semánticamente correcto, **falla** `check-docs`: `deriveKind` de `docs/dev/**` = `research`) / `research` (pasa el gate) | `research` | ✅ decidido por evidencia: `scripts/docs/lib.mjs:284` (`/dev/` → `research`); `kind` se deriva de la ruta, no se elige |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. Nunca indicar matar un proceso por nombre: `Get-Process opencode` siempre devuelve el cliente de la sesión en curso. El documento identifica al holder por `Path ==` el `.exe` bloqueado.
  2. Un MCP que corre desde `%TEMP%\vanta-mcp-bin\` **no** bloquea `target/debug` — matarlo no arregla nada y pierde la sesión.
  3. El scoping documentado es `-p vantadb --lib --features server` (WIRE-04), no `--workspace --lib` ni `-p vantadb-server --lib`.
  4. `dev-tools/verify.ps1` no se edita en esta tarea.
- **Comandos de verificación:** los dos `rg` del contrato + `node scripts/docs/check-docs.mjs` (exit 0) + `node scripts/docs/check-links.mjs` (0 broken) + `markdownlint-cli2 --config .github/markdownlint-cli2.yaml "docs/**/*.md"` (0 issues).
- **Deuda pendiente:** ver §Deuda (D1-D4).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** cero — no hay código nuevo; se paga deuda documental existente (la ruta estaba referenciada por 4 documentos y no existía).

### §Deuda — propuestas que NO se aplican acá (ownership ajeno)

**D1 — dos documentos de troubleshooting (`docs/dev/references/` vs `docs/user/`).**
`docs/user/troubleshooting.md` está indexado en `docs/index.md:758` y en `llms.txt:183`; la ruta `docs/dev/references/troubleshooting.md` estaba citada por `AGENTS.md`, Backlog y 2 planes pero no existía. Hoy conviven. Único dueño natural: el orquestador decide si la entrada migra a `docs/user/` (donde `kind: howto` es correcto) y `AGENTS.md` fija la ruta. Coste: 15 min. No lo resuelvo acá porque `AGENTS.md` y `docs/user/**` están fuera del scope de escritura declarado.

**D2 — probe advisory en `dev-tools/verify.ps1` (para `FIND-192`, NO aplicado).**
Ubicación: **después de la línea 9** (`Set-Location $ProjectRoot`), antes del bootstrap MSVC. Cambio propuesto (5 líneas, no bloqueante):

```powershell
# WIN-LOCK (FIND-177): un proceso lanzado desde target/debug mantiene el .exe mapeado
# y el relink falla deterministico con `Acceso denegado (os error 5)`. Aviso, no bloqueo:
# ver docs/dev/references/troubleshooting.md
$winLock = Get-Process -Name vantadb-server, vanta-cli -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -like "$ProjectRoot\target\debug\*" }
if ($winLock) {
    Write-Host "  WIN-LOCK: $($winLock.Count) proceso(s) desde target/debug bloquean el relink (os error 5) - ver docs/dev/references/troubleshooting.md" -ForegroundColor DarkYellow
}
```

**NO** añadir `--lib` a la línea 87 (`cargo nextest ... -p vantadb ...`): ese step ya está scoped a `vantadb` (nunca construye `vantadb-server`) y `--lib` eliminaría los integration tests del fast gate. El único step de `verify.ps1` expuesto es `cli-probes` (línea 106, `cargo run -p vantadb --bin vanta-cli`), y solo si alguien dejó un `vanta-cli.exe` corriendo desde `target/debug`.

**D3 — `docs/index.md` y `llms.txt` ya están stale en HEAD** (730 vs 731 documentos; el conteo actual incluye `docs/dev/tasks/F0-2-binary-repin.md`, untracked de otro carril). `gen-index.mjs --check` es un step de CI (`.github/workflows/docs-gates.yml:248`). No regeneré: hornearía el fichero de otro carril en mi commit. Acción del orquestador: correr `node scripts/docs/gen-index.mjs --write` en un commit de índice único al cerrar la wave.

**D4 — el documento es huérfano para `check-docs`** (0 inbound markdown links; report-only, no gating). Se resuelve solo si D1 mueve la entrada a `docs/user/` y la enlaza desde su índice. Mientras tanto lo alcanza `AGENTS.md` (fuera de `docs/`, no cuenta para el grafo).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | 2 `rg` del contrato ✅ · `check-docs` all clear ✅ · `check-links` 0 broken ✅ · markdownlint 0 issues ✅ |
| **Commit** | Atómico (2 archivos), `docs(troubleshooting): ...`, sin `--no-verify`, sin `push` (Regla 7) ✅ |
| **Release** | N/A: tarea docs sin impacto de versión (justificado en Notas) |

## Herramientas necesarias
- `rg`, Read/write, `node scripts/docs/*.mjs`, `npx markdownlint-cli2`
- **No** `dev-tools/verify.ps1`: el carril Rust está ocupado en esta wave (instrucción de la tarea)

**Skills cargadas (SDP):** `documentation-skill` (obligatoria: crea un `.md` bajo `docs/` — frontmatter, `kind`→ruta, links, DoD) · `documentation-and-adrs` (SDP keyword-mapped: doc de *porqué*, no de *qué*) · `writing-guidelines` (SDP base: voz y estructura) · `ponytail` (activo). SDP devolvió además `campaign-executor`, `progreso`, `writing-plans`, `incremental-implementation`, `test-driven-development`, `context-engineering`, `source-driven-development` — descartadas por no aplicar a un cambio de 2 archivos sin código (`SDP: documentation-skill, documentation-and-adrs, writing-guidelines`).

## Investigation Notes

- **Mecanismo confirmado en código, no supuesto:** `vanta-mcp-local.ps1` resuelve `target/debug/vanta-cli.exe` y copia **ambos** binarios (`vanta-cli.exe` + el sibling `vantadb-server.exe`) a `%TEMP%\vanta-mcp-bin\` en el bloque `WIN-LOCK` (`a431ae0a`); luego corre `& $VantaCli server --mcp`. Es decir: con el launcher, `target/debug` **no** se bloquea. El lock aparece cuando el MCP se arranca *in place* o cuando la copia falla (`Copy-Item -ErrorAction SilentlyContinue`).
- **`vantadb-server` es binario propio:** `vantadb-server/Cargo.toml:10-11` (`[[bin]] name = "vantadb-server"`), y el server lo lanza `vanta-cli` desde un candidato *hermano* — por eso el launcher tiene que copiar los dos.
- **Estado del entorno durante esta tarea (solo lectura, no se mató nada):** `Get-Process vantadb-server,vanta-cli` → vacío; solo `opencode.exe` (×2). No había lock activo, así que no se reprodujo: el diagnóstico está tomado de la fila del Backlog (reproducido 2×) y del código, y el documento no afirma que el lock esté ocurriendo.
- **`verify.ps1` no cae por este motivo:** línea 87 es `-p vantadb` (nunca construye `vantadb-server`); línea 81/82 (`check`/`clippy -p vantadb`) no enlazan binario. El que cae es el gate de cierre `cargo nextest run --profile audit --workspace` (pipeline-full §Cierre) y cualquier `-p vantadb-server`.
- **Workaround WIRE-04 verificado en el task file cerrado:** `cargo nextest run --profile audit -p vantadb --lib --features server -E 'test(ttl)'` → 21/21 ✅, con "E2E HTTP movido al lib test target por FIND-177" (`docs/dev/tasks/closed/WIRE-04.md:86,99`).
- **Sin web research:** el comportamiento (Windows no permite borrar la imagen de un proceso vivo) es del SO, no de una API externa; no aplica el gate de citas.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — resueltas: mecanismo del lock (código del launcher), ubicación de la entrada (4 referencias), alcance sobre `verify.ps1` (líneas 87/106 leídas) |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no aplica: documento de texto, sin input, sin auth, sin dependencias. Riesgo de seguridad real del contenido: **no** incluye comandos destructivos genéricos (`Stop-Process -Name opencode`, `taskkill /IM`), que matarían la sesión del agente; solo `Stop-Process` filtrado por `Path`.
- [x] **PERFORMANCE** — no aplica: sin hot path. El documento *desbloquea* builds (2h de fricción por sesión Windows) pero no mide ni cambia rendimiento.

## Steps

### Step 1: Crear la entrada de troubleshooting
- **Archivos:** `docs/dev/references/troubleshooting.md`
- **Acción:** frontmatter (`kind: research` — derivado de la ruta), tabla de secciones al inicio, entrada única con síntoma (`Acceso denegado (os error 5)` / `LNK1104`) / causa raíz (launcher `WIN-LOCK`, MCP in-place) / solución (identificar por `Path`, `Stop-Process` filtrado) / comandos exactos de detección y kill en Windows / tabla "dónde cae el gate" / workaround `--lib` con su alcance real
- **Verify:** los 2 `rg` del contrato
- **Estado:** ✅ DONE

### Step 2: Verificar gates de docs
- **Archivos:** —
- **Acción:** `check-docs` (kind/path/frontmatter), `check-links` (0 broken), `markdownlint-cli2` con la config del repo
- **Verify:** `node scripts/docs/check-docs.mjs` → `GATING: all clear`; `check-links` → `0`
- **Estado:** ✅ DONE

### Step 3: Review P2-01 (degradado, marcado)
- **Archivos:** —
- **Acción:** tier **fast** (paths `docs/dev/**`); sin `task` tool disponible en este sub-agente → review en contexto propio, declarado degradado, + las 11 comprobaciones del checklist anti-hábitos sobre el diff
- **Verify:** veredicto registrado en §Review
- **Estado:** ✅ DONE (degradado)

### Step 4: Commit local
- **Archivos:** los 2 de la tarea
- **Acción:** `git add docs/dev/references/troubleshooting.md docs/dev/tasks/FIND-177.md && git commit -m "docs(troubleshooting): document the Windows binary lock and the --lib workaround (FIND-177)"`
- **Verify:** `git show --stat HEAD`
- **Estado:** ✅ DONE

## Dependencias
- Ninguna. Wave 1.2, carril Docs. En paralelo: `FIND-164` (`.opencode/agents` + `task-system/RULES.md`), `FIND-328` (CI), `F0.2` (build). Rutas disjuntas.
- **No** tocar `dev-tools/**` (`FIND-192`), `src/**`, `.github/**`, `docs/dev/Backlog.md`, `docs/dev/plans/**`, `docs/pipeline-state.json`, `skills/**`.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-docs` self-review **degradado** (sin `task` tool en este sub-agente; no se pudo spawnear `vanta-review`/`vanta-audit`)
- **Tier:** fast (`docs/dev/**` → 0 paths adversariales; el diff no toca `docs/api/`, `src/`, ni formatos wire)
- **Enfoque:** ¿la entrada lleva a la acción correcta? Sí: el discriminante es `Path` del proceso, no el nombre; incluye la fila "`%TEMP%\vanta-mcp-bin\` → **leave it**" que es el error más probable de un lector apurado. ¿Alternativas mejores? Appendear a `docs/user/troubleshooting.md` era más barato (1 fichero, `kind` correcto, ya indexado) pero contradice 4 documentos que citan la ruta del plan → registrado como D1 en vez de improvisar.
- **Cómo se probó:** mecánico, no auto-reporte: 2 `rg` del contrato (6 + 1 hits), `check-docs` exit 0 `GATING: all clear`, `check-links` `broken: 0`, `markdownlint-cli2 --config` 0 issues. Comandos del documento revisados línea por línea contra `vanta-mcp-local.ps1` (bloque `WIN-LOCK`), `dev-tools/verify.ps1:87,106` y `vantadb-server/Cargo.toml:10-11`.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas: todas las salidas citadas son de esta sesión; el no-repro no se disfraza de repro (declarado en Investigation Notes).
  - [x] No saltarse la clarificación: sin ambigüedad de usuario (contrato mecánico dado por el orquestador).
  - [x] No declarar done sin verificar contra el contrato.
  - [x] No ignorar fallos: `gen-index --check` queda rojo por causa pre-existente y ajena → D3, no silenciado.
  - [x] No single-shot búsqueda: 3 rutas candidatas para el doc, 4 referencias cruzadas, 3 gates corridos.
  - [x] No copiar sin citar: el workaround cita `WIRE-04:86`; el launcher cita el bloque `WIN-LOCK`.
  - [x] No reintentos en bucle: 0 fallos de verify.
  - [x] Pasos conectados al objetivo: 4/4.
  - [x] No degradar chequeos de seguridad: el doc **evita** comandos que maten el cliente.
  - [x] Budget explícito: cerrado en una pasada.
  - [x] Cobertura SDP: `documentation-skill` (obligatoria) + `documentation-and-adrs` + `writing-guidelines`; sin `pinned` en el output de v3.
- **Veredicto:** ✅ approve (degradado, marcado) — pendiente de confirmación por el orquestador si quiere un segundo par de ojos (tier fast = spot-check).

## Notas
- **Decisión clave:** el documento escribe el discriminante correcto (`Path`), no la heurística cómoda (`nombre`). El pre-mortem de la tarea era "recomendar matar un proceso que no es": `opencode.exe` aparece **siempre** en `Get-Process` (es el cliente), así que cualquier receta por nombre habría matado la sesión del propio agente.
- **No se regeneró `docs/index.md`/`llms.txt`** a propósito: ya están stale en HEAD por un fichero de otro carril (D3). Un commit de índice único al cerrar la wave es el lugar correcto.
- **Release N/A:** sin cambio de código ni de versión; `docs(...)` genera patch en release-plz, que es lo esperado para un doc nuevo.
