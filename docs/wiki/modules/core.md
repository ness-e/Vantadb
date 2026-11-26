---
title: "Módulo: core (`vantadb` SDK + transversales)"
kind: concept
status: active
description: "SDK embebido, tipos fundamentales y módulos transversales del crate vantadb: responsabilidad, entradas públicas y fuentes."
---

# Módulo: core (`vantadb` SDK + transversales)

Responsabilidad: superficie embebida `sdk`, tipos de dominio y los módulos transversales que no tienen página propia.

## Entradas públicas

`[HECHO]` `Embedded` ([`src/sdk/builder.rs:15`](../../../src/sdk/builder.rs#L15)), `MemoryInput`/`MemoryRecord` ([`src/sdk/types/record.rs:184`](../../../src/sdk/types/record.rs#L184),[`:324`](../../../src/sdk/types/record.rs#L324)), `UnifiedNode` ([`src/node/unified.rs:15`](../../../src/node/unified.rs#L15)), `Config` ([`src/config.rs:719`](../../../src/config.rs#L719)), `Error` ([`src/error.rs:137`](../../../src/error.rs#L137)). Re-exportaciones completas en `src/lib.rs:190-222`; snapshot mecánico en `tests/api/public-api.txt`.

## Transversales agrupados (declaración en `src/lib.rs`)

| Grupo | Módulos | Línea `src/lib.rs` |
|---|---|---|
| Config/errores/tipos | `config`, `error`, `serialization`, `metadata`, `schema`, `binary_header` | L92, L108, L144, L126, L157, L76 |
| Migración/versión | `migration`, `schema`, `sync_ext` | L129, L157, L151 |
| Dominio agente | `entity`, `agentic`, `wiki`, `skills` | L106, L67, L176, L150 |
| Seguridad | `crypto` (feature `encryption`), `audit`, `attestation` | L64, L72, L70 |
| Observación | `metrics`, `telemetry` (opt-in TEC-09), `hardware`, `embedding_health` | L127, L153, L117, L104 |
| CLI/TUI | `cli`, `cli_complete`, `cli_handlers`, `cli_server`, `console`, `tui` | L82-100, L168 |
| Infra | `utils`, `accumulator`, `api`, `llm`, `integrations`, `transcript` | L169, L66, L68, L122, L120, L187 |
| FFI/testing | `python` (feature `python_sdk`), `testing` (feature `failpoints`) | L138, L241 |

`[INFERENCIA]` La tabla es asignación editorial primaria: `schema`/`binary_header` también sirven a [storage](storage.md); `api` solo contiene hoy `scores` (`src/api/scores.rs`).

## Contratos y límites

`[HECHO]` `Config` gobierna ruta de almacenamiento, backend, memoria (`memory_limit`, `rss_threshold`, eviction weights), read-only, mmap (`mmap_hnsw`, `force_mmap`, `prefetch_mode`), embeddings (`embedding_provider`, `openai_*`, `local_model_path`, `strict_embeddings`) y cuota `max_records_per_namespace` (`src/config.rs:719-789`). `[HECHO]` Errores: enum único `Error`; convención de variantes anti-stutter en ADR-0041. `[HECHO]` `telemetry` es opt-in anónimo (doc del módulo, `src/lib.rs:152-153`).

## Tests existentes (registrados en `Cargo.toml [[test]]`)

`[HECHO]` `regression_certification`, `basic_node`, `integration`, `memory_api`, `memory_export_import`, `sdk_serialization`, `version_coherence`, `cli_tests`, `query_result_*`, `edge_cases`, entre los tests registrados en `Cargo.toml`.

## Fuentes

`src/lib.rs` · `src/sdk/` · `src/config.rs` · `src/{entity,agentic,wiki,skills,crypto,audit,attestation,metrics,telemetry,hardware,utils}/` · `src/cli_handlers/` · [EMBEDDED_SDK](../../api/EMBEDDED_SDK.md) · [STORAGE_FORMATS](../../api/STORAGE_FORMATS.md)
