# FIND-110-spec: S4 bandeja de aprobación — diseño lifecycle submit→queue→decisión (spec-first, cero código)

## Metadata

- **Plan file:** `docs/dev/plans/2026-09-18-cierre-mvp.md` (Task 2, Wave0, segunda en secuencia)
- **Fuente:** `docs/dev/tasks/FIND-110.md` (re-DEFER fundado 2026-09-17: 5 evidencias + motivo) + Gate Justificación del plan (espejo FIND-112: la spec convierte el re-DEFER en slice mecánico futuro)
- **Esfuerzo:** 🟡 1d (appetite plan) · **Prioridad:** 🟡 Media
- **Tipo:** spec-first docs-only (cero código; auto-detect `docs`, ver Dependencias para el matiz)
- **Creado:** 2026-09-18
- **last-synced:** 2026-09-18
- **Estado:** ⬜ PENDING → 🟡 IN PROGRESS (esta ejecución escribe la spec; el file ES el entregable)
- **Incógnitas (uphill):** 3 iniciales (productor submit / dueño / restart) → 0 abiertas (resueltas con evidencia, ver Investigación problema)
- **Pendientes (downhill):** 0 (spec-first: nada que ejecutar en código; slices futuros quedan mecánicos, no pendientes de esta tarea)

## Tarea

**Objetivo:** producir el diseño lifecycle de la bandeja de aprobación S4 que faltaba en FIND-110, para volver el re-DEFER un slice mecánico futuro. Spec-first, cero código.

**Contrato (plan):** `docs/dev/tasks/FIND-110-spec.md` (este archivo) con (a) diseño submit→queue→decisión — quién produce, quién posee, lifecycle restart; (b) qué cambia en gateway/handlers — diseño, sin código; (c) gates+tests futuros; (d) qué NO cambia (sin persistencia salvo que el diseño la exija con motivo) + P2-01-spec; **cero código**.

**AC de esta ejecución:**

- (a) ✅ diseño submit→queue→decisión con dueño explícito (ver `## Diseño lifecycle`) — sin dueño no cierra (Stop del plan); hay dueño defendible, la spec cierra
- (b) ✅ qué cambia en gateway/handlers como diseño sin código (ver `## Qué cambia en gateway/handlers`)
- (c) ✅ gates+tests futuros nombrados y verificables (ver `## Gates y tests futuros`)
- (d) ✅ qué NO cambia + tradeoff persistencia escrito + P2-01-spec para el orquestador (ver `## Qué NO cambia` y `## Review`)

## Archivos

**Entregable (única creación permitida):**

- `docs/dev/tasks/FIND-110-spec.md` (ESTE archivo — ES el entregable)

**Lectura (leídos completos en DISCOVERY):**

- `docs/dev/tasks/FIND-110.md` (408L) — 5 evidencias + motivo re-DEFER + contrato alternativo; base de todo el diseño
- `vanta-memory/src/core/record/approval.rs` (158L) — `CaptureApprovalQueue` in-memory (`approval.rs:9` scope: "no threads, no persistence"), `submit`/`list_pending`/`approve`/`reject`, `should_gate` default-off
- `vanta-memory/src/gateway/approval_handlers.rs` (118L) — handlers puros `capture_list_pending`/`capture_approve`/`capture_reject` + `require_id` frontera; toman `&CaptureApprovalQueue` por parámetro (NO son dueños, NO construyen ninguno)
- `vanta-memory/tests/capture_approval.rs` (135L) — contrato fuente: 4 tests AAA (default-off passthrough, submit→approve persiste, reject descarta idempotente, approve-unknown falla sin panic)

**Referencias normativas (lectura completa, ver `## Referencias`):**

- `.opencode/rules/core-engine.md` (R-1–R-5) + `.opencode/rules/api-contract.md` R-8 (glue, lectura completa) — citadas donde aplican
- `.opencode/references/definition-of-done.md` (standing checklist + DoD VantaDB)
- `SPEC.md` raíz (§ Alcance cierre-mvp: "S4/S6b diseño primero en FIND-110-spec/FIND-113-spec (cero código); ship solo con dueño defendible, si no re-DEFER honesto")
- `docs/dev/plans/2026-09-18-cierre-mvp.md` Task 2 (solo recitation/contrato — el plan file no se edita desde la tarea)

**Prohibidos (no tocados ni leídos más allá de lo citado):** `src/`, `vanta-memory/src/` (salvo los 2 archivos de lectura de arriba), `skills/`, `examples/`, `scripts/`, `reparacion.bat`, `.opencode`, `Justfile`, `ocr-*`, `completions/*`, `desktop/src-tauri/Cargo.lock`, stash@{0} GOV-C4, `docs/dev/Backlog.md` (edición del lead), plan file (solo recitation), `C:/Users/Eros/.vantadb*`, `docs/dev/tasks/FIND-98.md` (FIND-98, no tocar), WIP ajeno en `git status` (`completions/`, `.opencode`, `docs/dev/Backlog.md` modificados).

## Blast Radius

Cero código tocado → cero blast radius en código. Mapeo de lectura (qué tocarían los slices futuros, verificado vía `codegraph_explore` en DISCOVERY):

| Dirección | Módulos |
|-----------|---------|
| Callers de `CaptureApprovalQueue` | NINGUNO en producción (solo tests + re-exports + handlers que la reciben por parámetro) — codegraph: 5 refs en `record/mod.rs` + `approval_handlers.rs`, 0 callers prod de `submit`/`should_gate` fuera de tests |
| Callers de `should_gate` | NINGUNO en producción (solo tests) |
| Callees de approval.rs | `apply_dedup_batch` (l1_writer), `Embedded` (SDK), `ExtractedMemory`/`MemoryRecord` |
| Callees de approval_handlers.rs | queue (parámetro) + `EmbedFn` (parámetro) — puro glue R-8 |
| Dueño futuro (diseño, no código hoy) | `vantadb-mcp/src/server.rs` — contexto del writer (`run_stdio_server`, `mcp_proxy_handler` writer-side, `server.rs:113,377,833` según FIND-110) + `vantadb-mcp/src/handlers/tools.rs` (registry + dispatch de las 3 tools cuando shippeen) |
| Implicaciones | este archivo es docs-only: ningún contrato cambia, ningún test afectado, ningún símbolo nuevo |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `docs/dev/tasks/FIND-110.md`, `approval.rs`, `approval_handlers.rs`, `capture_approval.rs`, rules `core-engine.md` + `api-contract.md` (R-8), `definition-of-done.md`, `SPEC.md` § Alcance cierre-mvp, plan file Task 2 (recitation/contrato)
- **Archivos referenciados hacia dentro:** este spec referencia (no edita) `server.rs` (writer/proxy MCP-35), `handlers/tools.rs` (dispatch stateless), `auto_capture.rs` (path captura actual sin gate), `dreams.rs` (contraste store persistido), `context.rs` (patrón glue R-8) — toda la evidencia vía FIND-110, no re-derivada
- **Archivos que referencian a los editados:** NINGUNO — el único archivo creado es este task file (sin referencias entrantes salvo el plan file y la próxima recitation)
- **Veredicto impacto:** NULO en código (docs-only, cero símbolos nuevos, cero ediciones). Si alguien ejecuta los slices futuros, el impacto sería: `vantadb-mcp/src/server.rs` o contexto equivalente (1 sitio: tenencia del `Arc`), `handlers/tools.rs` (registry + dispatch de 3 tools), `docs/api/MCP.md` (R-5 paridad en el mismo PR), tests MCP nuevos — handlers core y queue intactos

## Contrato

"Diseño submit→queue→decisión con dueño explícito + qué cambia en gateway/handlers (diseño sin código) + gates+tests futuros + qué NO cambia con tradeoff persistencia + P2-01-spec; commit `docs:` solo con este task file; cero archivos de código tocados."

## Spec (SDD — Phase 1b, tabla de decisiones)

**Veredicto Phase 1b:** NO es feature-add — cero símbolos públicos nuevos (ningún `pub fn`, tool MCP, endpoint o binding agregado; la spec DISEÑA pero no expone). Tabla resuelta POR EVIDENCIA in-repo (heredada de FIND-110, no re-derivada; sin ronda `question`: nada abierto que el repo no responda y el plan fija Stop honesto en vez de pregunta):

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Dueño del queue | A) writer-side MCP (`Arc` en contexto server; pro: visible a todos vía proxy writer-side ya existente; contra: restart pierde pendientes — mitigado: derivados re-extraíbles, ver Tradeoff) / B) `static` process-local (contra: diverge entre procesos por MCP-35, prohibido por evidencia FIND-110 §4) / C) persistencia on-disk (contra: fuera de appetite, exige ADR+store; solo con tradeoff escrito) / D) sin dueño → Stop + re-DEFER con diseño parcial | ✅ A por evidencia (refs abajo); D era el Stop si A no se sostenía — se sostiene |
| 2 | Productor `submit` en v1 | A) path interno gated en L1 (pro: sin API pública nueva, sin Gate D; contra: exige cablear gate donde hoy no hay path) / B) tool MCP `capture_submit` pública (contra: nuevo API público + routing multi-proceso + diverge sin dueño; excede 1d) / C) hook auto-capture gated (contra: el hook escribe L0 directo sin `ExtractedMemory`; rediseño, no slice) | ✅ A como contrato futuro (el productor llama al `submit` core ya existente; NO se expone tool submit en v1) |
| 3 | Ship de `list`/`approve`/`reject` hoy | A) shippear ya (contra: sin productor cableado = mostrador vacío, PROHIBIDO por FIND-110) / B) no shippear; ship condicionado a Slice B (productor) en el mismo PR | ✅ B — condición de ship explícita en Gates |
| 4 | Persistencia de la cola | A) persistir (contra: nuevo store + migración + ADR; fuera de appetite S4) / B) RAM-only con semántica restart documentada (pro: pendientes = derivados re-extraíbles desde L0/threads persistidos; contra: ventana de pérdida ante restart — aceptada, escrita y logueada, nunca silenciosa) | ✅ B con tradeoff escrito (ver `## Qué NO cambia`); A solo vía ADR futuro |
| 5 | Dónde vive la decisión approve (Store por defecto) | A) `apply_dedup_batch` con decisions vacías (actual: store-everything, documentado `approval.rs:135-136`) / B) decisiones por-memoria en la tool (contra: lógica de negocio en binding, viola R-8) | ✅ A — sin cambio; R-8 lo exige |

## Diseño lifecycle (AC-a)

### Dueño explícito (cierra el Stop del plan)

**Dueño: el proceso writer del servidor MCP (`vantadb-mcp`), tenencia `Arc<CaptureApprovalQueue>` en el contexto del server junto a `StorageEngine`/`Executor`.**

Fundamento (evidencia, no opinión):

1. El modelo de proceso MCP-35 ya resuelve la visibilidad multi-cliente: `mcp_proxy_handler` (`server.rs:113`) ejecuta `handle_tools_call` writer-side para llamadas proxeadas — un queue poseído writer-side ES visible a todos los clientes sin diseño adicional de routing (FIND-110 Investigación Notes §3).
2. Los handlers gateway son glue puro R-8 (toman `&CaptureApprovalQueue` por parámetro, `approval_handlers.rs:90,98,111`): recibirán el borrow del `Arc` del contexto. NO construyen, NO poseen, NO cambian (ver §b).
3. El lock fs2 writer/proxy (`server.rs:833`) garantiza un solo writer: la cola tiene un solo escritor por construcción, sin locks TTL ni coordinación multi-proceso (contraste con FIND-113-spec, donde los locks sí son el problema).
4. `Mutex<Inner>` en-RAM con poison-recover (`approval.rs:80-85`, misma política que `LocalStateBackend`) basta para escala "bandeja humana" (decenas de pendientes; `retain` O(n) irrelevante — PERFORMANCE N/A con justificación).

**Lo que el dueño hace (contrato de tenencia, 4 deberes):**

1. **Construir una vez** al arrancar el writer (junto al engine): `Arc::new(CaptureApprovalQueue::new())`; compartir por clon de `Arc` a cada dispatch (la struct ya declara "cheap to share: wrap in `Arc`", `approval.rs:61-62`).
2. **Prestar, no copiar:** cada llamada a `capture_list_pending`/`capture_approve`/`capture_reject` recibe `&queue` del `Arc` — todos los clientes (directos y proxeados) ven la misma cola.
3. **Anunciar la semántica efímera:** al arrancar, log `approval queue initialized empty (ephemeral: restart drops pending — re-extractable from L0)`; al apagar limpio con pendientes > 0, log warn con el count (observabilidad mínima, ver Gates G1).
4. **No inventar ids ni decisiones:** los ids los crea `submit` (`p_{ms}_{seq}`); las decisiones las toma el humano vía approve/reject; approve persiste con Store-por-defecto vía `apply_dedup_batch` (decisión 5).

### Flujo submit→queue→decisión

```
[FUTURO productor: path L1 gated en-proceso writer]        (Slice B, mismo proceso writer)
        │  gate on → queue.submit(session_key, session_id, memories, now_ms)
        │  gate off → apply_dedup_batch directo (byte-idéntico a hoy, FIND-110)
        ▼
[QUEUE: Arc<CaptureApprovalQueue> en contexto writer]      (Slice A)
        │  Vec<PendingCapture> en-RAM, orden de llegada, id p_{ms}_{seq}
        │  (seq desambigua mismo-ms; restart vacía la cola → sin colisión intra-cola viva)
        ▼
[DECISIÓN humana vía 3 tools MCP]                          (Slice A, condicionadas a Slice B para ship)
   list    → capture_list_pending(&queue) → snapshot (solo lectura)
   approve → capture_approve(&queue, &db, req, now_ms, embed) → persiste (Store default) + drop de la cola
   reject  → capture_reject(&queue, req) → drop sin persistir (idempotente: 2º reject = false)
```

**Invariantes del flujo (mecánica ya verificada 4/4, no re-diseñada):**

- approve fallido (Store error) NO saca la entrada de la cola (el `?` retorna antes del `retain`, `approval.rs:137-146`): reintentable, nunca pérdida silenciosa.
- approve de id desconocido = error `NotFound`, no panic (`capture_approval.rs:127-135`).
- reject idempotente (`capture_approval.rs:103-125`).
- gate default-off: capturas existentes nunca bloquean (`should_gate` intacto).

### Lifecycle restart (la 3ª incógnita, cerrada)

1. **Restart = cola vacía.** Sin excepción, sin recovery, sin migración. La cola nace vacía con cada writer.
2. **No es pérdida de datos de usuario:** los pendientes son `ExtractedMemory` DERIVADOS; la fuente (mensajes L0 / threads) persiste en la DB por el path directo actual. Re-extraer regenera lo pendiente. Contraste: dreams (S2+S3) persisten porque son dato primario; approval-pending es derivado → RAM-only es sound (este es el tradeoff que faltaba escribir en FIND-110).
3. **Nunca silencioso:** (i) arranque loguea la semántica efímera; (ii) apagado limpio con pendientes loguea warn + count; (iii) la descripción de `capture_list_pending` (cuando shippee) documenta "ephemeral: pending captures are lost on server restart; re-extract from source".
4. **Reapertura con persistencia:** solo vía ADR futuro si la bandeja deja de ser humana (miles de pendientes) o si el productor pasa a otro proceso. Hoy: prohibido por appetite (decisión 4).

## Qué cambia en gateway/handlers (AC-b — diseño, sin código)

**Respuesta corta: NADA en `approval.rs` ni en `approval_handlers.rs`.** El diseño no exige cambiar ni una firma. Lo que cambia vive en DOS sitios fuera de ellos:

**Cambio 1 — Tenencia (writer context, Slice A mecánico):**

- Un campo `approval_queue: Arc<CaptureApprovalQueue>` en el contexto/holder que ya transporta `StorageEngine`+`Executor` al dispatch (el holder exacto lo nombra el implementador en DISCOVERY del slice; candidatos: struct de contexto del server o parámetro thread-through junto a `&storage`/`&executor` en `handle_tools_call` — decisión de cableado, no de diseño).
- Construcción una vez al arrancar el writer; clon de `Arc` por dispatch. Estimación: <30 líneas + 2 logs (arranque/apagado).
- Alternativa descartada con motivo: `static` global (diverge writer/proxy por MCP-35; además `static` es dueño global implícito — viola el invariante "sin dueño global" de FIND-110; el `Arc` explícito en contexto es el dueño explícito que el pre-mortem exigía).

**Cambio 2 — Exposición condicionada (3 tools, Slice A, ship bloqueado hasta Slice B):**

- Registrar `capture_list_pending` / `capture_approve` / `capture_reject` en `handle_tools_list` + dispatch en `handle_tools_call` delegando a los handlers existentes (glue → core, patrón `context.rs`, coherente con R-8).
- `require_id` y envelopes tipados se reutilizan tal cual (validación frontera ya existe).
- Schemas + descripciones: incluir la semántica efímera en la descripción de cada tool (R-5: docs en el mismo PR).
- Estimación: <60 líneas + schemas + docs `docs/api/MCP.md` (R-5 paridad mismo PR).

**Cambio 3 — Productor gated (Slice B mecánico, el que habilita el ship):**

- En el futuro path L1 que produzca `ExtractedMemory` en el writer: `if should_gate(&config) { queue.submit(...) } else { apply_dedup_batch(...) }` — el `submit` core YA existe (`approval.rs:88`); cablear ≠ diseñar.
- Config: `CaptureApprovalConfig.enabled` default-off ya existe (Safe Defaults: el slice no añade flags; el flag ES el gate).
- Estimación: <20 líneas en el call-site + test T8.

**Suma slices futuros:** A (~90 líneas con docs/tests) + B (~40 líneas con test) — cada uno ≤100 líneas, independientes en orden A→B, revertibles por separado (aditivos; revert = no registrar tools / no cablear gate).

## Gates y tests futuros (AC-c)

### Tests existentes (contrato fuente, no re-ejecutar para esta spec — cero código)

- T1 `default_off_passthrough_writes_immediately` — gate off = passthrough byte-idéntico
- T2 `pending_to_approve_persists` — submit→approve persiste, cola a 0
- T3 `pending_to_reject_drops` — reject descarta, idempotente
- T4 `approve_unknown_id_fails` — error sin panic

### Tests futuros (nombrados, para los slices; espejo del estilo FIND-112 "tests 1-10")

- T5 `shared_arc_single_view` (Slice A): dos handles (`&queue` del mismo `Arc`) ven el mismo pendiente tras un `submit` — prueba scope-proceso del dueño (un writer, una vista).
- T6 `failed_approve_keeps_queued` (Slice A): approve con DB que falla (Store error) → `is_err` + `pending_count() == 1` — fija el invariante "fallo no saca de cola".
- T7 `fresh_queue_starts_empty` (Slice A): queue nuevo → `pending_count() == 0` — fija la semántica restart como test (efímero por construcción).
- T8 `gated_path_submits_instead_of_persisting` (Slice B): con gate on, el path productor deja 0 records + 1 pendiente; con gate off, 1 record + 0 pendientes — el test captura→decisión en-proceso que FIND-110 declaró imposible sin productor (el productor lo trae el Slice B).

### Gates de los slices (G0–G3, espejo FIND-112)

- **G0 (seguridad/higiene):** cero secrets (no hay ninguno en este diseño); `cargo fmt --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` 0 warnings; difSolo-archivos-del-slice.
- **G1 (observable):** con gate on y productor (Slice B): submit→`list` muestra 1 → approve → 1 record + cola 0; reject → 0 records + cola 0; arranque loguea semántica efímera; apagado con pendientes loguea warn+count.
- **G2 (paridad R-5):** si el slice registra tools, `docs/api/MCP.md` actualizado en el MISMO PR + `validate-docs-coverage` 0 gaps; descripciones con semántica efímera.
- **G3 (suites):** `cargo test -p vanta-memory -j 2` (T1–T8) + suite MCP del dispatch verdes, sin regresión en reads/stats; **condición de ship:** las 3 tools solo shippean en el PR que incluya Slice B (productor) — ship de A sin B = mostrador vacío, prohibido.

## Qué NO cambia (AC-d)

1. **`vanta-memory/src/core/record/approval.rs`** — ni firmas ni mecánica (`submit`, `list_pending`, `pending_count`, `approve`, `reject`, `should_gate`, `PendingCapture`, errores). El `Mutex`+`Vec` en-RAM es suficiente por diseño (bandeja humana).
2. **`vanta-memory/src/gateway/approval_handlers.rs`** — handlers siguen puros (queue por parámetro, `require_id` frontera, envelopes). R-8: la lógica vive en el core; el gateway es glue.
3. **`should_gate` default-off** — capturas existentes nunca bloquean; el gate es opt-in del host.
4. **Path L0 directo (`auto_capture.rs`)** — escribe vía `L0Recorder` sin gate; el productor gated es el path L1 futuro, no este.
5. **Stores persistidos (dreams/scenes/L0)** — la cola no toca WAL/storage/migraciones; cero cambios en durabilidad.
6. **Sin tool `capture_submit` pública en v1** — `submit` core (`pub`, ya existe) lo llama el productor interno; exponerlo como tool es API pública nueva (Gate D) y se difiere con motivo (decisión 2).
7. **Sin `static` global, sin threads, sin daemon** — el dueño es un `Arc` explícito en contexto writer (Regla 8: sin daemon; sin concurrencia nueva que auditar).
8. **Sin persistencia de la cola** — tradeoff escrito (decisión 4 + §restart): pendientes derivados re-extraíbles desde fuente persistida; persistir exigiría store+diseño+ADR fuera de appetite 1d. Solo se reconsidera vía ADR si la bandeja deja de ser humana o el productor sale del writer.

## Invariantes de dominio (handoff — MUST)

- **`should_gate` default-off intacto** (capturas existentes nunca bloquean)
- **Dueño explícito writer-side** (`Arc` en contexto server); prohibido `static` global y prohibida cola process-local por cliente
- **NO añadir tools approval sin productor cableado** (mostrador vacío prohibido — condición de ship G3)
- **NO persistir la cola sin ADR** (fuera de appetite S4)
- **Approve fallido no saca de cola; reject idempotente; approve-unknown = error sin panic**
- **Comandos de verificación:** `git diff --check` (esta spec) · a futuro: `cargo test -p vanta-memory -j 2` (T1–T8) + clippy/fmt + `validate-docs-coverage.ps1`
- **Deuda pendiente:** ninguna nueva (saldo Regla 6: 0). La deuda S4-lifecycle queda CERRADA en diseño; su ejecución futura son los slices A+B mecánicos de arriba

## Recitation (canónico)

- `activeGoal`: FIND-110-spec diseño lifecycle S4 (spec-first, cero código)
- `lastAction`: DISCOVERY (FIND-110 + 3 fuentes + rules R-8/core-engine + SPEC.md + codegraph + diff-check) + spec escrita en `docs/dev/tasks/FIND-110-spec.md` (dueño writer-side `Arc`, submit interno, restart efímero documentado, slices A+B, T5–T8, G0–G3) + commit `docs:` solo-task-file
- `result`: OK (spec completa a+b+c+d + dueño explícito; Stop no disparado — el dueño se sostiene con evidencia)
- `nextAction`: orquestador — P2-01 sobre LA SPEC (sección Review) + NextTask FIND-113-spec
- `contract`: verificacion `git diff --check` limpio + `git status` solo-task-file en staging · evidencia: cada claim de diseño cita `file:línea` (approval.rs:9,61-62,80-85,88,135-146; handlers:90,98,111; server.rs:113,377,833 vía FIND-110) · artefactos: `docs/dev/tasks/FIND-110-spec.md` · invariantes: ver sección (default-off, dueño explícito, sin tools sin productor, sin persistencia sin ADR) · deuda: ninguna · queda_pendiente: P2-01 (orquestador) + FIND-113-spec
- `nextTask`: FIND-113-spec

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** 0 (docs-only, cero código). `ponytail:` lo más simple que cierra es NO cambiar código — el diseño reutiliza `submit`/`handlers`/`should_gate` existentes y añade solo tenencia + cableado futuro. Skipped: persistencia, tool submit pública, daemon; add solo vía ADR o slices A+B.

## Definition of Done (3 niveles — P2-08)

| Nivel | Gate | Estado |
|-------|------|--------|
| **Task** | Contrato a+b+c+d en este archivo + dueño explícito (Stop no disparado con motivo fundado) + determinista aplicable (diff-check limpio; fmt/clippy/nextest N/A por cero código, justificado) | ✅ |
| **Commit** | Atómico (1 archivo nuevo), conventional `docs:`, staging selectivo verificado (`git status` solo-task-file), sin WIP ajeno, NO PUSH | ✅ |
| **Release** | N/A — spec docs sin artefacto shippable; `verify.ps1` completo no aplica (cero código). OCR N/A-justificado (cero código; `unsupported_ext` para `.md`, precedente FIND-110 Step 5) | N/A justificado |

## Herramientas necesarias

- `codegraph_explore` (lectura queue/handlers/tests + blast radius) · `git diff --check` + `git status` (scope/staging) · `campaign_verify_cmd` no corrido por bug exit -1 NO alcanzado (cero código: se usa bash directa `git diff --check`, precedente plan Riesgos globales) · OCR N/A-justificado (cero código)

**Skills cargadas (SDP Paso 0b, `campaign_discover_skills_v2` phase DEFINE, maxSkills 8):** spec-driven-development (núcleo: tabla decisiones + diseño) · doubt-driven-development (dueño difuso: adversarial sobre A-vs-B-vs-C + Stop honesto) · documentation-and-adrs (spec como documento de decisión) · writing-plans (estructura del task file) · interview-me + idea-refine (lifecycle DEFINE, base SDP) · writing-guidelines (prosa docs) + base campaign-executor/progreso/ponytail(full).
SDP: 7 cargadas-útiles + 1 base; `doubt-driven-development` añadida por sugerencia del plan (dueño difuso, pre-mortem #2) aunque SDP no la rankeó — justificado.
`frontend-ui-engineering` y similares no aplican (cero superficie `web/`).

**Notion (Paso 0c):** sin tool `fetch`/Notion disponible en este entorno — 4 páginas no consultables; registrado como limitación, no como evidencia (precedente FIND-110).

**Internet (punto 9):** NO requerida — las 3 incógnitas se cierran con evidencia in-repo (MCP-35 writer/proxy + handlers puros + tests fuente). Cero citas externas, cero deuda TSYS-13. Patrones Mem0/Letta no consultados: el diseño no los necesita (el gate es Cursor-pattern MEM-68 ya decidido en repo; la novedad es tenencia+restart, puramente local).

## Investigation Notes

### Código (DISCOVERY — queue lifecycle, handlers puros, tests fuente, evidencia re-DEFER)

- `approval.rs:9` fija el scope ("one in-memory queue (no threads, no persistence)"): la spec lo MANTIENE, no lo contradice — el diseño convierte ese scope en semántica restart documentada en vez de dejarlo como deuda implícita.
- `approval.rs:61-62` anticipa el dueño ("cheap to share: wrap in `Arc` if several owners need it"): el diseño toma esa invitación literal — un `Arc`, un writer.
- `approval.rs:80-85` poison-recover igual que `LocalStateBackend`: sin nueva política de errores que diseñar.
- `approval.rs:135-136` documenta Store-por-defecto en approve: la decisión 5 fija no moverla al binding (R-8).
- Handlers `approval_handlers.rs:90,98,111` reciben `&CaptureApprovalQueue`: el diseño los deja intactos — el `Arc` se dereferencia en el call-site del dispatch.
- Tests `capture_approval.rs` (4 AAA) son el contrato fuente de mecánica: T5–T8 los extienden, no los reescriben.
- Re-DEFER FIND-110 (5 evidencias): 0 callers `should_gate` + sin submitter + MCP-35 impide cola trivial + mostrador vacío prohibido + mecánica 4/4 verde. Esta spec responde punto por punto: productor = Slice B interno (evidencia 2), dueño = writer-side `Arc` con routing proxy ya existente (evidencias 1+4), restart = efímero documentado + re-extraíble (evidencia 4, pérdida → no-silenciosa), ship condicionado a A+B (evidencia: mostrador), mecánica intacta (evidencia 5).

### Problema (las 3 incógnitas uphill, con evidencia no opinión)

| # | Incógnita | Respuesta + evidencia |
|---|-----------|----------------------|
| 1 | ¿Quién produce (`submit`)? | El futuro path L1 gated en-proceso writer (interno, no tool pública). Evidencia: `auto_capture.rs:84` escribe L0 directo sin `ExtractedMemory` (no es productor); `submit` core ya existe (`approval.rs:88`) esperando un llamador; exponer `capture_submit` sería API pública nueva (Gate D, >1d). |
| 2 | ¿Quién posee (dueño)? | Writer MCP (`Arc` en contexto server). Evidencia: `mcp_proxy_handler` ejecuta writer-side (`server.rs:113`) → visibilidad multi-cliente gratis; lock fs2 single-writer (`server.rs:833`) → sin coordinación; `static` diverge por MCP-35 (FIND-110 §4). |
| 3 | Lifecycle restart | Efímero + logueado + re-extraíble. Evidencia: scope in-memory declarado (`approval.rs:9`); fuente L0 persiste por path directo (nunca se pierde dato de usuario); dreams-persisten vs approval-derivado justifica la asimetría. |

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — productor / dueño / restart RESPONDIDAS con evidencia |
| Pendientes de ejecución (downhill) | 0 — slices A+B son trabajo futuro mecánico, no pendiente de esta spec |
| % completado | 100% (discovery + diseño + task file + commit) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — N/A con justificación: cero código, cero trust boundaries tocados. Nota para Slice A/B: `approve`/`reject` aceptan `id` externo → `require_id` ya valida frontera vacía; un futuro productor que acepte `ExtractedMemory` externo debe validar como `context.rs`/`dreams.rs` (`validate_identifier` + `validate_payload`); restart-loss queda como integridad documentada, no silenciosa.
- [x] **PERFORMANCE** — N/A con justificación: `Vec`+`Mutex` en-RAM, O(n) `retain` a escala bandeja humana irrelevante; si un ADR futuro persiste la cola, exigir baseline before/after (Regla 9, `canonical_p99`).

## Steps

### Step 1: DISCOVERY queue + handlers + tests fuente + evidencia re-DEFER

- **Archivos:** `docs/dev/tasks/FIND-110.md`, `approval.rs`, `approval_handlers.rs`, `capture_approval.rs`, rules `core-engine.md` + `api-contract.md` R-8, `definition-of-done.md`, `SPEC.md` §cierre-mvp
- **Acción:** leer completos + `codegraph_explore` blast radius + `campaign_discover_skills_v2` DEFINE + `git diff --check` baseline
- **Verify:** citas `approval.rs:9,61-62,80-85,88,135-146`, `handlers:90-118`, 4 tests fuente, SDP registrado
- **Estado:** ✅ COMPLETED

### Step 2: Resolver las 3 incógnitas uphill con evidencia

- **Archivos:** FIND-110 §Decisión (5 evidencias) + `server.rs` writer/proxy vía FIND-110 (sin re-leer `vanta-memory/src/` más allá del scope)
- **Acción:** productor interno vs tool pública / dueño writer-side `Arc` vs `static` vs persistencia / restart efímero+re-extraíble; doubt-driven adversarial (¿y si el productor vive en otro proceso? → fuera de appetite, ADR futuro; ¿y si la bandeja crece? → ADR persistencia)
- **Verify:** tabla decisión 1–5 con opciones+tradeoffs resueltos; Stop evaluado y NO disparado con motivo (dueño se sostiene)
- **Estado:** ✅ COMPLETED

### Step 3: Escribir la spec (a+b+c+d + dueño) en este archivo

- **Archivos:** `docs/dev/tasks/FIND-110-spec.md` (nuevo, único permitido)
- **Acción:** poblar TODO por tarea: diseño lifecycle + cambios gateway (diseño) + gates/tests futuros + no-cambia + tradeoff + DoD + Review P2-01-spec
- **Verify:** secciones Metadata→Verificación completas, dueño explícito presente, cero código
- **Estado:** ✅ COMPLETED

### Step 4: Verify + commit selectivo + RESULTADO

- **Archivos:** `docs/dev/tasks/FIND-110-spec.md` (único en staging)
- **Acción:** `git diff --check` + `git status` (solo-task-file) + `git add docs/dev/tasks/FIND-110-spec.md` + `git commit -m "docs: FIND-110-spec — ..."` (NO PUSH) + `campaign_update_task_state` completed + bloque RESULTADO §7
- **Verify:** commit existe; WIP ajeno intacto; OCR N/A-justificado
- **Estado:** ✅ COMPLETED

## Dependencias

- FIND-110 ✅ (re-DEFER fundado — base evidencial; esta spec la convierte en slices mecánicos, no la reedita)
- FIND-98 ✅ (re-DEFER con evidencia; Wave0 primera; archivos disjuntos — fuera-del-repo vs docs/dev/tasks)
- Nota tipo: auto-detect dijo `docs`/`Documentation`; el contenido es spec de diseño lifecycle Rust-core/MCP (skills SDD+DDD aplicadas por contrato del plan, no por el label). Sin conflicto: el entregable es docs-only.
- NextTask: FIND-113-spec (orquestador; Wave0 tercera; comparte FIND-112 §(c) como referencia de coherencia sin editarla; archivos disjuntos `docs/dev/tasks/113` vs `docs/dev/tasks/110`)
- Stop del plan evaluado: dueño defendible SÍ existe en diseño (writer-side `Arc` + restart efímero) → la spec cierra; el Stop queda documentado para la EJECUCIÓN futura (si Slice A/B revelan que el productor no puede vivir en el writer → re-DEFER honesto con esta spec como diseño parcial, no forzar)

## Review (GATE — agente distinto, P2-01-spec)

> Lo ejecuta el ORQUESTADOR (no el implementador — instrucción de tarea). P2-01-spec: review SOBRE LA SPEC, no sobre código.

- **Revisor:** pendiente (orquestador asigna vanta-review)
- **Enfoque:** ¿el dueño writer-side `Arc` es defendible o esconde un `static` con otro nombre? ¿restart-efímero es sound (derivados re-extraíbles) o minimiza pérdida real? ¿la condición de ship (A+B mismo PR) bloquea de verdad el mostrador vacío? ¿algún cambio listado en §b exige en realidad cambiar `approval.rs`/`handlers` (rompería el claim "NADA")?
- **Alternativas evaluadas:** ship-3-tools sin productor (mostrador vacío, prohibido) · `capture_submit` pública (API nueva + Gate D, diferida con motivo) · `static` process-local (diverge MCP-35, prohibido) · persistencia ya (ADR+store, fuera de appetite) · re-DEFER con diseño parcial (era el Stop; no hizo falta)
- **Cómo se probó:** DISCOVERY con citas `file:línea` + codegraph blast radius + `git diff --check` limpio; mecánica heredada 4/4 de FIND-110 (no re-ejecutada: cero código)
- **OCR:** N/A-justificado — `.md` excluido (`unsupported_ext`, precedente FIND-110 Step 5); cero Critical/High propios, no bloquea
- **Checklist anti-hábitos tóxicos:** sin salidas inventadas (sin test output propio porque cero código; mecánica citada de FIND-110 con output real pegado allá); sin clarificación saltada (3 uphill con evidencia); sin done sin AC (a+b+c+d mapeados 1:1); sin fallos ignorados; sin búsqueda única (Read + codegraph + grep-FIND-110 + rules + SPEC); supuestos citados como evidencia con `file:línea`; cada step conectado al contrato; N/A dinero/seguridad-ejecución (docs-only); Stop evaluado explícitamente
- **Veredicto:** pendiente revisor distinto

## Notas

- Espejo FIND-112 deliberado: slices A+B ≤100L cada uno, tests nombrados T5–T8, gates G0–G3, condición de ship explícita — el futuro implementador no diseña, cablea.
- `ponytail:` lo más simple que cierra es NO cambiar código hoy — el diseño reutiliza `submit`, handlers puros y `should_gate` existentes. Skipped: persistencia, tool submit, daemon; add cuando slices A+B o ADR futuro.
- Coherencia FIND-113-spec (no editar): el dueño aquí es writer-proceso con cola RAM efímera; el scheduler de 113 necesita dueño de backend con locks — coherentes (ambos writer-scoped) sin ser idénticos; el orquestador lo verifica en 113-spec §(d).
- NOTICED BUT NOT TOUCHING: WIP ajeno en `git status` (`completions/*`, `.opencode`, `docs/dev/Backlog.md` modificados; `reparacion.bat` y plan file untracked) — fuera de scope, no se tocan ni se stagean.
- DoD Release N/A justificado: sin artefacto shippable no hay qué certificar con `verify.ps1` completo; `campaign_verify_cmd` no corrido por bug exit -1 no alcanzado (bash directa `git diff --check`, precedente plan Riesgos).
- Doubt-driven (degradado, anunciado): sin `task`/sub-agente reviewer disponible para P2-01 en este contexto (lo hace el orquestador); el ciclo adversarial se ejecutó como auto-revisión contra las 5 decisiones + Stop. Cross-model skipped: contexto no-interactivo.

## Verificación

- `git diff --check` — ✅ limpio (ver Step 4)
- `git status --short` — ✅ solo `docs/dev/tasks/FIND-110-spec.md` en staging al commitear; WIP ajeno intacto
- `cargo check -p vantadb` — N/A (cero código tocado)
- `cargo test -p vanta-memory -j 2 --test capture_approval` — N/A re-ejecución (mecánica heredada 4/4 de FIND-110 con output real pegado allá; cero código que la afecte)
- `campaign_verify_cmd` — no corrido (cero código; bug exit -1 no alcanzado; bash directa en su lugar)
- OCR delegation — N/A-justificado (cero código; `.md` `unsupported_ext`)
- **Scope discipline:** 1 archivo nuevo (`docs/dev/tasks/FIND-110-spec.md`), 0 ediciones. WIP ajeno intacto.
