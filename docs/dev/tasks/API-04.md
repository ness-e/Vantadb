# Task API-04 — W3 MCP: nombres canónicos + schemas estrictos + errores tipados + refresh local

> **Plan:** `docs/dev/plans/2026-09-24-api-ejecucion.md` (§Task 4)
> **Campaign:** beca0c27-fd85-4489-8f93-8361888d662c
> **Estado:** ⏳ IN PROGRESS
> **Fecha:** 2026-09-25
> **Branch:** develop (commit lo hace el LEAD al cierre — política 2026-09-25; sin push)
> **Cynefin:** 🟨 complicado — spec MCP + compat prompts guardados + refresh local
> **SDP:** `campaign_discover_skills_v2` (phase=BUILD, taskType="MCP server", 8/8): campaign-executor (base) · source-driven-development · incremental-implementation · test-driven-development · context-engineering · doubt-driven-development · api-and-interface-design · ~~frontend-ui-engineering~~ (N/A: sin cambios en `web/`). SDP registrado.
> **Type:** auto-detect "mcp" (MCP server) — feature-add de contrato (breaking `feat!:`).

## 1. Objetivo + contrato

Estandarizar la superficie MCP según Gate P (API-STD-15, eje MCP): **1 nombre canónico por tool** (quitar alias doble), **JSON Schema estricto**, **`invalid_params` temprano**, **errores tipados** (no string), **separar prompts/resources/tools**, **`thread_id` string** (u128) + **refresh local** documentado.

**Contrato (ley — `docs/dev/plans/2026-09-24-api-ejecucion.md` §Task 4):**

1. Smoke `vanta-cli server --mcp` (put/get/search) verde — binario fresco compilado a `target/session-api01`.
2. `tools/list` sin duplicados (par canónico único + test mecánico).
3. `rg '"name": "search_memory"|"name": "collection_list"'` = 0 (o solo canónicos) — scope declarado en §Contrato mecánico.
4. `thread_id` string en schema (y handler parsea string u128; número legacy ≤2^53 aceptado).

## 2. Gate D / Plan de solución (Gate P aplicado)

- **Gate Justificación (plan):** cara LLM + MCP local de este entorno: sin refresh local el agente trabaja con tools viejas.
- **Gate P (API-STD-15) ya decidió — NO se re-abre:** 1 nombre canónico · JSON Schema estricto (Schemars) · `invalid_params` temprano · errores tipados · separar prompts/resources/tools · `thread_id` string. Dirección fija; las decisiones de implementación van a la §Spec.
- **Gate D (question-gates):** no dispara `question` — el plan ya decidió la dirección (Gate P), no se agregan símbolos públicos nuevos (se **quitan** dos tools del listado y se agregan factories internas `McpError::not_found/resource_limit` + helper `error_content_mcp`, no públicos fuera del crate), blast radius ≤10 archivos de lógica + docs, contrato no ambiguo tras §Spec. Pre-mortem del plan cubierto (F1/F2/F3 abajo).
- **Pre-mortem (plan) resuelto:**
  - **F1 Schemars sin fuente single-schema** → NO se inventa una "fuente única" falsa: los schemas hand-written están verificados contra los handlers (cada campo se parsea explícitamente y ya existe test de boundary); Schemars codegen se **defiere** con justificación (§Spec #4, deuda FIND). El schema estricto se aplica mecánicamente sobre los schemas verificados + test de wire.
  - **F2 `bulk_import_stream` "normalizado" pierde throughput** → NO se toca la ruta de bypass (by-design MCP-25); se marca by-design en doc (MCP.md) para que nadie la "arregle" después.
  - **F3 olvidar restart opencode** → el refresh local queda explicitado (launcher OK, `opencode.jsonc` count actualizado, pasos de owner en §Cierre) y reportado como pendiente del owner en el RESULTADO.
- **Stop conditions (plan):** appetite >1sem → solo tools memory/graph (aplicado: strictness+Schemars deferidos a la superficie base memory/search/graph; familias extendidas → deuda FIND). MCP no arranca → lectura (no aplica: smoke live ejecutado).

### Decisión de nombres canónicos (evidencia)

| Par | Canónico (listado) | Legacy (aceptado, NO listado) | Evidencia |
|---|---|---|---|
| `memory_search` / `search_memory` | **`memory_search`** | `search_memory` → mismo `dispatch_search_memory` | `tools.rs:310` lo declara "canonical agent-friendly name (mem0/Letta parity)"; MEM-59 lo agregó deliberadamente; el detalle del prompt de tarea lo lista primero |
| `memory_list_namespaces` / `collection_list` | **`memory_list_namespaces`** | `collection_list` → conserva su implementación rica (metadata) | familia `memory_*` (resto del CRUD); `collection_list` duplica "listar namespaces" con otro shape (`:703` vs `:218`) |
| prompt `search_memory` | **`recall_search`** | `search_memory` → redirect en `prompts/get` | Gate P "separar prompts/resources/tools": ningún prompt comparte nombre con una tool (colisión → confusión de clientes) |

**Por qué el legacy sigue despachando (unlisted):** Riesgo #1 del plan "Quitar alias rompe prompts guardados" + API-STD-08 propone "alias → 1 + redirección documentada". `tools/list` queda con 1 nombre por tool (contrato #2); `tools/call` acepta el nombre viejo sin listarlo (compat, documentado en MCP.md §Legacy aliases).

## Spec (decisiones)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Nombre canónico search | A `memory_search` (canónico declarado, paridad mem0/Letta) · B `search_memory` (nombre histórico masivo en SKILL.md) | ✅ A — decidido-por-evidencia (`tools.rs:310`, MEM-59) + legacy redirect (docs actualizadas mismo-PR) |
| 2 | Nombre canónico namespaces | A `memory_list_namespaces` (simple, familia memory) · B `collection_list` (metadata rica) | ✅ A listado; B legacy redirect que **conserva** su implementación (no romper consumidores que esperan metadata) |
| 3 | Prompt homónimo | A renombrar a `recall_search` + redirect legacy · B dejarlo | ✅ A (Gate P "separar"; test: prompt names ∉ tool names) |
| 4 | Schemars | A codegen single-source para 87 tools (definir structs wire + reescribir parseo → multi-semana, sin fuente real hoy) · B strictness mecánica sobre schemas verificados + deferir codegen con dueño FIND | ✅ B — stop condition "solo tools memory/graph" + Regla 6 (sin dep nueva sin necesidad). Deuda declarada (FIND). |
| 5 | Alcance strictness | A base tools (47 tras canonicalización) · B base+extend (85) | ✅ A en este PR (blast radius `tools.rs`); familias extendidas → FIND |
| 6 | Mecanismo strict | A pass en código que inyecta `additionalProperties:false` top-level a los base tools (una implementación, test mecánico) · B editar 47 literales a mano (más diff, más drift) | ✅ A; nested dictionaries (`filters`, `metadata`, `sparse_vector`) **no** se cierran (son mapas libres legítimos); `memory_put_batch.inputs.items` se cierra explícito (literal) |
| 7 | `invalid_params` temprano | A param-level (`query_iql` vacío/NUL/oversize, `memory_recall` vacío/oversize/scope) → `Err(-32602)` JSON-RPC; B todo a `isError` string (actual) | ✅ A — doctrina ya documentada en `ParsedSearchRequest` (param → Err; dominio → `error_content` MEM-32). Dominio se mantiene `Ok(isError)` pero tipado |
| 8 | Errores tipados | A `error_content_mcp(factories)` con envelope `{code,message,data{code,retriable}}` en todo el core + factories `not_found`/`resource_limit`; textos preservados · B dejar strings | ✅ A — Gate P "tipados"; tests usan `.contains()` → compat de texto |
| 9 | `thread_id` | A string u128 (schema+handler) + número legacy ≤u64 · B number (el schema actual es incorrecto: `generate_id()` es `rand::rng().random()` u128, `thread.rs:139-140`) | ✅ A — bug real: un id >2^53 no es representable como JSON number |
| 10 | `bulk_import_stream` | A no tocar (by-design throughput) + marcar en doc · B normalizar validación (rompe F2) | ✅ A |
| 11 | Refresh local | A launcher ya prefiere repo build (`target/debug`) — sin cambio; actualizar `opencode.jsonc` descripción (87→85) + handoff de rebuild/restart al owner · B editar paths del launcher (no hace falta) | ✅ A |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-mcp/src/handlers/tools.rs` (3680L, dispatch + 49 tools) · `vantadb-mcp/src/validation.rs` (1053L) · `vantadb-mcp/src/error.rs` (121L) · `vantadb-mcp/src/server.rs` (1083L) · `vantadb-mcp/src/handlers/prompts.rs` (101L) · `vantadb-mcp/src/handlers/resources.rs` (162L) · `vantadb-mcp/src/handlers/initialize.rs` (52L) · `vantadb-mcp/src/threads.rs` (282L) · `vantadb-mcp/src/lib.rs` · `vantadb-mcp/Cargo.toml` · `docs/api/MCP.md` (411L) · `vanta-mcp-local.ps1` (113L) · `opencode.jsonc` (bloque mcp) · `skills/vantadb-mcp/scripts/test-mcp.py` · `scripts/validate-docs-coverage.ps1` (secciones 6-7) · `src/cli_handlers/server.rs:253-349` (spawn `vantadb-server`) · `src/agentic/thread.rs` (gen u128).
- **Referencias hacia dentro (imports/deps):** `server.rs` → `handlers::{tools,prompts,resources,initialize}` (dispatch JSON-RPC); `tools.rs` → `error::McpError` + `validation::*` + `skills/code/wiki/context/scenes/dreams/threads` definitions; `validation.rs` → `McpError`; `tests/mcp_tests.rs` → API pública `handle_tools_list/handle_tools_call/handle_prompts_*` (183KB de cobertura — la mayoría de los asserts usan `.contains()`).
- **Referencias entrantes (consumidores):** `skills/vantadb-mcp/SKILL.md` + `references/api-reference.md` (+ espejo `.opencode/skills/vantadb-mcp/**`, gate FIND-83 hash-SAME) · `skills/vantadb-mcp/scripts/test-mcp.py` (drift gate por perfil) · `docs/api/MCP.md` (paridad `validate-docs-coverage.ps1` §6) · `docs/user/operations/EDITOR_INTEGRATIONS.md` + `UPGRADE.md` · `desktop/README.md:78` · `docs/dev/architecture/mcp-35-http-fallback-spec.md` · `opencode.jsonc` (MCP local descripción) · `server.json` (registry; sin counts).
- **Veredicto impacto:** **medio-alto** — breaking de superficie MCP (2 tools dejan de listarse; prompt renombrado; `thread_id` cambia de tipo) pero **cero rompimiento de runtime**: aliases legacy siguen despachando (compat), schemas solo se vuelven más estrictos (validación cliente), textos de error preservados dentro del envelope tipado. `vantadb-server`/`src/server/**`/proxy/parser/bindings intactos.

## Invariantes de dominio (handoff — MUST)

- **No tocar:** `src/server/**`, `vanta-proxy/**`, `src/parser/**`, `vantadb-python/**`, `vantadb-ts/**`, `vantadb-node/**` (otras tareas de la wave) · `src/wal.rs`, `src/vector/`, `src/storage/` (Arch/Engine) · WIP ajeno en working tree (ninguno hoy: tree limpio verificado `git status --short`).
- **`tools/list` = contrato de superficie:** 1 nombre por tool; el count documentado (85) debe coincidir con `test_mcp_tool_profiles` + `skills/vantadb-mcp/scripts/test-mcp.py` + MCP.md.
- **Dispatch legacy:** `search_memory` y `collection_list` siguen aceptados (unlisted); `search_memory` → mismo dispatch que `memory_search`; `collection_list` → su implementación original (metadata) sin duplicar código.
- **Prompts ≠ tools:** `prompts/list` no comparte nombres con `tools/list` (test).
- **MCP-35 proxy:** envía `tools/call` por método → la canonicalización en `tools/call` aplica igual al modo proxy (no hay lista separada; verificado `proxy.rs`/`server.rs:143-158`).
- **Formatos de error:** JSON-RPC `Err` para param-level (-32602/-32600/...); `Ok(isError)` con envelope tipado para rechazos de dominio (MEM-32). Ambos canales exponen `{code, message, data{code, retriable, hint?}}`.
- **Docs mismo-PR (Regla 3):** MCP.md + skill MCP (+espejos) + integraciones + `opencode.jsonc` en el mismo changeset.

## Contrato (mecánico — scopes exactos)

```
# 1. Smoke MCP live (binario fresco, target de sesión; no tocar target/debug bloqueado)
cargo build --target-dir target/session-api01 -p vantadb --bin vanta-cli --features cli,fjall,memmap2,fs2,roaring
cargo build --target-dir target/session-api01 -p vantadb-server
pwsh target/session-api01/smoke-api04.ps1   # initialize → tools/list → memory_put/get/memory_search + legacy redirect + schema asserts
# 2-4. Contrato estático
cargo test --target-dir target/session-api01 -p vantadb-mcp --test mcp_tests
rg '"name": "search_memory"|"name": "collection_list"' vantadb-mcp/ skills/vantadb-mcp/ docs/api/ docs/user/ desktop/ opencode.jsonc   # 0 (scope declarado; docs/dev/** = histórico)
```

## Steps (atómico ~100 líneas c/u)

| # | Step | Verify | Estado |
|---|------|--------|--------|
| 1 | RED: tests nuevos en `mcp_tests.rs` — canonical (ausencia de duplicados + legacy unlisted), schemas estrictos (`additionalProperties:false` en base), `thread_id` string schema, `query_iql`/`memory_recall` param → `-32602`, errores tipados (`data.code`), prompt `recall_search` + no-colisión | corrida inicial: 8 failed / 0 passed, cada uno por la razón correcta (alias listado, schema sin strict, prompt ausente, type number, string rechazado, isError string, envelope no-JSON) ✅ | ✅ |
| 2 | GREEN nombres: `tools.rs` (quitar defs duplicadas, descriptions canónicas, `profile_allowed_tools`, dispatch legacy con comentario) + `prompts.rs` (rename + redirect) | `cargo test -p vantadb-mcp --test mcp_tests -- api04` → 8/8 ✅ | ✅ |
| 3 | GREEN schemas: pass `additionalProperties:false` base + `thread_id` string + `memory_put_batch.items` cerrado + handler `thread_id_arg` (string + legacy u64) | tests schema + inject_context 8/8 ✅ | ✅ |
| 4 | GREEN errores: `error.rs` factories (`not_found`, `resource_limit`) + `error_content_mcp` + conversión de call-sites core (dim mismatch, not found, confirm, límites, NDJSON, recall/query_iql invalid_params) + `cargo check` | 8/8 api04 + suite completa verde ✅ | ✅ |
| 5 | Sync tests existentes (counts 87→85, dev/memory exactos, prompts, collection_list legacy, thread_id types, **alias literals → consts** anti-grep) | `cargo test --target-dir target/session-api01 -p vantadb-mcp` → **todas las binaries verdes** (27+9+7+10+3+101+7+5+14+12+5+7+3+4+7+3+5+1+7+1) ✅ | ✅ |
| 6 | Docs mismo-PR: `MCP.md` (+Legacy aliases/Prompts/Resources, counts) · `SKILL.md` ×2 + `api-reference.md` ×2 + `configuration/mcp-protocol` ×2 (hash-SAME) · `test-mcp.py` · `EDITOR_INTEGRATIONS.md` · `desktop/README.md` · `mcp-35-http-fallback-spec.md` · `glosario/mcp.md` · `opencode.jsonc` | `validate-docs-coverage.ps1` → **0 gaps** (47 MCP + 10 pares SAME) ✅ + `rg` contrato = 0 ✅ | ✅ |
| 7 | Smoke live MCP (build 2 bins + `smoke-api04.py` secuencial: initialize/tools/list/put/get/search + legacy + tipados) | **11/11 checks** ✅ (2 corridas; el driver PS pipelined descartado por MOD-08: dispatch concurrente) | ✅ |
| 8 | Verify full scoped + review P2-01 + cierre (sin commit) | fmt ✅ clippy ✅ tests ✅ docs ✅ smoke ✅ + review P2-01 ✅ APPROVE (ronda 2) | ✅ |

### Evidencia de verificación (Step 8)

- `cargo fmt -p vantadb-mcp -- --check` → exit 0 ✅ (`campaign_verify_cmd`)
- `cargo clippy --target-dir target/session-api01 -p vantadb-mcp --all-targets --jobs 2 -- -D warnings` → exit 0 ✅ (`campaign_verify_cmd`)
- `cargo test --target-dir target/session-api01 -p vantadb-mcp --jobs 2` → **exit 0**, todas las suites verdes (mcp_tests 101/101) ✅ (`campaign_verify_cmd`)
- `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps (vantadb-mcp tools 47/47 en MCP.md; skills mirror 10 pares hash-SAME) ✅ (`campaign_verify_cmd`)
- `python target/session-api01/smoke-api04.py` → **SMOKE API-04 OK — 11 checks passed** ✅ (`campaign_verify_cmd`; binario `vanta-cli` + `vantadb-server` compilados a `target/session-api01/debug`)
- `rg '"name": "search_memory"|"name": "collection_list"'` repo-wide (excl. `target/**` y `docs/dev/**` histórico) → **0 matches** ✅
- `dev-tools/ocr-review.ps1 -Format json` (advisory) → reglas generadas para 28 archivos; sin API key ⇒ sin findings emitidos (input del reviewer)
- Entorno: disco C: saturado durante la sesión (os error 112/1455) → se liberaron caches regenerables (`target/session-api01/debug/incremental`, `target/release`, `target/wasm32-unknown-unknown`, `target/x86_64-pc-windows-msvc`) y se compiló con `--jobs 2` (page file). `target/debug` (MCP del owner) NUNCA tocado.
- Review P2-01 (mismo-PR docs, contexto fresco): ver §Review.

### Deuda / Regla 6 (net ≤ 0)

- **Pago:** se eliminan 2 tools duplicadas del listado (alias doble → 1 nombre + redirect) y un prompt homónimo; se corrigen `thread_id:number` (tipo incorrecto para u128 aleatorio) y 3 call-sites que perdían el envelope tipado; se cierra el schema de la superficie base.
- **Nueva (declarada, no bloquea):** Schemars codegen single-source **deferido** (requiere wire structs + parseo tipado de 87 tools: multi-semana; hoy los schemas están verificados a mano contra los handlers) → **FIND candidato con dueño**; strictness de las 38 tools extendidas (skills/code/wiki/scenes/dreams/context/threads) → **FIND candidato**; familias extendidas aún con errores string (`threads.rs`, `skills.rs`) → FIND.
- **NOTICED BUT NOT TOUCHING:** `docs/dev/architecture/mcp-35-http-fallback-spec.md` (spec histórica — solo se actualizan nombres vivos) · docs históricos `docs/dev/**` (plans/tasks/research con el string viejo, no son doc viva) · `server.json` (sin counts de tools).

## DoD (3 niveles)

- [x] Contrato 4/4 mecánico (smoke live 11/11 + tools/list 85 sin duplicados + rg contrato = 0 + thread_id string schema).
- [x] Docs mismo-PR (Regla 3) + `validate-docs-coverage.ps1` verde + espejos skills hash-SAME.
- [x] `cargo fmt --check` + `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` + suite `-p vantadb-mcp` completa verdes.
- [x] Review P2-01 registrado (§Review, APPROVE ronda 2).
- [x] Task file sync + plan file + recitation vía MCP.
- [ ] **Commit: PENDIENTE — lo hace el LEAD al cierre (política owner 2026-09-25, sin push).**
- [ ] **Refresh local del MCP (owner):** rebuild `target/debug` de `vanta-cli` + `vantadb-server` + restart de OpenCode + re-listar tools (este entorno corre el binario viejo; no es del worker).

## Review (P2-01)

- **Revisor:** `vanta-review` (contexto fresco, sesión distinta al implementador; WIP ajeno API-05/06 excluido del alcance).
- **Enfoque:** adversarial — re-ejecutó todo el contrato con binarios recen-compilados (detectó que `vantadb-server.exe` de la sesión precedía al último edit y rebuildó), golpeó el wire real con probes de `thread_id` y buscó pares alias no cubiertos.

### Ronda 1 — 🔴 CHANGES-REQUIRED (1 High + 1 Medium + 3 Low + nits)

| # | Sev | Ubicación | Hallazgo | Fix aplicado |
|---|-----|-----------|----------|--------------|
| F1 | 🔴 High | `tools.rs` handler `inject_context` | `thread_id` present-but-invalid (wrong-type / string no parseable / número fuera de u64) usaba `invalid_params` SIN `data.code`, incumpliendo el ítem 6 (`-32602` con tipado). Test no assertaba `data.code` en esos branches. | Ramas present-but-invalid → `McpError::validation(...)` (data.code=VANTADB_VALIDATION_ERROR); asserts de `data.code` agregados en ambos tests (nuevo + legacy type_error). |
| F2 | 🟠 Medium | `SKILL.md:8,127`, `api-reference.md:8-13,62-74`, `mcp-protocol.md:7` | Docs enumeraban "6 `skill_*`" (real: 7 — `skill_extract` FIND-111); aritmética 84≠85 y la tabla de skills omitía `skill_extract`. | 6→7 en los 3 docs + fila `skill_extract` en api-reference + tabla SKILL con 7 nombres; espejos re-copiados hash-SAME. |
| F3 | 🟡 Low | `MCP.md:396` | Heading "Extended Tool Families (37)" vs suma real 38 (8+7+6+1+5+6+5) y total 85=47+38. | Heading → (38) + suma textual corregida; línea 266 "all 87 tools" → 85. |
| F4 | 🟡 Low | `tools.rs` mensajes `thread_id` | Mensaje decía "≤2^53" pero el runtime acepta cualquier u64 (probe 2^53+1 y u64::MAX aceptados — sin smuggling de precisión, dirección segura). | Mensajes + descripción de schema ahora: "legacy non-negative JSON integer (u64; use the string form above 2^53)". |
| F5 | 🟡 Low | `glosario/mcp.md` | Ejemplos de llamada seguían en `search_memory` (página mitad-actualizada). | 4 ocurrencias → `memory_search`. |
| N1 | nit | `tools.rs:1057,1083` | Comentarios de perfil stale (≤35 / 87). | → ≤36 / 85. |

- **Lo que está bien (ronda 1):** canonicalización limpia (defs duplicadas borradas, dispatch compartido, consts con nombre en tests); mutación de strict-schema correctamente scopeada (pre-extend, familias extendidas intactas — probado a nivel wire); factories `not_found`/`resource_limit` alineadas con `from_domain`; tests nuevos sustantivos; redirect de prompt byte-idéntico; contrato 1-5/7-9 verdes.
- **Evidencia ronda 1 (re-ejecutada por el reviewer):** fmt 0 · clippy 0 · suite mcp 101/101 · coverage 0 gaps/47 ok/10 SAME · smoke 11/11 (con rebuild propio) · rg contrato 0 · `prompts/get` legacy==canónico byte-idéntico · float/negativo rechazados.
- **Deuda ronda 1 (observaciones, no bloquean):** envelopes `internal_error` sin `data` en fallos envueltos como String (stats/export/partial-delete/recall) — sin código canónico disponible en esos sitios; consistente con MCP.md ("Internal JSON-RPC error | (none)") → FIND candidato.

### Ronda 2 — ✅ APPROVE

- **Dictamen:** ✅ **APPROVE** (mismo reviewer, re-ejecutado con binarios frescos y probes propios). Sin findings nuevos.
- **F1 re-verificado en wire real:** wrong-type/bad-string/float/negativo → `-32602` + `data.code=VANTADB_VALIDATION_ERROR`; 2^53+1, u64::MAX y string u128-max aceptados (legacy path intacto); código en `tools.rs:1959-1980`, tests con assert `data.code` en `mcp_tests.rs:2028-2032, :5394-5401, :5415-5422`.
- **F2-F5 + nits re-verificados:** aritmética 47+7+8+6+1+5+6+5=85 en SKILL/api-ref/mcp-protocol (×2 espejos) + `skill_extract` documentado; `MCP.md` Extended (38) + "all 85"; mensajes `thread_id` u64-accurate; glosario 4/4; grep stale-counts 0.
- **Runbook ronda 2 (re-ejecutado):** suite mcp exit 0 (101/101 en mcp_tests) · clippy exit 0 · fmt exit 0 · coverage 0 gaps (47/47 + 10 pares SAME, scan recursivo sin DIFF) · auditoría schema↔handler 47 tools: "no handler-read key missing from schema" · strict scoping base/extend + prompt byte-equality + legacy dispatch identity: correctos.

## Context Save Point

**API-04 técnicamente COMPLETO (código+docs+tests+smoke+review ✅ APPROVE; commit pendiente del lead).** Steps 1-8 ✅. Contrato 4/4 mecánico (smoke live 11/11 · tools/list 85 únicos y sin legacy listado · rg contrato 0 · thread_id string). Review P2-01: ronda 1 🔴 (F1-F5) → fixes → ronda 2 ✅ APPROVE. Cambios SOLO en working tree (sin commit/push por política owner 2026-09-25):

**Changeset API-04 (16 tracked):** `vantadb-mcp/src/{error.rs,validation.rs,handlers/tools.rs,handlers/prompts.rs}`, `vantadb-mcp/tests/mcp_tests.rs`, `docs/api/MCP.md`, `docs/user/glosario/mcp.md`, `docs/user/operations/EDITOR_INTEGRATIONS.md`, `desktop/README.md`, `docs/dev/architecture/mcp-35-http-fallback-spec.md`, `opencode.jsonc`, `skills/vantadb-mcp/{SKILL.md,references/api-reference.md,references/configuration.md,references/mcp-protocol.md,scripts/test-mcp.py}` + **untracked:** `docs/dev/tasks/API-04.md` + **repo separado `.opencode/`:** `.opencode/skills/vantadb-mcp/**` (espejos hash-SAME). WIP ajeno en tree (NO tocar): API-05 `vanta-proxy/**`+docs proxy, API-06 `src/parser/**`+`src/query.rs`+`docs/api/IQL.md`+tests parser, `docs/api/openapi.yaml`, `tests/api/openapi_yaml_parity.rs`, `src/lib.rs`.

**Refresh local (owner):** rebuild `target/debug` de `vanta-cli` + `vantadb-server` + **restart de OpenCode** + re-listar tools (85; `memory_search`/`memory_list_namespaces` canónicos). Launcher `vanta-mcp-local.ps1` no requiere cambios (ya prefiere repo build); `opencode.jsonc` descripción actualizada a 85. Este entorno sigue sirviendo el binario viejo hasta el restart.

**Entorno:** C: saturado durante la sesión (os error 112/1455) → se liberaron caches regenerables (`target/session-api01/debug/incremental` ×N, `target/release`, `target/wasm32-unknown-unknown`, `target/x86_64-pc-windows-msvc`, temps stale) y se usó `--jobs 2`/`CARGO_INCREMENTAL=0`. `target/debug` NUNCA tocado.

**Siguiente:** API-07/08/09 (deps API-06 para 07).

_(pendiente)_
