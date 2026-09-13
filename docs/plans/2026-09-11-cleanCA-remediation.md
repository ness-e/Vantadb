# Plan: Remediación cleanCA — 2 bloqueantes + 20 deuda priorizada

> **Fecha:** 2026-09-11 · **Origen:** auditoría `/cleanCA` (sin parámetro) + evaluación vs `.opencode/references/clean-code-clean-architecture.md`
> **Estado:** PLAN (no ejecutar sin aprobación del usuario) · **Rama:** `develop` (trunk-based, PR a `main`)

## 0. Norma obligatoria (leer antes de todo)

1. `.opencode/references/clean-code-clean-architecture.md` **completo, incluyendo Apéndice V** (manda sobre ejemplos genéricos; severidades 🔴🟡🟢 en V.4).
2. `.opencode/commands/cleanCA.md` — cada tarea abre con `/cleanCA <scope>` (baseline) y cierra con `/cleanCA <scope>` (veredicto PASS).
3. `.opencode/AGENTS.md` — Ritual, Reglas 1/3/4/5/6/9/10/11, Regla 0 (impacto antes de modificar).

## 1. Pasos para CREAR un plan (aplicados aquí)

1. Partir de hallazgos registrados (`FIND-*` vía `prompts/findings.md`).
2. Agrupar por afinidad (misma familia de regla + mismos archivos) en tareas de 1–3 días.
3. Por tarea: objetivo, scope exacto, skills, MCP/herramientas, dependencias, prompt de subagente, verificación.
4. Ordenar por riesgo (🔴 primero) y dependencias (renames antes que splits que los tocan).
5. Guardar en `docs/plans/` y pedir aprobación (este archivo). Al completar → `docs/plans/archive/`.

## 2. Pasos para EJECUTAR un plan

1. Usuario aprueba el plan (o recorta alcance).
2. Por cada tarea en orden (salvo paralelas marcadas ⇄): crear task file → delegar a subagente con su prompt → revisión post-delegación del lead → verify mecánico → siguiente.
3. Regla 6: cada PR con deuda nueva paga deuda equivalente (saldo neto ≤ 0).
4. Cierre: `/cleanCA` global debe dar PASS (0 🔴); actualizar `docs/api/` en el mismo PR si cambió API pública (Regla 3); ADR si hubo tradeoff (Regla 5).

## 3. Pasos para CREAR y EJECUTAR una tarea

1. **Lookup:** verificar ID y estado en `docs/tasks/` (+ `complete/`, `closed/`).
2. **Clasificar:** `campaign_detect_task_type` + `campaign_classify_workflow` → routing (§5).
3. **Skills:** `campaign_discover_skills_v2` + skills fijas de la tarea.
4. **Task file:** crear `docs/tasks/<ID>.md` con las 4 fases de `prompts/task.md` (ver §4).
5. **Delegar:** `task(description, prompt, subagent_type)` con el prompt completo de §6 (incluye Contexto + Estado, referencia a `pipeline-full.md`: DISCOVERY → EJECUCIÓN → CIERRE, y bloque `RESULTADO` exigido).
6. **Recuperación (SARL):** ✅ COMPLETO → revisión; 🟡/❌ → RESUME (`task(task_id=<T>)`) → RETRY fresco → STRATEGY (`campaign_mom_escalate`) → ESCALATE humano.
7. **Revisión post-delegación del lead:** `codegraph_explore` de lo modificado + `campaign_verify_cmd` del contrato + `cargo check -p <crate>` o `just verify-quick`.

## 4. Pasos para la CREACIÓN del task file (`docs/tasks/<ID>.md`)

```markdown
# <ID> — <título>
## 1. Descubrimiento (auto-detect tipo → codegraph blast radius → web si ambigüedad → baseline `/cleanCA <scope>`)
## 2. Contrato (qué cambia / qué NO cambia / archivos exactos / comandos de verify)
## 3. Steps atómicos (☐ uno por slice: implementar → test → verificar → commit; si falla: `git reset --hard HEAD` del slice)
## 4. Cierre (RESULTADO + `/cleanCA <scope>` PASS + recitation)
```

## 5. Routing y skills/MCP/herramientas comunes

| Tarea | Subagente | Skills fijas | MCP/tools clave |
|---|---|---|---|
| B1, D1, D5 (lógica/frontera) | `vanta-worker` | `campaign-executor`, `systematic-debugging`, `test-driven-development` | CodeGraph, codebase-memory (`detect_changes`), `cargo nextest --profile audit`, clippy |
| B2 (unwrap, prod) | `vanta-worker` (+ `vanta-chaos` para stress del fix WAL) | `systematic-debugging`, `test-driven-development` | `rg`, clippy, nextest, miri si `unsafe` cercano |
| D2, D3 (flags/params) | `vanta-worker` | `campaign-executor` | clippy (`fn_params_excessive`), nextest |
| D4a (renames internos) | `vanta-worker` | `campaign-executor` | CodeGraph callers, `cargo check --workspace` |
| D4b (enum `Error`) | `vanta-arch` diseña + `vanta-worker` implementa | `api-design-principles`, `deprecation-and-migration` | `cargo semver-checks`, `cargo public-api` si disponible |
| Revisiones | `vanta-review` (nunca implementa) | `code-review-and-quality`, `doubt-driven-development` | `codegraph_explore`, verify scripts |

Relaciones/dependencias entre tareas: D4a antes que D1a/D1c (renames tocan firmas que los splits mueven); B2-triaje antes que B2-fix; D5c (clamp) independiente y primera (riesgo DoS); D4b última (breaking, requiere release major + ADR + `cargo semver-checks`).

## 6. Tareas con PROMPT COMPLETO personalizado

> Cada prompt se entrega tal cual al subagente. Incluye: rol, norma, contexto verificado, estado, scope exacto, prohibiciones, relaciones/dependencias, verificación y formato RESULTADO. El subagente sigue `pipeline-full.md` (DISCOVERY → EJECUCIÓN → CIERRE).

### B1 — Extraer `StartConversationUseCase` de `conversation_add` 🔴
**Scope:** `src/server/handlers.rs:1334-1356` (+ rutas que lo llaman). **Skills:** campaign-executor, systematic-debugging, test-driven-development.
**Prompt:**
```
Eres vanta-worker. Norma: .opencode/references/clean-code-clean-architecture.md + Apéndice V
(§5.4 Humble Object, §4.1 SRP/DIP) y .opencode/commands/cleanCA.md regla A2.
Contexto verificado: conversation_add orquesta create_thread + send_message + audit +
trigger dentro del handler HTTP; la mayoría de handlers delegan vía run_db_op.
Estado: task file docs/tasks/B1.md fase 1; baseline: /cleanCA src/server/handlers.rs (A2 🔴).
Haz DISCOVERY (codegraph_explore conversation_add: callers, callees, blast radius; detect_changes)
→ EJECUCIÓN por slices (1: struct StartConversationCommand + trait puerto en capa aplicación;
2: mover orquestación al caso de uso, handler queda humilde: DTO→uso→respuesta;
3: tests AAA/FIRST del caso de uso en memoria, sin HTTP) → CIERRE.
PROHIBIDO: reescribir handlers.rs entero; cambiar API HTTP; tocar trigger más allá de mover su llamada.
Relaciones: padres=rutas conversation; hijos=ThreadStore/audit/trigger; riesgo=testabilidad.
Verifica: cargo nextest --profile audit -p vantadb conversation + cargo clippy -p vantadb --deny warnings
+ /cleanCA src/server/handlers.rs (A2 debe pasar a ✅).
RESULTADO: ✅/🟡/❌ + archivos:líneas + comandos corridos + próximo step ☐ si incompleto.
```

### B2a — Triaje `unwrap/expect` en producción (base de B2) 🔴
**Scope:** `src/parser/mod.rs`, `src/physical_plan/mod.rs`, `src/wal.rs`, `src/wal_sharded.rs`, `src/engine.rs`, `src/sdk/api.rs` (+ resto de `src/`).
**Prompt:**
```
Eres vanta-worker. Norma: guía §2.3/E1 + Apéndice V.2 (clippy) + cleanCA.md regla E1.
Contexto: 1691 ocurrencias brutas (incluye #[cfg(test)] inline — debes separar prod vs test).
Estado: docs/tasks/B2a.md fase 1.
DISCOVERY: rg por archivo, clasifica cada unwrap/expect en: (a) test, (b) invariante imposible
documentable, (c) error real tragado → tabla archivo:línea + categoría + tipo de Error a usar.
NO implementes fixes (eso es B2b). PROHIBIDO tocar código prod en esta tarea.
Verifica: tu tabla cubre los 6 focos + `cargo clippy -p vantadb --deny warnings` sigue verde.
RESULTADO: tabla completa + conteo prod-real + plan de fix por archivo para B2b.
```

### B2b — Eliminar `unwrap` prod con `Result` + `?` 🔴
**Scope:** archivos categoría (c) de B2a. **Depende de:** B2a.
**Prompt:**
```
Eres vanta-worker. Norma §2.3/E1 + Regla 6 (cada fix cuenta como pago de deuda).
Contexto/Estado: tabla de B2a en docs/tasks/B2a.md; empieza por wal.rs + engine.rs.
Slices atómicos por archivo: reemplazar unwrap→Result con variante Error existente (o nueva
mínima con thiserror) + propagar ? + test que dispara el error (TDD: RED→GREEN).
PROHIBIDO: expect() nuevo; cambiar mensajes de error públicos sin nota; slices >1 archivo.
Relaciones: hijos=callers que ahora reciben Result (actualizar firmas); riesgo=cambios en api.
Verifica por slice: nextest del módulo + clippy deny warnings; al cierre /cleanCA <archivos> (E1 ✅).
RESULTADO: ✅/🟡/❌ + archivos:líneas + tests agregados + deuda pagada (Regla 6).
```

### D1a — Partir `batch_insert_with_opts` (~339) e `ingest get` (~282) 🟡
**Scope:** `src/storage/engine/insert.rs:517`, `src/storage/engine/get.rs:70`. **Skills:** + planning-and-task-breakdown.
**Prompt:**
```
Eres vanta-worker. Norma §2.2/F1 (SLAP, ≤20 líneas/función como convención operativa) + Regla 9
(benchmark before/after si tocas hot path: benches/canonical_p99.rs).
Contexto: insert batch y get mezclan validación, I/O, métricas y política en una sábana.
DISCOVERY: codegraph_explore de ambas (callers/callees) + baseline /cleanCA de cada archivo.
EJECUCIÓN por slices: extraer helpers de un solo nivel (validate→fetch→transform→persist→metrics),
cada helper ≤20 líneas, nombres intention-revealing (§2.1). PROHIBIDO cambiar semántica/orden de
efectos; PROHIBIDO optimizar (solo partir; medir con canonical_p99 antes/después para probar
neutralidad). Tests: los existentes deben seguir verdes + 1 test por helper extraído si hay lógica.
Verifica: nextest storage::engine + clippy + /cleanCA (F1 mejora medible: contar funciones >20 antes/después).
RESULTADO: antes/después (nº funciones >20) + archivos:líneas + bench neutral.
```

### D1b — Partir `vector_search` (~133) y `run_pipeline` (~132) 🟡 ⇄ con D1a
**Scope:** `src/engine.rs:338`, `src/storage/engine/maintenance.rs:1086`. Prompt análogo a D1a (mismos gates + bench).

### D1c — Adelgazar `records_list` (~129) moviendo paginación a caso de uso 🟡
**Scope:** `src/server/handlers.rs:390-492`. **Depende de:** patrón de B1 (reutilizar su puerto/estilo).
**Prompt:**
```
Eres vanta-worker. Norma §5.4/A2 + §2.2/F1. Contexto: records_list hace fan-out namespaces,
sort, merge_all_namespaces_pages y slice en el handler (B1 ya estableció el patrón humilde).
Mueve fan-out+merge+paginación a ListRecordsUseCase con DTO (namespace, cursor, limit) y deja el
handler traduciendo HTTP↔DTO. Slices: 1 caso de uso + tests en memoria; 2 handler delgado;
3 borrar código muerto del handler. PROHIBIDO cambiar el wire (next_cursor idéntico).
Verifica: tests HTTP existentes + nextest nuevo + /cleanCA (A2/F1 ✅).
RESULTADO: diff de líneas handler + archivos + tests.
```

### D2 — Flags `bool` → enums/opciones 🟡 ⇄ entre sí
**Scope:** `src/server/telemetry.rs:45` (3 flags), `src/cli_handlers/diagnostics.rs:79`, `src/cli_handlers/backup.rs:232`, `src/rbac.rs:63` (`write: bool`), `src/storage/engine/maintenance.rs:267` (`lock_held`), `src/storage/engine/delete.rs:125` (`acquire`).
**Prompt:**
```
Eres vanta-worker. Norma §2.2 (no flag arguments) + §4.1 OCP. Por cada firma: crea enum
(TelemetryFormat/OutputMode, DoctorMode{Check,Fix{DryRun}}, Permission{Read,Write},
LockPolicy::AssumeHeld/Acquire) y divide en métodos independientes donde aplique.
Slices por archivo con sus tests. PROHIBIDO: mantener el bool "por compat" (si es API pública,
deprecar vía deprecation-and-migration, no dual eterno — ver P2-5). Callers: actualizar todos
(codegraph callers). Verifica: nextest por módulo + clippy + /cleanCA (F2 ✅).
RESULTADO: tabla firma antes→después + callers tocados + tests.
```

### D3 — Structs de comando para 5–7 params 🟡 ⇄ con D2
**Scope:** `src/entity/scene.rs:76` (7), `src/agentic/thread.rs:89` (5), `src/entity/mod.rs:81` (5).
**Prompt:**
```
Eres vanta-worker. Norma §2.2 (objeto de configuración inmutable) + §3.2/§2.5 G7 (Value Objects).
Crea SceneNodeWrite{...}, CreateThread{...}, EntityWrite{...} (readonly/inmutables) y migra las
3 firmas + callers. Slices por struct. PROHIBIDO cambiar validación (validate_key igual).
Verifica: nextest entity/agentic + /cleanCA (F2 ✅). RESULTADO: firmas + callers + tests.
```

### D4a — Renames stuttering internos 🟡 (antes que D1a/D1c)
**Scope:** `EntityStore::entity_*→set/get/delete/list` (`entity/mod.rs:81,110,129,153`), `SceneNodeStore::scene_node_*→set/get`
(`scene.rs:76,107`), `ThreadStore::*_thread→create/get/list/delete` (`agentic/thread.rs:89,173,181,195`),
`Entity::entity_id→id` (+serde alias de compat), `SceneNode::scene_name→name`.
**Prompt:**
```
Eres vanta-worker. Norma Apéndice V.3 (Don't Add Gratuitous Context) + Regla 0 (grep de
referencias ANTES de renombrar) + Regla 3 (docs/api/ en el mismo PR si es pub).
DISCOVERY: codegraph callers de cada símbolo + rg workspace (incluye bindings que los usan).
EJECUCIÓN por símbolo (rename + alias serde donde haya formato persistido + docs).
PROHIBIDO: renombrar QuickInstallProps/i18n/testids/CSS (excepciones V.3); tocar Error (es D4b).
Verifica: cargo check --workspace + nextest afectado + /cleanCA (N2 ✅).
RESULTADO: tabla símbolo antes→después + refs actualizadas + alias compat.
```

### D4b — Enum `Error`: `IoError→Io`, `BackendError→Backend`, … 🟡 (última, breaking)
**Scope:** `src/error.rs:142,157,161,196,216,252,256,260,264,268,280`. **Depende de:** todo lo anterior. **Agentes:** `vanta-arch` diseña, `vanta-worker` implementa, `vanta-review` visa.
**Prompt (arch):**
```
Eres vanta-arch. Norma §4.1 OCP/LSP + Apéndice V.3 + Regla 5 (ADR humano) y Regla 7
(release major). Contexto: Error::XxxError tartamudea; el match de code()/is_retriable()/
recovery_hint() y los bindings dependen de cada variante. Diseña: tabla variante→nuevo nombre,
estrategia de compat (type aliases deprecated vs major directo), orden de migración por binding
(Python/TS/WASM/server), y plan de cargo semver-checks. Evidencia con codegraph (quién matchea
cada variante). RESULTADO: diseño + ADR-borrador(datos) + secuencia de slices para worker.
```

### D5 — Endurecer fronteras 🟡 (D5c primera: riesgo DoS)
- **D5a** `vantadb-ts/src/metadata.ts:38-50` + `guards.ts:154-165`/`vantadb.ts:565-578`: validar en frontera (zod/valibot o asserts que lancen `DbError VALIDATION_ERROR` como `native.ts:94-104`). **Prompt:** vanta-worker, norma §2.3 TS + §5.4; slices: normalizeValue estricta + buildSearchRequestBase valida + tests; PROHIBIDO `any`.
- **D5b** `vantadb-python/src/lib.rs:1242-1252,2317-2327,2360-2375`: `distance_metric`/`method` desconocido → error (como `parse_backend_kind:179-188`), no warn+fallback. Slices + tests pytest.
- **D5c** `src/server/handlers.rs:419,532-534`: clampar `limit/top_k` a `MAX_K` como Python (`clamp_top_k`, ERR-022). Prioritaria. Slices: constante + clamp + test de carga.
- **D5d** `vantadb-ts/src/vantadb.ts:1094-1147`: simetrizar DTO id (`number|bigint` en get/delete/addEdge o documentar límite 2⁵³). Sugerencia: fix o doc.
- Verificación común: tests del binding + `/cleanCA <scope>` (E1/A2 ✅) + Regla 11 si se citan números.

## 7. Validación, verificación y pruebas (global)

| Nivel | Qué | Gate |
|---|---|---|
| Baseline | `/cleanCA <scope>` antes de cada tarea | Hallazgos registrados FIND-* |
| Mecánico | `campaign_verify_cmd` del contrato del task file | PASS obligatorio (si falla → SARL, no cuenta como completo) |
| Calidad | `cargo fmt --check`, `clippy --deny warnings`, `cargo nextest --profile audit`, `cargo deny check` | `just verify` / `dev-tools/verify.ps1` |
| Perf | `canonical_p99` before/after en D1a/D1b | Sin regresión P95/P99 |
| Compat | `cargo semver-checks` (D4b), wire idéntico (D1c) | Verde antes de merge |
| Cierre | `/cleanCA` global | 0 🔴 para cerrar el plan |

## 8. Orden de ejecución

```
D5c → B1 → D4a → B2a → B2b → D1a ⇄ D1b → D1c → D2 ⇄ D3 → D5a ⇄ D5b ⇄ D5d → D4b → /cleanCA global
```

## 9. Criterio de cierre del plan

`/cleanCA` (sin parámetro) con 0 🔴, deuda 🟡 restante registrada con dueño, docs/api al día, ADRs escritos, plan archivado en `docs/plans/archive/`.

> **Decisión humana 2026-09-12 (benches H2):** justificación ACEPTADA — splits mecánicos sin
> cambio semántico + suites verdes como evidencia; canonical_p99 no mide estas rutas y el
> build release está roto por tantivy/rustc (preexistente). Sin bench obligatorio.

## 10. Adaptador de despacho (manda sobre pipeline-full.md — causa raíz 2026-09-11)

> **Diagnóstico:** los IDs de este plan (D5c, B1, D4a…) NO existen en el MCP campaign
> (`get_task_detail D5c` → `Task D5c block not found`; `get_next_task` → `hasTask:false`).
> El prompt de despacho original mandaba al subagente a `pipeline-full.md`, que exige
> `campaign_get_next_task` → `campaign_update_task_state` → `campaign_verify_cmd` y
> "leer el task file si existe". Resultado: el subagente (1) no encuentra la tarea en
> campaign, (2) hace `Read` de `docs/tasks/<ID>.md` inexistente → error de herramienta,
> (3) interpreta el error como fallo y se detiene ANTES de crear el task file.
> El task file nunca nace porque el paso que lo crea está después de los pasos que fallan.

**Todo despacho de este plan OBEDECE este adaptador (prevalece sobre pipeline-full.md):**

1. **PROHIBIDO campaign MCP en este plan:** no llamar `campaign_get_next_task`,
   `campaign_update_task_state`, `campaign_verify_cmd`, `campaign_discover_skills_v2`
   (el plan no está registrado en campaign; esos calls fallan siempre).
2. **Existencia del task file SOLO con `glob`:** jamás `Read` de `docs/tasks/<ID>.md`
   para ver si existe (Read de inexistente = error fatal). `glob docs/tasks/<ID>.md`:
   si no hay match → **PRIMER paso = crearlo con `Write`** (formato §4) y recién ahí seguir.
3. **Skills OBLIGATORIAS vía tool `skill` (mínimo 10 por tarea, nunca prohibir skills):**
   Base siempre: `campaign-executor`, `progreso`, `systematic-debugging`,
   `test-driven-development`, `code-review-and-quality`, `doubt-driven-development`,
   `source-driven-development`, `planning-and-task-breakdown`, `codebase-memory`.
   + según tarea: `security-and-hardening` (si toca trust boundaries/input/auth/storage/FFI/red),
   `performance-optimization` (si toca hot path/engine/vector/serialización),
   `api-and-interface-design` (si toca API pública/bindings), `writing-plans` (si multi-slice).
   El subagente las carga con `skill <nombre>` y declara `SKILLS_CARGADAS:` (≥10).
   Lo único prohibido es el *discovery vía MCP* (`campaign_discover_skills_v2`), no las skills.
4. **Verify directo, sin MCP:** `cargo fmt --check`, `cargo clippy -p <crate> --deny warnings`,
   `cargo nextest run --profile audit -p <crate> <filtro>` corridos en `bash`, más
   `/cleanCA <scope>` de cierre. El contrato se da por cumplido con outputs reales.
5. **Commit:** `git add <solo archivos de la tarea> && git commit -m "<tipo>: <ID> — <qué>"`.
   Lo prepara el worker, lo ejecuta el lead (tabla AGENTS.md).
6. **Gate D pre-respondido:** blast radius de estas tareas <10 archivos, sin símbolos
   públicos nuevos (salvo D4b, que lleva su propio gate) → GO sin `question`.
7. **Resto de pipeline-full.md sí aplica:** DISCOVERY (codegraph_explore + detect via
   `codebase-memory-mcp` si disponible) → EJECUCIÓN por slices → CIERRE + `RESULTADO §7`.
