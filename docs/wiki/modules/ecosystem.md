---
title: "Módulo: ecosystem (server, bindings, providers, integrations)"
kind: concept
status: active
description: "Todo lo que rodea al motor: módulo server del core, crates de servicio, hoja FFI compartida, bindings y adaptadores externos."
---

# Módulo: ecosystem (server, bindings, providers, integrations)

Responsabilidad: superficies de transporte y adaptadores fuera del núcleo embebido.

## Dentro del core

`[HECHO]` Tras feature `server`: `src/server/` (router, handlers, jwt, middleware, pagination, state, bootstrap, self_healing, telemetry, conversation, list_records, routing, errors), `src/circuit_breaker.rs`, `src/connection_pool.rs`, `src/cli_server.rs` (`src/lib.rs:78-100`,`:145-146`); `src/rbac.rs` (`pub(crate)`) aísla namespaces.

## Crates de servicio

`[HECHO]` `vantadb-server` (binario HTTP+MCP, [http](../api/http.md)/[mcp](../api/mcp.md)), `vantadb-mcp` (librería de tools), `vantadb-ffi-core` (hoja std-only con [`OpGate`/`OpGuard`](../../../vantadb-ffi-core/src/lib.rs) — su propio doc exige `cargo tree -p vantadb-ffi-core -e normal` vacío), `vanta-proxy` ([proxy](proxy.md)).

## Bindings

`[HECHO]` `vantadb-python` (PyO3→PyPI `vantadb-py`), `vantadb-node` (napi-rs→npm `vantadb-node`, publicado 0.9.0 — verificado 2026-11-06), `vantadb-wasm` (wasm-bindgen→npm `vantadb-wasm`), `vantadb-ts` (npm `vantadb`, wrapper). Matriz completa: [BINDINGS_NAMESPACES](../../api/BINDINGS_NAMESPACES.md); detalle por transporte en [api/](../api/rust.md).

## Providers e integrations (fuera del workspace)

`[HECHO]` `providers/{openai,ollama,litellm}`: crates Rust (`src/lib.rs`) + shim Python (`src/python.rs`), NO miembros del workspace (`Cargo.toml:796`), canal PyPI preparado no publicado, contrato de salida en ADR-0033. `[HECHO]` `integrations/{langchain,llamaindex,crewai,dspy,haystack,letta,mem0,ollama,openai}`: paquetes Python con `pyproject.toml`, cada uno `vantadb_<framework>/` + `tests/`; `integrations/vantadb_shared/` comparte utilidades y `integrations/test_pins.py` fija pins. `[HECHO]` ADR-0010/0016 clasifican adaptadores por lenguaje y tier; los importadores nativos Letta/Mem0/Zep viven en `src/sdk/importers/`.

## Fuentes

`src/server/` · `vantadb-server/` · `vantadb-mcp/` · `vantadb-ffi-core/` · `vantadb-{python,node,wasm,ts}/` · `providers/` · `integrations/` · ADR-0010/0016/0033/0036/0050
