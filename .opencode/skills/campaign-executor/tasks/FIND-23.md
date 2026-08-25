# FIND-23: vanta-http-map envía namespace vacío en ingest/get

> Plan: `docs/plans/2026-08-25-batch-core-fixes-research.md` · Task 1 · Campaign `aa2cde2b-e52f-4dae-910a-b274373a5bda`
> Estado: ⬜ PENDING → ⏳ IN PROGRESS (2026-08-25)
> Contrato: ingest/get con namespace omitido usan `DEFAULT_NS` en vanta-http-map; `npm run build` (desktop) exit 0

## Objetivo

`desktop/src/vanta-http-map.ts` manda `namespace: item.namespace ?? ""` (línea 93) y
`q.namespace ?? ""` (línea 114) en ingest/search con namespace omitido → el server embebido
rechaza con "Validation error: namespace must not be empty" (`src/sdk/serialization/mod.rs:93`).
El mapping WASM (`vanta-wasm-map.ts:45`) sí defaulta `DEFAULT_NS = "default"`. Bug detectado
por E2E-VISUAL (workaround documentado en `desktop/e2e/flujo-critico.spec.ts:61`).

## Metadata

- **Plan file:** `docs/plans/2026-08-25-batch-core-fixes-research.md`
- **Fuente:** E2E-VISUAL → Backlog FIND-23 → plan Task 1
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴 (rompe ingest en web build embebido)
- **Tipo:** TypeScript (desktop mapping HTTP transport)
- **Turns estimados:** 5-10
- **Creado:** 2026-08-25
- **Estado:** ✅ COMPLETED (2026-08-25, 3/3 steps; commit pendiente del lead)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `desktop/src/transport.ts` (getHttpMapping), `desktop/src/vanta-wasm-map.ts` (importa `ingestToInput`/`searchToRequest` y hace override post-call), `desktop/src/vanta-http-map.test.ts` |
| Callees | `./vanta` (solo tipos: IngestItem, SearchQuery, MemoryRecord, etc.) |
| Implicaciones | Contrato HTTP wire NO cambia (mismo endpoint/shape); comportamiento de namespace vacío cambia de `""` → `"default"`; el path WASM es idempotente (wasm-map ya resuelve DEFAULT_NS antes/después de llamar los adapters compartidos) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `desktop/src/vanta-http-map.ts` (445L), `desktop/src/vanta-http-map.test.ts` (334L), `desktop/src/vanta-wasm-map.ts` (parcial — imports + patrón DEFAULT_NS líneas 45/82/92/99-100), `desktop/vitest.config.ts`, `desktop/package.json`, `.opencode/rules/frontend-web.md`, `.opencode/rules/js-ecosystem.md`
- **Archivos referenciados hacia dentro (imports):** `./vanta` (solo type-only imports)
- **Archivos que referencian a los editados (referencias entrantes):**
  - `desktop/src/transport.ts:8` → `getHttpMapping` (resolución de comandos HTTP)
  - `desktop/src/vanta-wasm-map.ts:27-31` → `ingestToInput`, `searchToRequest` (adapters compartidos) — **crítico: no cambiar semántica para namespace explícito**
  - `desktop/src/vanta-http-map.test.ts` (getHttpMapping, mappedCommands, unsupportedCommands)
  - `desktop/vitest.config.ts:22` → excluye el test de vitest (corre con `node --test`)
  - `desktop/e2e/flujo-critico.spec.ts:61` → comentario del workaround (NO tocar — el workaround sigue válido)
- **Veredicto impacto:** bajo. Fix interno al mapping layer: `?? ""` → `|| DEFAULT_NS`. Sin cambio de API pública, sin cambio de endpoints, sin cambio de wire shape. Para namespace explícito el comportamiento es idéntico (`"ns1"` sigue `"ns1"`). El path WASM ya hace `item.namespace || DEFAULT_NS` ANTES de llamar `ingestToInput` (wasm-map:82/92) y `q.namespace || DEFAULT_NS` DESPUÉS de `searchToRequest` (wasm-map:100) → el fix es idempotente allí (resultado ya era "default"). Venta colateral: el override post-call del wasm-map pasa a ser no-op (inofensivo).

## Contrato

"ingest/get con namespace omitido usan `DEFAULT_NS` en vanta-http-map; `npm run build` (desktop) exit 0" — verificación mecánica: `cd desktop && node --test src/vanta-http-map.test.ts` (tests del mapping, incluido el nuevo RED→GREEN) + `cd desktop && npm run build` exit 0.

## Spec (SDD — bug-fix sin símbolos públicos nuevos; decisión única resuelta por evidencia)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Valor de `DEFAULT_NS` en vanta-http-map.ts | A: `"default"` / B: otra string | A | ✅ decidido-por-evidencia: `vanta-wasm-map.ts:45` (`const DEFAULT_NS = "default"`), `desktop/src-tauri/src/connections/native.rs:34` (`DEFAULT_NAMESPACE: &str = "default"`), `types.rs:369` (`default_namespace()` → "default") — los 3 transports del proyecto usan "default"; el server valida namespace no-vacío (serialization/mod.rs:93) y "default" es el namespace canónico |
| 2 | Operador: `\|\|` vs `??` | `\|\|` defaulta null/undefined Y vacío (match wasm-map) / `??` solo null/undefined | `\|\|` | ✅ decidido-por-evidencia: wasm-map usa `\|\|`; el server rechaza "" (serialization/mod.rs:93) → vacío también debe defaultar |
| 3 | Alcance: solo líneas 93/114 vs los 6 sitios con `?? ""` | Solo 2 líneas (contrato literal) / los 6 (misma causa raíz: get/get_version/versions/delete paths también mandan "") | 6 sitios | ✅ decidido-por-evidencia: ponytail bug-fix — causa raíz única (`?? ""` en todo record path); `vanta_get` (244), `vanta_get_version` (253), `vanta_versions` (263), `vanta_delete` (290) son callers hermanos con el mismo bug → `/api/v2/records//k` también rechazado. Contrato dice "ingest/get" → get paths son parte del contrato. Fix único donde todos rutear |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) cero cambios en `desktop/src-tauri/**` (Rust bridge), `web/`, core Rust; (2) comportamiento para namespace EXPLÍCITO no cambia (tests existentes de ns explícito siguen verdes); (3) NO commit — el lead verifica mecánico y commitea por tarea (regla del batch); (4) no tocar `desktop/e2e/flujo-critico.spec.ts` (workaround sigue válido, fuera de scope).
- **Comandos de verificación:** `cd desktop && node --test src/vanta-http-map.test.ts` (20+ tests → todos pass) · `cd desktop && npm run build` (exit 0)
- **Deuda pendiente:** ninguna nueva (saldo 0 — fix sin deuda). El dedupe de `DEFAULT_NS` entre wasm-map.ts y http-map.ts (2 constantes iguales) es deuda existente, no introducida por este cambio.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | Fuente |
|------------------------|--------|
| `activeGoal` | FIND-23: vanta-http-map envía namespace vacío en ingest/get |
| `lastAction` | Discovery + task file creado (Impacto mapeado + Fase 1) |
| `result` | PARTIAL (steps pendientes) |
| `nextAction` | Step 1 RED: agregar asserts de namespace-default a vanta-http-map.test.ts y verificar que FALLAN |
| `contract` | Ver `## Contrato` + evidencia/artefactos en recitation del cierre |
| `nextTask` | AUD-044 (plan Task 2) |

## Deuda técnica (Regla 6 — MUST)

**Sin deuda** — cambio correctivo sin deuda nueva introducida.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate | Estado |
|-------|------|--------|
| Task | Contrato verificable (`node --test` mapping + `npm run build` exit 0) | ⏳ |
| Commit | El lead commitea (NO commit por worker — regla del batch) | n/a |
| Release | No aplica (desktop web build, no release crate) | n/a |

## Herramientas necesarias

- `node --test` (Node v24.16.0 — type stripping nativo, sin flags)
- `npm run build` (tsc + vite build)
- codegraph_explore / grep (blast radius — CodeGraph auto-sync deshabilitado → Read/Grep directo)

**Skills cargadas (SDP):** systematic-debugging (bug-fix Iron Law — root cause antes de fix) · test-driven-development (RED→GREEN con test de regresión) · source-driven-development (validar patrón DEFAULT_NS contra el código existente) · campaign-executor (base pipeline) · progreso (base) · ponytail (lazy full — fix mínimo). SDP: sin candidatos adicionales (grep SKILLS-MANIFEST por keywords typescript/desktop/http-map no reveló skill específica de mapping TS).

## Investigation Notes

- **Causa raíz confirmada (Phase 1 systematic-debugging):** `?? ""` en 6 sitios de vanta-http-map.ts convierte namespace omitido en string vacía; el SDK del server valida `namespace must not be empty` (`src/sdk/serialization/mod.rs:93`). Repro determinístico: llamar ingest con `{text: "x"}` sin namespace → body `namespace: ""` → 400.
- **Patrón correcto existente (Phase 2):** `vanta-wasm-map.ts:45` `const DEFAULT_NS = "default"` + `item.namespace || DEFAULT_NS`; `native.rs:34` `DEFAULT_NAMESPACE = "default"` (Rust bridge usa `unwrap_or(DEFAULT_NAMESPACE)`); `types.rs:369` serde default. Los 3 transports convergen en "default".
- **Blast radius del wasm-map:** importa los adapters compartidos y ya aplica DEFAULT_NS alrededor → el fix es idempotente; el override post-call (wasm-map:100) queda como no-op sin efecto.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 3 (Step 1 RED, Step 2 GREEN, Step 3 Verify) |
| % completado | 10% |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

- **Repro:** `node --test src/vanta-http-map.test.ts` con un assert nuevo: `getHttpMapping("vanta_ingest").body?.({records:[{text:"x"}]})[0].namespace` → actualmente `""`, debe ser `"default"`. (Equivalente server-side: POST /api/v2/records/batch con `namespace: ""` → 400 "namespace must not be empty" — validado en `src/sdk/serialization/mod.rs:93`.)
- **Hipótesis:** la causa raíz es el operador `?? ""` en los 6 sitios que construyen requests record-path de vanta-http-map.ts; el fix correcto es defaultar a `DEFAULT_NS = "default"` con `||` (defaulta también string vacía), mirror del wasm-map.
- **1 variable controlada:** exactamente una variable por intento — el valor de namespace en los bodies/paths del mapping; nada más cambia (no endpoints, no DTOs, no tests existentes).
- **Test RED:** agregar 3 asserts (ingest body, search body, get path) con namespace omitido → verificar FALLO antes del fix; GREEN solo con el fix aplicado.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — NO aplica: el cambio no toca trust boundaries, no agrega/quita dependencias, no cambia autenticación ni storage; solo normaliza un valor de request dentro del mapping (default de namespace, validado luego por el server).
- [x] **PERFORMANCE** — NO aplica: no toca hot paths del core (esto es mapping de transporte desktop, una operación `||` por request); sin baseline requerido (Regla 9 no dispara — no es optimización).

## Steps

### Step 1: RED — asserts de namespace-default en vanta-http-map.test.ts
- **Archivos:** `desktop/src/vanta-http-map.test.ts`
- **Acción:** agregar test `FIND-23: ingest/search/get con namespace omitido usan DEFAULT_NS` (3 asserts: body de ingest `[{text:"x"},{text:"y",namespace:""}]` → namespace "default"; body de search `{query:"hi"}` → namespace "default"; path de get sin namespace → `/api/v2/records/default/k1`).
- **Verify:** `cd desktop && node --test src/vanta-http-map.test.ts` → el test NUEVO FALLA (RED, 20 pass / 1 fail), los 20 existentes pasan.
- **Estado:** ✅ COMPLETED

### Step 2: GREEN — fix en vanta-http-map.ts
- **Archivos:** `desktop/src/vanta-http-map.ts`
- **Acción:** (1) agregar `const DEFAULT_NS = "default";` con comentario FIND-23; (2) `replaceAll` `(args.namespace as string) ?? ""` → `(args.namespace as string) || DEFAULT_NS` (4 sitios: get/get_version/versions/delete); (3) línea 93 `namespace: item.namespace ?? ""` → `namespace: item.namespace || DEFAULT_NS`; (4) línea 114 `namespace: q.namespace ?? ""` → `namespace: q.namespace || DEFAULT_NS`.
- **Verify:** `cd desktop && node --test src/vanta-http-map.test.ts` → 21 pass (RED→GREEN).
- **Estado:** ✅ COMPLETED

### Step 3: Verify full + cierre
- **Acción:** `cd desktop && npm run build` exit 0 (tsc + vite); `node --test src/vanta-http-map.test.ts` 21/21 y `node --test src/vanta-wasm-map.test.ts` 14/14 (idempotencia wasm path); actualizar task file con Context Save Point.
- **Verify:** `npm run build` exit 0 ✅ · ambos node --test pass ✅.
- **Estado:** ✅ COMPLETED

## Context Save Point (cierre 2026-08-25)

- **Contrato cumplido:** ingest/get con namespace omitido usan `DEFAULT_NS` en vanta-http-map; `cd desktop && npm run build` → **exit 0** (tsc + vite, 17.61s; warning chunk GraphLens pre-existente, no relacionado).
- **Tests:** `node --test src/vanta-http-map.test.ts` → **21/21 pass** (RED verificado antes del fix: 20 pass/1 fail; GREEN después). `node --test src/vanta-wasm-map.test.ts` → **14/14 pass** (idempotencia: wasm-map ya resolvía DEFAULT_NS alrededor de los adapters compartidos → sin cambio de comportamiento).
- **Archivos tocados (worker):** `desktop/src/vanta-http-map.ts` (const DEFAULT_NS + 6 sitios), `desktop/src/vanta-http-map.test.ts` (+18L test FIND-23), task file. **Cleanup WIP guard:** `FIND-11.md` + `UX-POLISH.md` estado Metadata ⏳ → ✅ (stale documentado en E2E-VISUAL; 4/4 y 13/13 steps verificablemente completos).
- **NO commit** (regla del batch) — el lead verifica mecánico y commitea SOLO: `desktop/src/vanta-http-map.ts`, `desktop/src/vanta-http-map.test.ts`, `.opencode/skills/campaign-executor/tasks/FIND-23.md`, `.opencode/skills/campaign-executor/tasks/FIND-11.md`, `.opencode/skills/campaign-executor/tasks/UX-POLISH.md` (cleanup), `docs/plans/2026-08-25-batch-core-fixes-research.md` (estado de la tarea). NO incluir archivos de agentes paralelos (AUD-047.md, src/index/search/layer.rs — trabajo de otros).
- **git status del worktree:** contiene cambios de agentes paralelos del batch (AUD-047, layer.rs) — no son de esta tarea.
- **Deuda:** ninguna nueva (saldo 0). `DEFAULT_NS` duplicado wasm-map/http-map aceptado (dedupe de 1 línea no vale tocar 2 archivos; valor "default" triple-verificado).
- **Handoff lead:** verificar `cd desktop && npm run build` + `node --test src/vanta-http-map.test.ts` mecánicamente, commitear solo los archivos listados, ejecutar skill progreso (fila FIND-23 en Backlog). Review P2-01 pendiente: vanta-review sobre el diff (6 sitios vs 2 del contrato literal — justificado en Spec #3).

## Dependencias

- Ninguna (task aislada; E2E-VISUAL ya completado y commiteado por el lead)

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review (o vanta-audit) — delegar por el lead al cierre
- **Enfoque:** ¿el fix de 6 sitios (vs solo 2 líneas del contrato literal) es correcto? ¿idempotencia con wasm-map verificada?
- **Cómo se probó:** `node --test src/vanta-http-map.test.ts` (RED verificado antes del fix, GREEN después) + `npm run build` exit 0 — evidencia mecánica, no auto-reporte
- **Checklist anti-hábitos tóxicos:** [ ] pendiente de verificación por el revisor
- **Veredicto:** ⬜ pendiente

## Notas

- No tocar el workaround del e2e spec (flujo-critico.spec.ts:61) — el comentario queda desactualizado pero el spec sigue verde; limpieza opcional en otra tarea si el lead decide.
- `DEFAULT_NS` duplicado entre wasm-map.ts y http-map.ts: aceptado (ponytail — no tocar 2 archivos para dedupe de 1 línea; valor canónico "default" ya triple-verificado).