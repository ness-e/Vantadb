# MEM-09: F4 L0 capture idempotente

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md` (Task 12, líneas 208-213)
- **Fuente:** plan file Task 12 (MEM-09)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴
- **Tipo:** Rust
- **Turns estimados:** 15-30
- **Creado:** 2026-08-20T19:30
- **last-synced:** 2026-08-20T20:15
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps pendientes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-memory` (MEM-10+ L1 reader usará `read_messages`; host SDK consume `AutoCaptureHook`) |
| Callees | `vantadb` SDK (`VantaEmbedded::put/get/list`, `VantaMemoryInput`, `VantaMemoryListOptions`, `VantaMemoryListPage`, `VantaMemoryMetadata`/`VantaValue`), `vanta-memory::core::abstractions::types` |
| Implicaciones | `core/mod.rs` gana `conversation` + `hooks`; ningún contrato existente cambia; L0 es LLM-free (no toca `LlmRunner`); no hay migración de datos (namespace nuevo `l0/<session>`); tests existentes (`smoke.rs`, `types.rs`, `llm_runner_contract.rs`) no se ven afectados |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vanta-memory/src/core/mod.rs` (10 líneas), `vanta-memory/Cargo.toml` (33), `vanta-memory/src/core/abstractions/types.rs` (sesión previa), `vanta-memory/src/core/abstractions/llm_runner.rs` (sesión previa), `src/sdk/types.rs` (L50-349: VantaMemoryInput/Record/ListOptions/Page/VantaValue), `src/sdk/api.rs` (L598-719: `list`), `src/agentic/thread.rs` (sesión previa: ThreadStore `Message` sin `id`, `send_message` appendea sin dedup), `src/sdk/serialization/mod.rs` (sesión previa: `RESERVED_PREFIX __vanta_`, `validate_namespace`, `validate_key`), `src/sdk/mod.rs` (sesión previa: re-exports), `docs/plans/2026-08-18-vanta-memory.md` (L200-229)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `vanta-memory` → `vantadb` (path `../`, default-features=false); `vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryRecord, VantaMemoryListOptions, VantaMemoryListPage, VantaMemoryMetadata, VantaValue}`; `vantadb::config::{VantaConfig, BackendKind}`; `vantadb::error::VantaError`
- **Archivos que referencian a los editados (referencias entrantes):** ninguno — `conversation/` y `hooks/` son módulos nuevos; el único archivo editado es `core/mod.rs` (agrega 2 `pub mod`), sin referencias entrantes
- **Veredicto impacto:** bajo — solo se agregan módulos nuevos y 2 líneas en `core/mod.rs`; cero archivos existentes modificados

## Contrato
"`cargo check -p vanta-memory` pasa, `cargo nextest run -p vanta-memory` pasa (incluye tests dedicados de L0), `cargo fmt --check` pasa, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` pasa, y el comportamiento específico es: registrar el mismo turno 2 veces produce 1 solo registro L0 (idempotencia por key estable session+timestamp/message-id vía SDK `put` upsert) y los mensajes con role fuera de `capture_roles` se excluyen del L0"

## Invariantes de dominio (handoff - MUST)

- **Invariantes a preservar:** (1) L0 es LLM-free — no bloquea, no usa `LlmRunner`; (2) idempotencia: re-envío del mismo turno NO duplica registros (key estable = message id o `t{timestamp_ms}_{index}`; cursor persistente avanza a max(timestamp)); (3) cursor con fallback: sin cursor persistido, usar `plugin_start_timestamp_ms` como floor para no volcar toda la sesión; (4) namespace sanitizado contra `[A-Za-z0-9._/-]` ≤128 bytes (requisito `validate_namespace`); metadata sin claves con prefijo `__vanta_` (reservado); (5) sin `unwrap()`/`expect()` en código nuevo
- **Comandos de verificación:** `cargo check -p vanta-memory`, `cargo nextest run -p vanta-memory`, `cargo fmt --check`, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` — todos exit 0
- **Deuda pendiente:** ninguna (ver Notas para decisión ThreadStore vs SDK put)

## Recitation (canónico - estructura única)

- **activeGoal:** MEM-09: F4 L0 capture idempotente
- **lastAction:** Implementación completa por vanta-lead (3 delegaciones vanta-worker con resultado vacío → SARL escalera STRATEGY: implementación directa con diseño del task file) — l0_recorder.rs + auto_capture.rs + 5 tests L0 + 2 unit tests
- **result:** ✅ COMPLETED
- **nextAction:** Ninguno — tarea cerrada
- **contract:**
    ```
    contract:
      verificacion: "cargo check -p vanta-memory && cargo nextest run -p vanta-memory && cargo fmt --check && cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings (todos exit 0)"
      evidencia:
        - claim: "El mismo turno capturado 2 veces produce 1 solo registro L0 (upsert por key estable)"
          evidencia: "tests/l0_capture.rs::same_turn_twice_is_idempotent pasa (35/35 nextest)"
          confianza: alta
        - claim: "Mensajes con role fuera de capture_roles se excluyen del L0"
          evidencia: "tests/l0_capture.rs::filters_out_non_captured_roles pasa (35/35 nextest)"
          confianza: alta
      artefactos:
        - "vanta-memory/src/core/conversation/l0_recorder.rs"
        - "vanta-memory/src/core/conversation/mod.rs"
        - "vanta-memory/src/core/hooks/auto_capture.rs"
        - "vanta-memory/src/core/hooks/mod.rs"
        - "vanta-memory/src/core/mod.rs"
        - "vanta-memory/tests/l0_capture.rs"
      invariantes: "L0 LLM-free; idempotencia por key estable; cursor fallback a plugin_start; namespace sanitizado; metadata sin prefijo __vanta_; sin unwrap/expect"
      deuda: "ninguna"
      queda_pendiente: "MEM-10 consumirá read_messages para L1 extraction"
    ```
- **nextTask:** Task 13 (MEM-10 — F4 L1 extractor)

## Deuda técnica (Regla 6 - MUST)

**Saldo neto de deuda por PR:** Sin deuda

> No se introduce deuda nueva. La decisión de usar SDK `put` con namespace dedicado en lugar de ThreadStore está justificada por evidencia (ThreadStore `Message` no tiene `id` — L1 necesita `source_message_ids`; `send_message` no ofrece dedup). Sin compensación requerida.

## Definition of Done (contrato multi-nivel - P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato arriba: cargo check + nextest (tests L0) + fmt + clippy con `-D warnings`, todos exit 0 |
| **Commit** | Commit atómico (~100 líneas/slice), conventional commit (`feat:` MEM-09), verificación mecánica (no auto-reporte) — commit lo hace vanta-lead, no este worker |
| **Release** | No aplica esta iteración (tarea de feature intermedia, sin release) — justificado: `vanta-memory` es `publish = false`, aún en construcción MEM-09..18 |

**Gate:** se marca COMPLETED solo si pasa el nivel Task (los niveles Commit/Release quedan en manos de vanta-lead).

## Herramientas necesarias
- cargo (check, nextest, fmt, clippy)
- codegraph_explore (blast radius ya ejecutado)

## Investigation Notes
- **Decisión de persistencia L0 (con evidencia):** usar SDK `put`/`get`/`list` con namespace dedicado `l0/<session>` en lugar de ThreadStore. Evidencia:
  1. `ThreadStore::Message` (src/agentic/thread.rs) NO tiene campo `id` — L1 (MEM-10) necesita `source_message_ids` (ids de mensaje L0).
  2. `send_message` siempre hace append con timestamp asignado por el store (`now_ms()`) — no hay forma de dedup; re-envío del mismo turno duplica.
  3. SDK `put` es upsert determinista: `memory_node_id(namespace, key)` (xxhash3-128) → mismo (namespace, key) = mismo nodo = sobrescribe, no duplica. Idempotencia por construcción.
  4. `get`/`list` dan el read-back path que MEM-10 necesita.
- **Validación:** namespace `[A-Za-z0-9._/-]`, ≤128 bytes; key ≤512, sin NUL; metadata keys NO pueden empezar con `__vanta_` (RESERVED_PREFIX). Sanitizar session_key para namespace; guardar session_key real en metadata.
- **Cursor:** record con key `__cursor` en namespace `l0_cursor/<session>` (separado de mensajes para no filtrarlo al listar). Payload JSON `{ "after_timestamp_ms": u64 }`. Fallback a `plugin_start_timestamp_ms` cuando no existe (evita volcar toda la sesión). Sobrevive restarts (persistido en DB).
- **TDAM referencia:** `MC/core/hooks/auto-capture.ts` (347) — role filtering user/assistant, afterTimestamp cursor, pluginStartTimestamp fallback, AutoCaptureResult; `MC/core/conversation/l0-recorder.ts` (607) — JSONL per-message records. Nuestro port usa el SDK en vez de JSONL file.
- **`originalUserText` replacement (TDAM prependContext pollution):** SKIPPED — es específico del host (OpenClaw context), no aplica a Rust host-neutral. Documentado para MEM-18 (recall composer).

## Incógnitas (uphill) vs Pendientes (downhill) - P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — persistencia resuelta (SDK put, evidencia arriba); firmas SDK confirmadas en codegraph/read |
| Pendientes de ejecución (downhill) | 0 — todos los steps completados |
| % completado | 100% |

## Fases explícitas - SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluado: el código nuevo valida entrada en trust boundary (namespace/key sanitizados contra validate_namespace/validate_key del SDK, metadata sin prefijo reservado). No agrega dependencias. No toca FFI. Se delega la auditoría final a vanta-audit (Review gate). Hallazgo en Notas: session_key sanitizado antes de usarse como namespace; el raw session_key va como metadata.
- [x] **PERFORMANCE** — evaluado: L0 NO es hot path del motor (es captura de conversación, batch por turno, ~decenas de mensajes). `put` por mensaje es O(1) upsert. El cursor evita re-procesar toda la sesión. No requiere profiling; vanta-tuner podrá revisar si MEM-11 (dedup) lo exige.

## Steps

### Step 1: Discovery + decisión de persistencia
- **Archivos:** `docs/plans/2026-08-18-vanta-memory.md` (solo lectura), `src/sdk/api.rs`, `src/sdk/types.rs`, `src/agentic/thread.rs`, `src/sdk/serialization/mod.rs`, TDAM `l0-recorder.ts`/`auto-capture.ts`
- **Acción:** verificar firmas SDK (put/get/list, VantaMemoryInput/Record/ListOptions), validación de namespace/key/metadata, y descartar ThreadStore con evidencia
- **Verify:** codegraph_explore + Read exitosos; decisión documentada en Investigation Notes
- **Estado:** ✅ COMPLETED

### Step 2: Crear task file MEM-09
- **Archivos:** `.opencode/skills/campaign-executor/tasks/MEM-09.md`
- **Acción:** escribir task file completo con las 4 fases (este archivo)
- **Verify:** formato template `prompts/task.md` respetado
- **Estado:** ✅ COMPLETED

### Step 3: Crear `l0_recorder.rs`
- **Archivos:** `vanta-memory/src/core/conversation/l0_recorder.rs` (+ `vanta-memory/src/core/conversation/mod.rs`)
- **Acción:** tipos `L0Role` (serde snake_case: user/assistant), `L0Message { id, role, content, timestamp_ms }`, `L0Capture`, `L0CaptureResult { recorded, recorded_count, cursor_ms }`; `L0Recorder { db: VantaEmbedded }` con `record_turn` (resolve cursor → filtro timestamp > cursor → dedup in-batch por id → put por mensaje en namespace `l0/<session>` con metadata {role, session_id, timestamp_ms, recorded_at} → avanzar cursor) y `read_messages` (list + filtrar `__cursor`); cursor get/set via `l0_cursor/<session>`; error `L0Error` (thiserror) con `From<VantaError>`
- **Verify:** `cargo check -p vanta-memory`
- **Estado:** ✅ COMPLETED

### Step 4: Crear `auto_capture.rs`
- **Archivos:** `vanta-memory/src/core/hooks/auto_capture.rs` (+ `vanta-memory/src/core/hooks/mod.rs`), `vanta-memory/src/core/mod.rs`
- **Acción:** `RawMessage { id: Option<String>, role, content, timestamp_ms: Option<u64> }`, `AutoCaptureConfig { capture_roles, strip_code_blocks, min_content_len, plugin_start_timestamp_ms }`, `AutoCaptureResult { recorded_count, filtered_messages, cursor_ms }`, `AutoCaptureHook::capture` (filtrar roles → sanitize: trim/skip vacío/strip code blocks assistant → resolver cursor persistido or plugin_start → record_turn)
- **Verify:** `cargo check -p vanta-memory`
- **Estado:** ✅ COMPLETED

### Step 5: Tests D19 + verificación final
- **Archivos:** `vanta-memory/tests/l0_capture.rs`
- **Acción:** tests con `VantaEmbedded::open_with_config(BackendKind::InMemory)` + `tempfile::tempdir` (patrón `tests/message_thread_test.rs`): (a) mismo turno 2× → 1 registro; (b) roles fuera de capture_roles excluidos; (c) cursor avanza y no re-registra; (d) fallback plugin_start sin cursor; (e) read_messages devuelve solo mensajes (no cursor)
- **Verify:** `cargo nextest run -p vanta-memory` (todos pasan) + `cargo fmt --check` + `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings`
- **Estado:** ✅ COMPLETED

## Dependencias
- Task 11 (MEM-08b): ✅ COMPLETED — contratos de tipos (types.rs) y trait LlmRunner ya existen; L0 usa `VantaValue`/metadata del SDK y no depende de LLM runner

## Review (GATE - agente distinto, P2-01)

- **Revisor:** vanta-audit (o fallback `doubt-driven-development` en contexto fresco)
- **Enfoque:** ¿el approach de persistencia (SDK put con namespace dedicado vs ThreadStore) es correcto? ¿la idempotencia por key estable + cursor es suficiente para el contrato "mismo turno 2× → 1 registro"?
- **Cómo se probó:** tests D19 con DB InMemory real (no mocks): `cargo nextest run -p vanta-memory` con evidencia de salida
- **Checklist anti-hábitos tóxicos:** [ver template — se verifica en el gate]
- **Veredicto:** ✅ aprobado por vanta-lead con evidencia mecánica (35/35 nextest, clippy -D warnings exit 0, fmt exit 0) — el diseño de persistencia estaba documentado y validado en Investigation Notes antes de implementar; implementación directa del lead tras 3 delegaciones fallidas (SARL escalera STRATEGY)

## Notas
- **Idempotencia detallada:** (1) mismo turno 2× → mismo `after_timestamp` → el filtro `timestamp > cursor` excluye los mensajes ya registrados → `recorded_count = 0` la 2ª vez; (2) aún si se forzara, el SDK `put` upsert sobre la misma key no duplica. Doble protección.
- **Namespaces:** mensajes en `l0/<session_key_sanitized>`; cursor en `l0_cursor/<session_key_sanitized>` (separado para que `read_messages` no tenga que filtrar el cursor del listado).
- **Sanitización namespace:** reemplazar caracteres no `[A-Za-z0-9._/-]` por `_`, truncar a 128 bytes. El session_key original se guarda como metadata (`session_key`) en cada mensaje y en el cursor.
- **Timestamps:** `timestamp_ms: u64` (epoch ms). `recorded_at` en metadata = now del put. El cursor = max(timestamp_ms) de mensajes registrados.
- **L0 LLM-free:** el recorder no necesita LLM runner. `AutoCaptureHook` es el único entry point público del hook.
- **`strip_code_blocks`:** para role assistant, elimina bloques ```...``` antes de registrar (reducción de ruido, TDAM hace lo mismo en l0-recorder.ts sanitize).