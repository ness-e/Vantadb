# FIND-110: S4 bandeja de aprobación — decisión lifecycle + re-DEFER fundado

## Metadata

- **Plan file:** `docs/dev/plans/2026-09-17-seguimiento-mvp.md` (Task 2, Wave0)
- **Fuente:** plan seguimiento-mvp Task 2 + FIND-107 §7 (S4)
- **Esfuerzo:** 🟡 1d (consumido: ~1 slice discovery, cero código)
- **Prioridad:** 🟡 Media
- **Tipo:** Rust core (decisión docs-only, sin símbolos nuevos)
- **Turns estimados:** 15-30 → reales: ~8 (uphill cerró en 1 ronda de evidencia)
- **Creado:** 2026-09-17
- **last-synced:** 2026-09-17
- **Estado:** ✅ COMPLETED (re-DEFER fundado — cierre válido, no fracaso)
- **Incógnitas (uphill):** 0 abiertas (1 inicial, resuelta con evidencia)
- **Pendientes (downhill):** 0 (ship cancelado por decisión; nada que ejecutar)

## Tarea

**Objetivo:** cerrar S4 de FIND-107 — o hay lifecycle defendible para la bandeja
de aprobación con test en-proceso, o re-DEFER fundado. Prohibido: mostrador
vacío (exponer tools sin lifecycle) y persistencia completa (fuera de appetite).

**Contrato (plan):** decisión lifecycle escrita (dueño del queue en proceso MCP o
persistencia) + test en-proceso verde que prueba captura→decisión en mismo
proceso + tools/docs si shippea; si no cierra → fila Backlog actualizada con
motivo (re-DEFER válido, NO es fracaso).

**AC de esta ejecución:**

- (a) ✅ decisión lifecycle escrita (ver `## Decisión lifecycle`)
- (b) ✅ test en-proceso verde EXISTENTE verificado mecánicamente
  (`cargo test -p vanta-memory -j 2 --test capture_approval` → 4/4 ok) +
  justificación de re-DEFER en este task file (Backlog lo actualiza el
  orquestador/lead — `docs/dev/Backlog.md` es edición del lead, fuera de mi scope)
- (c) N/A ship (no shippea: sin tabla Spec por tool; ver motivo)

## Archivos

**Clave (leídos completos):**

- `vanta-memory/src/core/record/approval.rs` (158L) — `CaptureApprovalQueue`
  in-memory (`Mutex<Inner>`, `approval.rs:9` scope declarado:
  "one in-memory queue (no threads, no persistence — pre-mortem #2"),
  `submit`/`list_pending`/`approve`/`reject`, `should_gate` default-off
- `vanta-memory/src/gateway/approval_handlers.rs` (118L) — handlers puros
  `capture_list_pending`/`capture_approve`/`capture_reject` + validación
  frontera `require_id`; toman `&CaptureApprovalQueue` por parámetro
  (NO son dueños, NO construyen ninguno)
- `vanta-memory/tests/capture_approval.rs` (135L) — contrato fuente: 4 tests
  AAA (default-off passthrough, submit→approve persiste, reject descarta
  idempotente, approve-unknown falla sin panic)

**Relacionados (leídos, solo lectura):**

- `vanta-memory/src/core/record/mod.rs` — re-export approval
- `vanta-memory/src/gateway/mod.rs` — re-export gateway handlers
- `vanta-memory/src/core/hooks/auto_capture.rs` — único entry L0 host-facing;
  escribe directo vía `L0Recorder`, NO produce `ExtractedMemory`, NO consulta
  el gate (sin path captura→cola)
- `vantadb-mcp/src/server.rs` (`run_stdio_server`, `serve_lines`,
  `dispatch_request`, `dispatch_request_proxy`, `mcp_proxy_handler`,
  `run_stdio_server_auto`) — evidencia del modelo de proceso MCP
- `vantadb-mcp/src/handlers/tools.rs` (`handle_tools_call(&params, &executor,
  &storage, &cfg)`) — dispatch stateless por llamada, sin estado compartido
- `vantadb-mcp/src/dreams.rs` (patrón FIND-107 S2+S3: tools sobre store
  persistido `&Embedded`) — contraste: dreams/scenes persisten en DB,
  approval vive solo en RAM
- `vantadb-mcp/src/context.rs` — patrón handler glue (validación frontera →
  core → envelope tipado), coherente con R-8

**Prohibidos (no tocados):** persistencia nueva on-disk · `src/wal*.rs` +
`src/cli_handlers/` (FIND-109 ✅ `9c881ac5`) · `vanta-memory/src/core/skill/`
(FIND-111) · `reparacion.bat` · `.opencode` · `Justfile` · `ocr-*` ·
`completions/*` · `desktop/src-tauri/Cargo.lock` · stash@{0} GOV-C4 ·
`docs/dev/Backlog.md` (lead) · plan file (solo recitation) · `C:/Users/Eros/.vantadb*`
· WIP ajeno en `git status` (`completions/`, `.opencode`, `docs/dev/Backlog.md`).

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers de `CaptureApprovalQueue` | NINGUNO en producción (solo tests + re-export + handlers que la reciben por parámetro) |
| Callers de `should_gate` | NINGUNO en producción (solo tests) |
| Callees de approval.rs | `apply_dedup_batch` (l1_writer), `Embedded` (SDK), `ExtractedMemory`/`MemoryRecord` |
| Callees de approval_handlers.rs | queue (parámetro) + `EmbedFn` (parámetro) — puro glue R-8 |
| Implicaciones | cero código tocado → cero blast radius; ningún contrato cambia; ningún test afectado |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** approval.rs, approval_handlers.rs,
  capture_approval.rs, record/mod.rs, gateway/mod.rs, auto_capture.rs,
  server.rs, handlers/tools.rs (parcial 120L cabecera+registry),
  dreams.rs (parcial 120L), context.rs
- **Archivos referenciados hacia dentro:** approval.rs → l1_writer
  (`apply_dedup_batch`), `vantadb::sdk::Embedded`, abstractions
  (`ExtractedMemory`, `MemoryRecord`); handlers → approval + l1_writer
  (`EmbedFn`); server.rs → handlers/tools, StorageEngine, Executor
- **Archivos que referencian a los editados:** NINGUNO — esta tarea no edita
  código; el único archivo creado es este task file (sin referencias
  entrantes salvo el plan file)
- **Veredicto impacto:** NULO en código (docs-only). Si alguien shippeara las
  3 tools mañana, el impacto sería: `handlers/tools.rs` (registry + dispatch),
  `docs/api/MCP.md` (R-5 paridad), tests MCP nuevos — y seguiría vacío el
  mostrador (ver Decisión).

## Contrato

"Decisión lifecycle escrita + `cargo test -p vanta-memory -j 2
--test capture_approval` 4/4 verde + commit `docs:` solo con este task file;
cero archivos de código tocados; re-DEFER fundado con motivo accionable."

## Spec (SDD — Phase 1b)

**Veredicto Phase 1b:** NO es feature-add — cero símbolos públicos nuevos
(ningún `pub fn`, tool MCP, endpoint o binding agregado). La decisión es
docs-only. Tabla de decisiones resuelta POR EVIDENCIA (no hay ronda
`question`: nada abierto que el repo no responda):

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Dueño del queue en MCP stdio | A) proceso writer vía `static` (pro: visible tras proxy; contra: restart pierde pendientes = pérdida de datos silenciosa; divergencia proxy si se posee mal) / B) persistencia on-disk (PROHIBIDA por contrato) / C) ningún dueño defendible en 1d → re-DEFER | ✅ C por evidencia (refs abajo) |
| 2 | Ship 3 tools sin submit | A) shippear (mostrador vacío: `list` siempre `[]`, `approve` siempre NotFound) / B) no shippear | ✅ B — A es el mostrador vacío PROHIBIDO por el contrato |
| 3 | Submit path mínimo (`capture_submit`) | A) añadir tool submit (nuevo API público → Gate D + diseño multi-proceso + restart-loss; excede 1d) / B) diferir | ✅ B — scope fuera de appetite |

## Decisión lifecycle (AC-a)

**DECISIÓN: re-DEFER fundado. No existe lifecycle defendible en 1d sin violar
el contrato (mostrador vacío o persistencia prohibida).**

Evidencia (toda in-repo, verificable por grep/lectura):

1. **La cola no tiene dueño en ningún proceso.** `capture_list_pending`,
   `capture_approve`, `capture_reject` reciben `&CaptureApprovalQueue` por
   parámetro (`approval_handlers.rs:90,98,111`); nadie construye una fuera de
   tests. Grep workspace (`CaptureApprovalQueue|should_gate|\.submit\(` en
   `*.rs`): únicos hits de producción son definición + re-exports; los
   `.submit(` restantes son del pipeline de ingestión y benches (no
   relacionados).
2. **El gate nunca está on en producción.** `should_gate` tiene CERO callers
   fuera de tests. Aunque existiera un dueño, nada entraría jamás a la cola:
   ni siquiera hay path captura→cola en-proceso.
3. **No hay path captura→cola.** El único entry de captura
   (`AutoCaptureHook::capture`, `auto_capture.rs:84`) escribe L0 directo vía
   `L0Recorder`; no produce `ExtractedMemory` ni consulta el gate. La
   extracción L1 (`extract_l1_memories`) exige runner LLM sin path degradado
   (evidencia FIND-107 §7) — cablear gate ahí es diseño nuevo, no slice de 1d.
4. **El proceso MCP no admite dueño trivial.** `run_stdio_server` SÍ es un
   proceso long-lived por cliente (loop `serve_lines`, `server.rs:377`), pero
   MCP-35 (`run_stdio_server_auto`, `server.rs:833`) divide writer/proxy por
   lock fs2: los clientes no-writer proxean `tools/call` por HTTP al writer
   (`dispatch_request_proxy`, `mcp_proxy_handler`). Una cola `static` local
   divergería entre procesos salvo diseño explícito writer-side + routing
   proxy + semántica de pérdida ante restart (pendientes = memorias NO
   persistidas en ningún lado → restart = pérdida silenciosa, contradice el
   principio never-lose-data del propio hook). Ese diseño + `capture_submit`
   (nuevo API público) excede el appetite y roza la persistencia prohibida.
5. **La mecánica en-proceso YA funciona** (4/4 verde, ver Verificación): el
   gap es 100% ownership/lifecycle, 0% mecánica. El test "captura→decisión en
   mismo proceso" que pide el contrato no puede probarse porque no existe el
   lado "captura" — escribir un test que llama `submit` directo (como los 4
   existentes) probaría lo ya probado, no el lifecycle.

**Motivo re-DEFER (para la fila Backlog — la edita el orquestador/lead):**
"S4 sin dueño defendible: `should_gate` sin callers prod + ningún path
captura→cola + MCP-35 writer/proxy impide cola process-local trivial.
Shippear list/approve/reject solo = mostrador vacío (prohibido).
Requiere diseño submit+ownership+restart (roza persistencia, fuera de
appetite 1d). Mecánica en-proceso verificada 4/4 — el gap es lifecycle, no
código. Reabrir cuando: (i) exista productor que llame `submit` (gate cableado
en un path real de ingesta), o (ii) se apruebe diseño writer-owned con
semántica de pérdida documentada."

**Implicancia SPEC.md F5:** el ítem "aprobación" de F5 queda abierto; sueños
(S2+S3 ✅) y resto F5 no afectados — S4 era el único pendiente de exposición
con gap de lifecycle.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `should_gate` default-off intacto (capturas
  existentes nunca bloquean); `CaptureApprovalQueue` sin dueño global —
  cualquier futuro dueño debe ser explícito (writer-side) con semántica de
  restart documentada; NO añadir tools approval sin path `submit` (mostrador
  vacío prohibido); NO persistir la cola sin ADR (fuera de appetite S4)
- **Comandos de verificación:** `cargo test -p vanta-memory -j 2
  --test capture_approval` (4/4 ok) · `cargo fmt --check` N/A (cero Rust
  tocado) · clippy N/A (cero Rust tocado)
- **Deuda pendiente:** ninguna nueva (saldo Regla 6: 0). Deuda conocida que
  ESTE re-defer registra (no crea): S4-lifecycle sin dueño — vive en la fila
  Backlog FIND-110 con el motivo de arriba

## Recitation (canónico)

- `activeGoal`: FIND-110 S4 bandeja aprobación — lifecycle o re-DEFER
- `lastAction`: DISCOVERY completo (10 fuentes leídas) + grep exhaustivo
  workspace + test mecánica 4/4 verde + decisión re-DEFER escrita en
  `docs/dev/tasks/FIND-110.md` + commit `docs:` solo-task-file
- `result`: OK (re-DEFER fundado = cierre válido del contrato alternativo)
- `nextAction`: orquestador — actualizar fila Backlog FIND-110 con motivo +
  P2-01 sobre la decisión + NextTask FIND-111
- `contract`: verificacion `cargo test -p vanta-memory -j 2
  --test capture_approval` → 4 passed / 0 failed · evidencia: cada claim de
  Decisión cita `file:línea` + output del test · artefactos:
  `docs/dev/tasks/FIND-110.md` · invariantes: gate default-off, sin dueño
  global, sin tools sin submit, sin persistencia sin ADR · deuda: ninguna
  nueva · queda_pendiente: Backlog (lead) + P2-01 (orquestador) + FIND-111
- `nextTask`: FIND-111

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (docs-only, cero código).

## Definition of Done (3 niveles — P2-08)

| Nivel | Gate | Estado |
|-------|------|--------|
| **Task** | Contrato alternativo cumplido (decisión + test verde existente verificado + motivo) + determinista aplicable (fmt/clippy N/A sin Rust) + tests del cambio N/A (sin cambio) | ✅ |
| **Commit** | Atómico (1 archivo nuevo), conventional `docs:`, staging selectivo verificado (`git status`), sin WIP ajeno | ✅ |
| **Release** | N/A — tarea docs sin artefacto shippable; `dev-tools/verify.ps1` completo no aplica (cero código). Justificado en Notas | N/A justificado |

## Herramientas necesarias

- `cargo test -p vanta-memory -j 2` (mecánica) · codegraph_explore (blast
  radius) · grep workspace (exhaustividad submit-path) · `campaign_verify_cmd`
  N/A (cero código; bug exit -1 no alcanzado) · OCR delegation (cierre)

**Skills cargadas (SDP Paso 0b, `campaign_discover_skills_v2` phase BUILD,
maxSkills 8):** spec-driven-development (decisión lifecycle, núcleo) ·
test-driven-development (test en-proceso como evidencia) ·
doubt-driven-development (adversarial sobre ship-vs-defer) ·
incremental-implementation (slice discovery único) · context-engineering
(context pack por slice) · source-driven-development (base Rust) ·
api-and-interface-design (evaluar superficie tools, R-8) · codebase-memory
(blast radius KG) + base campaign-executor/progreso/ponytail(full).
`frontend-ui-engineering` (sugerida por SDP lifecycle, score 1.00) DESCARTADA:
cero superficie `web/` en S4 — justificado, no cargada.
SDP: 8 cargadas + 1 descartada con motivo.

**Notion (Paso 0c):** sin tool `fetch`/Notion disponible en este entorno —
4 páginas no consultables; registrado como limitación, no como evidencia.
Filtro VantaDB N/A en consecuencia.

**Internet (punto 9):** no requerida — incógnita uphill respondida con
evidencia in-repo (`server.rs` writer/proxy + grep exhaustivo). Cero citas
externas, cero deuda TSYS-13.

## Investigation Notes

- MEM-68 diseñó el gate como opt-in de host (`approval.rs:1-9` doc:
  "hosts call apply_dedup_batch directly" cuando off). El "host" en MCP sería
  el server — pero el server MCP no corre capturas (sus tools son CRUD
  memoria/grafos/threads, 86 tools registradas en `tools.rs:25-27`, cero de
  approval). El productor natural (auto-capture hook) vive en desktop/plugin,
  otro proceso → la cola MCP nacería vacía por construcción.
- Contraste S2+S3 (dreams, shippeado FIND-107): dreams persisten en
  `dream/<session>/<run_id>` en la DB — todo tool lee/escribe store
  compartido writer-side sin estado process-local. Approval no tiene store →
  no hay plantilla aplicable sin diseñar ownership.
- `mcp_proxy_handler` (`server.rs:113`) ejecuta `handle_tools_call`
  writer-side para llamadas proxeadas: un futuro dueño writer-side SÍ sería
  visible a todos los clientes. Camino técnico existe, pero exige (submit +
  dueño + restart-semantics) = diseño >1d. Anotado como reapertura, no como
  slice escondido.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — "¿quién posee el queue en MCP stdio?" RESPONDIDA: nadie hoy; dueño trivial diverge por MCP-35; dueño real excede appetite |
| Pendientes de ejecución (downhill) | 0 — ship cancelado por decisión fundada |
| % completado | 100% (discovery + decisión + task file + commit) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — N/A con justificación: cero código, cero trust
  boundaries tocados. Nota para reapertura: un futuro `capture_submit`
  aceptaría `ExtractedMemory` externo → validar en frontera (patrón
  `context.rs`/`dreams.rs`: `validate_identifier` + `validate_payload`) y
  auditar restart-loss como integridad de datos.
- [x] **PERFORMANCE** — N/A con justificación: la cola es `Vec` + `Mutex`
  en-RAM (O(n) `retain`); a escala bandeja humana es irrelevante; si la
  reapertura la convierte en store persistido, exigir baseline antes/after
  (Regla 9, `canonical_p99`).

## Steps

### Step 1: DISCOVERY queue + handlers + tests fuente

- **Archivos:** `vanta-memory/src/core/record/approval.rs`,
  `vanta-memory/src/gateway/approval_handlers.rs`,
  `vanta-memory/tests/capture_approval.rs`
- **Acción:** leer completos vía codegraph_explore + Read; confirmar
  in-memory por proceso y handlers puros con queue por parámetro
- **Verify:** 3 archivos leídos, citas `approval.rs:9,64-72`,
  `approval_handlers.rs:90-118` confirmadas
- **Estado:** ✅ COMPLETED

### Step 2: Investigación uphill — dueño del queue en MCP stdio

- **Archivos:** `vantadb-mcp/src/server.rs`, `handlers/tools.rs`,
  `context.rs`, `dreams.rs` (patrón)
- **Acción:** responder con evidencia del server (no opinión): loop
  long-lived + split writer/proxy MCP-35 + dispatch stateless
- **Verify:** citas `server.rs:300,377,571,659,833` + firma
  `handle_tools_call(&params, &executor, &storage, &cfg)` sin estado
- **Estado:** ✅ COMPLETED

### Step 3: Exhaustividad submit-path + mecánica en-proceso verde

- **Archivos:** workspace `*.rs` (grep), `auto_capture.rs`
- **Acción:** grep `CaptureApprovalQueue|should_gate|\.submit\(` en todo el
  workspace; correr `cargo test -p vanta-memory -j 2 --test capture_approval`
- **Verify:** 0 callers prod de `should_gate`/`submit`; 4/4 tests ok
- **Estado:** ✅ COMPLETED

### Step 4: Decisión lifecycle + task file completo

- **Archivos:** `docs/dev/tasks/FIND-110.md` (nuevo)
- **Acción:** poblar TODO por tarea (este archivo): decisión re-DEFER con
  5 evidencias + motivo Backlog + Spec Phase 1b + DoD 3 niveles
- **Verify:** secciones Metadata→Notas completas, sin referencias vacías
- **Estado:** ✅ COMPLETED

### Step 5: Verify + OCR + commit selectivo + RESULTADO

- **Archivos:** `docs/dev/tasks/FIND-110.md` (único en staging)
- **Acción:** `pwsh dev-tools/ocr-review.ps1` (advisory) + `git add
  docs/dev/tasks/FIND-110.md` + `git commit -m "docs: FIND-110 — ..."` (NO PUSH)
  + `campaign_update_task_state` + bloque RESULTADO §7
- **Verify:** `git status` muestra solo el commit nuevo; WIP ajeno intacto
- **Estado:** ✅ COMPLETED
- **OCR:** `pwsh dev-tools/ocr-review.ps1 -Format json` → `docs/dev/tasks/FIND-110.md`
  excluido (`unsupported_ext`, no revisable por OCR); únicos reviewables son
  WIP ajeno (`completions/*`, `.opencode`) — fuera de scope, no bloquean este
  commit. Cero Critical/High propios.

## Dependencias

- FIND-109 ✅ (`9c881ac5`) — archivos disjuntos, sin interferencia
- NextTask: FIND-111 (orquestador; S5 skill_extract, archivos disjuntos
  `vanta-memory/src/core/skill/`)
- Stop del plan aplicado: sin lifecycle defendible en 1d → re-DEFER con
  motivo (este archivo + RESULTADO; Backlog lo toca el orquestador)

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta el ORQUESTADOR (no el implementador — instrucción de tarea).

- **Revisor:** pendiente (orquestador asigna vanta-review/vanta-audit)
- **Enfoque:** ¿re-DEFER fundado o había lifecycle shippeable en 1d?
  Alternativas evaluadas: ship-3-tools (mostrador vacío, prohibido), ship con
  `capture_submit`+`static` writer-side (nuevo API + restart-loss, >1d),
  test-only scope-proceso sin dueño (ficción sin owner)
- **Cómo se probó:** `cargo test -p vanta-memory -j 2 --test capture_approval`
  → 4 passed / 0 failed (mecánica); decisión por grep exhaustivo + lecturas
  citadas (evidencia, no auto-reporte)
- **OCR:** delegation corrida — `FIND-110.md` excluido `unsupported_ext`;
  reviewables solo WIP ajeno (`completions/*`); cero Critical/High propios,
  no bloquea
- **Checklist anti-hábitos tóxicos:** sin salidas inventadas (test output
  real pegado en Verificación); sin clarificación saltada (uphill respondida
  con evidencia); sin done sin AC (AC-a/b/c mapeados); sin fallos ignorados;
  sin búsqueda única (codegraph + grep + lectura directa server.rs);
  supuestos citados como evidencia con `file:línea`; sin reintentos en bucle;
  cada step conectado al contrato; N/A dinero/seguridad (docs-only);
  presupuesto explícito (stop 1d aplicado como re-DEFER, no como FAILED)
- **Veredicto:** pendiente revisor distinto

## Notas

- Re-DEFER válido ≠ fracaso: el contrato del plan lo lista como cierre
  alternativo de primera clase ("re-DEFER válido"). El valor entregado es
  certeza con evidencia (qué falta exactamente para reabrir) en vez de un
  mostrador vacío que erosionaría confianza (R-3 api-contract).
- `ponytail:` lo más simple que funciona aquí es NO shippear — 0 líneas.
  Skipped: tools/docs/coverage S4; add cuando exista productor `submit` o
  diseño writer-owned aprobado.
- NOTICED BUT NOT TOUCHING: WIP ajeno en `git status` (`completions/`,
  `.opencode`, `docs/dev/Backlog.md` modificados; `reparacion.bat` y plan file
  untracked) — fuera de scope, no se tocan ni se stagean.
- DoD Release N/A justificado: sin artefacto shippable no hay qué certificar
  con `verify.ps1` completo; la capa determinista aplicable (nada que
  compilar/lintear) se cumple por vacuidad + test de mecánica verde.
- Doubt-driven (degradado, anunciado): sin `task`/sub-agente reviewer
  disponible para P2-01 en este contexto (lo hace el orquestador); el ciclo
  adversarial se ejecutó como auto-revisión contra las 3 alternativas de
  Review/Enfoque. Cross-model skip: contexto no-interactivo.

## Verificación

- `cargo test -p vanta-memory -j 2 --test capture_approval` — ✅ 4 passed,
  0 failed (output real):

```text
running 4 tests
test pending_to_reject_drops ... ok
test approve_unknown_id_fails ... ok
test pending_to_approve_persists ... ok
test default_off_passthrough_writes_immediately ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
```

- `cargo check -p vantadb` — N/A (cero Rust tocado)
- `cargo check -p vantadb_py` — N/A (cero bindings tocados)
- `campaign_verify_cmd` — no corrido (cero código; bug exit -1 no alcanzado)
- OCR delegation — ver Step 5
- **Scope discipline:** 1 archivo nuevo (`docs/dev/tasks/FIND-110.md`), 0
  ediciones. WIP ajeno intacto.
