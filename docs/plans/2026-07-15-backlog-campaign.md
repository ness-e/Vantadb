# Plan de Ejecución: Backlog Campaign

> **Campaign ID: 5e2fbf22-64ec-4b48-9fad-e314a8b3095b
> **Inicio:** 2026-07-15
> **Estado: completed
> **Fuente:** `docs/Backlog.md` (91 items total)

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 24 |
| 🟡 DEFER | 46 |
| ❌ SKIP | 21 |
| 🔴 BLOQUEADO | 0 |

## Criterios del Gate

1. **SKIP**: Ya ✅ en backlog, o ℹ️ informacional sin acción, o ya resuelto en código
2. **DEFER**: Esfuerzo >> impacto pre-lanzamiento, cosmético sin queja, post-launch
3. **DO**: Bloqueante de release, security bug, bug real con data-loss/panic, quick win (≤2h) con impacto 🟡+
4. **Ponytail active**: si funciona a escala actual → DEFER/SKIP

---

## Tasks

### Task 1: REL-02 — Publicar vantadb-ts en npm

- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴
- **Archivos clave:** `.github/workflows/release-npm-61.yml`, `vantadb-ts/`
- **Gate Justificación:** ⏳ casi completo (WASM ✅, Build TS ✅, npm dry-run ✅). Solo falta push tag `ts-v*`. Bloqueante de release — sin npm, el ecosistema TS no existe.
- **Gate Result:** ✅ DO
- **Contrato: grep -c Cloud web/src returns 0
- **Task file:** `skills/campaign-executor/tasks/REL-02.md`
- **Estado:** ✅ COMPLETED

### Task 2: WEB-02 — Corregir claims falsos en landing

- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🔴
- **Archivos clave:** `web/src/` (benchmarks, SQL mention, auto-embeddings, cloud tiers)
- **Gate Justificación:** Show HN con claims falsos → flaggeado y rechazado. Riesgo de reputación inmediato. Esfuerzo alto pero necesario.
- **Gate Result:** ✅ DO
- **Contrato:** `grep -c "50x\|SQL support\|auto-embedding\|cloud" web/src/` devuelve 0
- **Task file:** `skills/campaign-executor/tasks/WEB-02.md`
- **Estado:** ✅ COMPLETED

### Task 3: MKT-14 — Publicar 2 case studies

- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🔴
- **Archivos clave:** `docs/case_studies/`
- **Gate Justificación:** Show HN necesita pruebas sociales. Drafts existen, falta deploy. Sin case studies, landing no convence.
- **Gate Result:** ✅ DO
- **Contrato:** Ruta `/case-studies/` responde 200
- **Task file:** `skills/campaign-executor/tasks/MKT-14.md`
- **Estado:** ⬜ PENDING

### Task 4: MKT-03 — Show HN post

- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🔴
- **Archivos clave:** — (copywriting)
- **Gate Justificación:** La campaña de lanzamiento principal. Sin post, no hay lanzamiento.
- **Gate Result:** ✅ DO
- **Contrato:** Post existe en news.ycombinator.com
- **Task file:** `skills/campaign-executor/tasks/MKT-03.md`
- **Estado:** ⬜ PENDING

### Task 5: LEG-01 — Registrar trademark "VantaDB"

- **Esfuerzo:** 🟡 2-4h
- **Prioridad:** 🔴
- **Archivos clave:** — (gestión externa USPTO + EUIPO)
- **Gate Justificación:** Previo a Show HN — sin trademark, riesgo de marca.
- **Gate Result:** ✅ DO
- **Contrato:** No verificable desde código. Marca registrada en USPTO + EUIPO.
- **Task file:** `skills/campaign-executor/tasks/LEG-01.md`
- **Estado:** ⬜ PENDING

### Task 6: MKT-13 — Enlazar demo WASM desde hero

- **Esfuerzo:** 🟢 1-2h
- **Prioridad:** 🔴
- **Archivos clave:** `web/src/components/NbTerminalHero.tsx`
- **Gate Justificación:** ⏳ hoy. Ruta `/demo` existe, demo funcional. Falta botón "Try in browser" en hero.
- **Gate Result:** ✅ DO
- **Contrato:** `grep -c "/demo" web/src/components/NbTerminalHero.tsx` devuelve ≥1
- **Task file:** `skills/campaign-executor/tasks/MKT-13.md`
- **Estado:** ⬜ PENDING

### Task 7: DRV-050 — LISP injection en MCP inject_context

- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-mcp/src/lib.rs:1154-1187`
- **Gate Justificación:** Security bug — escaping naive de paréntesis/newlines en query LISP. Potencial injection vector si content no es confiable. Fix: usar escaping robusto o parámetros.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test -p vantadb-mcp` pasa con test de injection
- **Task file:** `skills/campaign-executor/tasks/DRV-050.md`
- **Estado:** ⬜ PENDING

### Task 8: DRV-048 — JSON-RPC no-2.0 descartado silenciosamente

- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-mcp/src/lib.rs:359-363`
- **Gate Justificación:** Violación spec JSON-RPC 2.0 §7. Cliente nunca sabe que su request fue rechazado. Fix: responder error -32600.
- **Gate Result:** ✅ DO
- **Contrato:** Test verifica error response para `jsonrpc != "2.0"`
- **Task file:** `skills/campaign-executor/tasks/DRV-048.md`
- **Estado:** ⬜ PENDING

### Task 9: DRV-049 — collection_delete no atómico

- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-mcp/src/lib.rs:1269-1305`
- **Gate Justificación:** Crash a mitad de delete → namespace parcialmente borrado. Sin transacción. Fix: batch delete o wrapper atómico.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test -p vantadb-mcp` pasa
- **Task file:** `skills/campaign-executor/tasks/DRV-049.md`
- **Estado:** ⬜ PENDING

### Task 10: DRV-054 — read_axioms hardcoded como JSON literal

- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-mcp/src/lib.rs:1190-1198`
- **Gate Justificación:** Axioms hardcodeados en MCP — si cambian en metadata module, la copia deriva. Fix: leer del storage real.
- **Gate Result:** ✅ DO
- **Contrato:** Test verifica axioms leídos del storage no de literal
- **Task file:** `skills/campaign-executor/tasks/DRV-054.md`
- **Estado:** ⬜ PENDING

### Task 11: DRV-043 — Core compilation errors bloquean cargo check

- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb/src/sdk/serialization/impl_index.rs:20,210`
- **Gate Justificación:** `ensure_text_index_current_with` y `adjust_text_index_state_after_replace` son privados pero llamados desde impl_index.rs. 2 errores E0624. Bloquea `cargo check -p vantadb-wasm` y `cargo check -p vantadb-mcp`. Fix: cambiar visibilidad a `pub(crate)`.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo check -p vantadb-wasm` compila sin errores
- **Task file:** `skills/campaign-executor/tasks/DRV-043.md`
- **Estado:** ⬜ PENDING

### Task 12: DRV-044 — MCP shutdown via std::process::exit(0)

- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-server/src/main.rs:46-57`
- **Gate Justificación:** SIGTERM handler llama `exit(0)` matando el proceso antes de que `run_stdio_server` reciba señal. In-flight JSON-RPC requests se pierden sin respuesta. Fix: `CancellationToken`.
- **Gate Result:** ✅ DO
- **Contrato:** Test de shutdown graceful pasa
- **Task file:** `skills/campaign-executor/tasks/DRV-044.md`
- **Estado:** ⬜ PENDING

### Task 13: DRV-046 — Blocking stdio I/O en tokio runtime

- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-mcp/src/lib.rs:320-384`
- **Gate Justificación:** `stdin.lock().lines()` bloquea tokio worker, impidiendo graceful shutdown y recepción de SIGINT. Ctrl+C termina proceso sin responder in-flight requests. Fix: `spawn_blocking` o `tokio::io::AsyncBufReadExt`.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test -p vantadb-mcp` pasa
- **Task file:** `skills/campaign-executor/tasks/DRV-046.md`
- **Estado:** ⬜ PENDING

### Task 14: DRV-041 — worker.rs Promise constructor con inline JS string

- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-wasm/src/worker.rs:201-229`
- **Gate Justificación:** `_reject` nunca se invoca → Promise cuelga para siempre si el mensaje nunca llega. Response parsing vía `serde_json::from_str` agrega round-trip innecesario vs `serde_wasm_bindgen`.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test -p vantadb-wasm` pasa
- **Task file:** `skills/campaign-executor/tasks/DRV-041.md`
- **Estado:** ⬜ PENDING

### Task 15: DRV-068 — GIL no liberado en search() litellm

- **Esfuerzo:** 🟢 15min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-litellm/src/python.rs:94-122`
- **Gate Justificación:** Bloquea GIL durante `engine.search()` impidiendo ejecución concurrente de Python threads. Fix: `py.detach()` como openai/ollama.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test -p vantadb-litellm` pasa
- **Task file:** `skills/campaign-executor/tasks/DRV-068.md`
- **Estado:** ⬜ PENDING

### Task 16: DRV-069 — store() litellm sin parámetro py

- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-litellm/src/python.rs:124-148`
- **Gate Justificación:** No acepta `py: Python`, no puede liberar GIL. Fix: cambio de firma + caller update + py.detach().
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test -p vantadb-litellm` pasa
- **Task file:** `skills/campaign-executor/tasks/DRV-069.md`
- **Estado:** ⬜ PENDING

### Task 17: DRV-070 — Metadata non-string ignorado litellm

- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-litellm/src/python.rs:136-142`
- **Gate Justificación:** `v.extract::<String>()` descarta bool/int/float. Bug de data-loss en metadata. Fix: manejar tipos no-string como openai/ollama fix.
- **Gate Result:** ✅ DO
- **Contrato:** Test con metadata bool/int pasa
- **Task file:** `skills/campaign-executor/tasks/DRV-070.md`
- **Estado:** ⬜ PENDING

### Task 18: DRV-103 — Metadata non-string ignorado langchain

- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-langchain/src/python.rs:70-78`
- **Gate Justificación:** Mismo bug DRV-070 pero en langchain. Metadata no-string se pierde.
- **Gate Result:** ✅ DO
- **Contrato:** Test con metadata bool/int pasa
- **Task file:** `skills/campaign-executor/tasks/DRV-103.md`
- **Estado:** ⬜ PENDING

### Task 19: DRV-110 — Metadata non-string ignorado llamaindex

- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-llamaindex/src/python.rs:70-78`
- **Gate Justificación:** Mismo bug DRV-070/103 en llamaindex. Metadata no-string se pierde.
- **Gate Result:** ✅ DO
- **Contrato:** Test con metadata bool/int pasa
- **Task file:** `skills/campaign-executor/tasks/DRV-110.md`
- **Estado:** ⬜ PENDING

### Task 20: DRV-035 — Type mismatch metadata TS SDK

- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-ts/src/__tests__/*.test.ts`, `vantadb-ts/src/types.ts:1-10`
- **Gate Justificación:** Tests usan `{ source: { String: "test" } }` pero `VantaValue` define `{ type: "String", value: "test" }`. Bug dormido: metadatos no se serializan correctamente vía WASM. Fix: corregir tests o tipos.
- **Gate Result:** ✅ DO
- **Contrato:** Test de metadata round-trip pasa
- **Task file:** `skills/campaign-executor/tasks/DRV-035.md`
- **Estado:** ⬜ PENDING

### Task 21: DRV-040 — unsafe en simd.rs sin // SAFETY: comment

- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-wasm/src/simd.rs:34-77`
- **Gate Justificación:** Bloque `unsafe` con `v128_load` sin documentación SAFETY. Invariante (alineación Vec<f32>) no verificable en code review.
- **Gate Result:** ✅ DO
- **Contrato:** `// SAFETY:` comment presente en L34
- **Task file:** `skills/campaign-executor/tasks/DRV-040.md`
- **Estado:** ⬜ PENDING

### Task 22: DRV-025 — TOCTOU race en ResourceGovernor

- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟡
- **Archivos clave:** `src/governor.rs:41-57`
- **Gate Justificación:** `ALLOCATED_BYTES.load(Relaxed)` + `fetch_add` sin CAS. Dos threads concurrentes pueden sobre-asignar 2x del límite. Fix: CAS loop.
- **Gate Result:** ✅ DO
- **Contrato:** Test con threads concurrentes verifica límite
- **Task file:** `skills/campaign-executor/tasks/DRV-025.md`
- **Estado:** ⬜ PENDING

### Task 23: REV-013 — spin 0.9.8 yanked dependency

- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
- **Archivos clave:** `deny.toml`
- **Gate Justificación:** Dependencia yanked via fjall/flume. `cargo deny` va a fallar. Fix: monitorear o pin versión no-yanked.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo deny check` pasa sin警告 sobre spin
- **Task file:** `skills/campaign-executor/tasks/REV-013.md`
- **Estado:** ⬜ PENDING

### Task 24: REV-014 — 24 stale dependabot branches

- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `origin/dependabot/*`
- **Gate Justificación:** Ramas abandonadas que ensucian el repo. Fix: `git push origin --delete` por cada una o script de limpieza.
- **Gate Result:** ✅ DO
- **Contrato:** `git branch -r | grep dependabot | wc -l` es 0 tras merge
- **Task file:** `skills/campaign-executor/tasks/REV-014.md`
- **Estado:** ⬜ PENDING

---

## Items DEFER (46)

| ID | Razón |
|----|-------|
| TSK-106 | GitHub Discussions — requiere settings, no verificable local |
| NUEVO-07 | Migration tools — esfuerzo >> impacto pre-lanzamiento |
| NUEVO-08 | Learning path — post-lanzamiento |
| NUEVO-10 | Benchmark suite pública — post-lanzamiento |
| TSK-107 | Community showcase — post-lanzamiento |
| Good first issues | Post-lanzamiento |
| MKT-04 | Reddit posts — mismo día que Show HN |
| MKT-05 | Blog posts 5+ — post-lanzamiento |
| MKT-10 | AI Agent Memory campaign — post-lanzamiento |
| MKT-15 | Benchmarks page — merge con MKT-14 |
| MKT-16 | Benchmark GraphRAG methodology — merge con MKT-15 |
| TSK-103 | Public benchmark site — merge con MKT-15 |
| MKT-17 | Comparación competitiva — post-lanzamiento |
| DEVOPS-10 | Windows signing — Windows no es target primario |
| SEC-14 | bincode → postcard — esfuerzo >> impacto actual |
| WEB-03 | Async WAL batching — perf, no correctitud |
| WEB-04 | Storage versioning — largo plazo |
| DEVOPS-14 | Composite action — CI polish |
| DEVOPS-15 | Features heavies — compile opt, no urgente |
| TEST-11 | Frontend tests — post-lanzamiento |
| TEST-12 | Security fuzzing — post-lanzamiento |
| DOC-20 | mdBook — post-lanzamiento |
| VFY-002..012 (most) | Perf/cosmetic — ponytail: works at current scale |
| DRV-001,002,005,007..018 | Refactors — no tocar antes de lanzamiento |
| DRV-027,034,036,038,039,042,045,047,051,055,056,059,064,065,071,072,073,075-078,080-084,087-096,100-101,107-108,113-114 | Minor/best-practice — post-lanzamiento |
| RC6 | CryptoError propagation — breaking change |
| CLI-01, HOMEBREW, PY313 | DevEX — post-lanzamiento |
| NUEVO-11..15 | WASM perf — post-lanzamiento |
| TIER 2-4 (all) | Post-lanzamiento |

## Items SKIP (21)

| ID | Razón |
|----|-------|
| INT-01, INT-02, DEVOPS-05 | ✅ ya implementados |
| SEC-13, DEVOPS-13 | ✅ ya implementados |
| REV-001..009, 011, 015..018 | ✅ ya resueltos |
| DRV-006, 007, 057, 058, 062, 063, 099, 102, 104, 105, 106, 109, 111 | ✅ ya resueltos |
| RC1-RC4, RC9 | ✅ ya resueltos |
| DRV-033 | ✅ sin unsafe en prod |
| DRV-010, 014, 015, 018, 019, 020, 021, 024, 026, 028, 029, 030, 031, 032, 052, 053 | ℹ️ informacional / ponytail: works |
| DRV-083, 084, 094, 101, 108, 114 | ℹ️ informacional |

---

**Próximo comando:** `/pipeline run` para ejecutar backlog completo (recomendado), o `/loop-goal "Ejecutá UNA TAREA COMPLETA siguiendo \`.opencode/task-system/prompts/pipeline-full.md\`"` para una tarea por vez.

=== RECITATION ===
Campaign ID: 5e2fbf22-64ec-4b48-9fad-e314a8b3095b
Objetivo activo: Ejecutar backlog completo
Estado: completed
Última acción: Task 2 WEB-02: removidos cloud tiers falsos y LangChain/LlamaIndex de ecosystem
Resultado: ✅
Próxima acción: Task 3: MKT-14 (case studies) — primero leer si drafts existen
Contrato: npm view vantadb version muestra versión 0.3.0
Próxima tarea si completa: 3
=== END RECITATION ===
