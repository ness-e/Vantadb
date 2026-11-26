---
title: Hacer observable el descarte L1 por `type` inválido y documentar el set válido
kind: task
description: "Contador de fallos de decodificación L1 (invalid_type vs unparseable) sin cambiar el reader + los 7 valores válidos en VANTA_MEMORY.md"
id: FIND-259
type: fix
state: IN_PROGRESS
plan: docs/dev/plans/2026-11-23-release-v0.10.0-gates.md
depends_on: FIND-699
---

# FIND-259 — Descarte L1 por `type` inválido: superficie de diagnóstico

## Objetivo

Un registro L1 con `type` fuera de los siete valores válidos no se puede decodificar como
`MemoryRecord`. El reader lo reinterpreta con el fallback `sdk_to_memory_record` (el registro
sobrevive, pero como `episodic` con `content` = payload crudo), así que `memory_recall` nunca lo
devuelve como se escribió y el único rastro es un `tracing::debug!` en la ruta `core_search`.
Nadie puede distinguir "escribí mal el `type`" de "este namespace tiene payloads planos".

Entregables:

1. Contador de fallos consultable en el crate, **aditivo** (misma salida del reader).
2. Los 7 valores válidos enumerados en `docs/api/VANTA_MEMORY.md`.

## Contrato

```
cargo nextest run --profile audit -p vantadb --lib -E 'test(l1)' --build-jobs 2
cargo nextest run --profile audit -p vanta-memory --test l1_type_diagnostics --build-jobs 2
cargo nextest run --profile audit -p vantadb-mcp -E 'test(memory_put)' --build-jobs 2   # regresión FIND-699
cargo clippy -p vanta-memory --all-targets -- -D warnings
```

Test que demuestra el contador: N payloads con `type` inválido → `decode_stats().invalid_type == N`
y `perform_auto_recall` devuelve el mismo resultado que antes (no cambia el comportamiento).

## Impacto mapeado (Regla 0)

| Archivo | Rol | Leído |
|---|---|---|
| `vanta-memory/src/core/record/l1_reader.rs` | `read_namespace_records_impl` (decode + fallback), `sdk_to_memory_record` | SÍ (completo, 670L) |
| `vanta-memory/src/core/abstractions/types.rs` | `MemoryType` (serde `snake_case`) + `MemoryRecord` wire | SÍ (completo) |
| `vanta-memory/src/core/hooks/auto_recall.rs` | consumidor de recall (`read_active_session_records`, `read_scoped_records`); `core_search` decodifica por su cuenta (`:1042`) | SÍ (secciones del path) |
| `vantadb-mcp/src/handlers/tools.rs` | `memory_recall` → `perform_auto_recall_governed` con `core_search: false` | SÍ (handler) |
| `docs/api/VANTA_MEMORY.md` | destino de la enumeración (§Contratos clave) | SÍ (§Contratos clave y adyacencias) |
| `git show 61c2be235` | FIND-699: scoping `agent_id`/`team_id` en el fallback — no romper | SÍ (commit completo) |

**Referencias entrantes:** `read_namespace_records` (raw: dedup, consolidación, decay, cascade
forget) y `read_active_namespace_records` (recall: `perform_auto_recall`, `read_scoped_records`)
→ 2 callers directos del impl, ambos en el mismo fichero (codegraph). Ruta MCP `memory_recall` y
SDK consumen esas dos funciones, nunca el impl.

**Referencias salientes:** `vantadb::sdk::{MemoryListOptions, MemoryListPage}`, `sdk_to_memory_record`,
`L1Error`.

**Veredicto:** blast radius cerrado. El contador es aditivo: no toca el `Ok`/`Err` del decode, ni
el orden, ni el contenido devuelto. `sdk_to_memory_record` (FIND-699) queda intacto.

## Diseño

- `L1DecodeFailure { InvalidType, Unparseable }` — el `type` presente pero fuera de `MemoryType`
  es un bug de escritura; la ausencia de `type` (o payload no-JSON) es el caso documentado de
  payload plano (ADRs/docs, FIND-699).
- `L1DecodeStats { invalid_type, unparseable }` + `decode_stats()` / `reset_decode_stats()`
  (dos `AtomicU64`, proceso-wide).
- `classify_decode_failure` reusa el `Deserialize` de `MemoryType` en vez de hardcodear la lista.

## SDP

SKILLS: documentation-skill, documentation-and-adrs, api-and-interface-design,
incremental-implementation, test-driven-development (SDP v3, phase=BUILD).

## Steps

- [x] DISCOVERY: leer `l1_reader.rs`, `types.rs`, docs, commit `61c2be235`; blast radius con codegraph
- [x] RED: `vanta-memory/tests/l1_type_diagnostics.rs` (contador inexistente → no compila)
- [x] GREEN: `decode_stats()` / `reset_decode_stats()` + clasificación en el fallback
- [x] Doc: enumeración de los 7 valores + semántica de los contadores en `VANTA_MEMORY.md`
- [x] VERIFY: contrato + regresión FIND-699 + clippy + fmt + docs + `ocr-review.ps1`
- [x] Review P2-01 (`vanta-review`) + commit local

## Verificación

| Comando | Resultado |
|---|---|
| `cargo nextest run --profile audit -p vantadb --lib -E 'test(l1)' --build-jobs 2` (contrato) | ✅ 2/2 |
| `cargo nextest run --profile audit -p vanta-memory --test l1_type_diagnostics --build-jobs 2` | ✅ 1/1 |
| `cargo nextest run --profile audit -p vanta-memory -E 'test(recall) or test(l1) or test(heat) or test(decay) or test(semantic)'` | ✅ 144/144 |
| `cargo nextest run --profile audit -p vantadb-mcp -E 'test(memory_put)'` (regresión FIND-699) | ✅ 1/1 |
| `cargo clippy -p vanta-memory --all-targets -- -D warnings` | ✅ |
| `cargo fmt -p vanta-memory -- --check` | ✅ |
| `scripts/validate-docs-coverage.ps1` | ✅ 0 gaps |
| `node scripts/docs/check-docs.mjs` / `check-links.mjs` | ✅ exit 0 / 0 links markdown rotos |
| `pwsh dev-tools/ocr-review.ps1` + `ocr delegate rule` | ✅ sin Critical/High en los 2 `.rs` |
| `node scripts/docs/gen-index.mjs --check` | ⚠️ drift PRE-EXISTENTE: falla igual sin este task file (verificado moviéndolo); lo regenera el orquestador al cerrar la ola |

**Comportamiento medido (no cambiado):** un envelope con `type:"fact"` vuelve del reader como
`episodic` con `content` = payload crudo; `recall` con query sin solape → `None` (modo
invisible del finding); con solape → hit con el payload crudo. Antes y después del contador,
idéntico.

## Review (GATE P2-01)

**Veredicto: ✅ APPROVE** — `vanta-review`, contexto fresco (reviewer ≠ autor), 2026-11-24.

Invariantes verificadas leyendo el diff: aditividad (`Ok`/`Err`, orden y `push` intactos;
el borrow de `&record.payload` termina antes del move al fallback); `sdk_to_memory_record`
(FIND-699) sin tocar, scoping `agent_id/team_id/user_id` intacto (`l1_reader.rs:259-261`);
clasificación coherente con el enum (7 variantes, `snake_case`) y sin `type`/no-JSON →
`Unparseable`; sin `unwrap`/`expect`/`unsafe` fuera de tests; `Relaxed` sin consecuencia real;
los dos claims verificables del doc confirmados (`auto_recall.rs:1043` usa `tracing::debug!`
en la ruta `core_search`; `vantadb-mcp/src/handlers/tools.rs:2404` fija `core_search: false`).

Aplicado tras el review (opcionales, no bloqueaban):

- `L1DecodeFailure` pasó a privado (superficie pública sin consumidor).
- El test de `types.rs` ahora cierra con un `match` exhaustivo sobre `MemoryType`: una 8ª
  variante rompe la compilación del test → la lista documentada no puede derivar.
- Doc: `type` presente pero inválido incluye `null`/`""`/`5`; los contadores cubren también
  las lecturas raw (dedup, dream, decay, cascade-forget), no sólo recall.
