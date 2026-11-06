---
title: "API MCP (`vantadb-mcp`)"
kind: reference
status: active
description: "Servidor Model Context Protocol stdio JSON-RPC 2.0: lanzamiento, herramientas, handlers y límite single-writer."
---

# API MCP (`vantadb-mcp`)

Audiencia: usuarios que conectan un IDE/agente por MCP. Referencia canónica: [MCP](../../api/MCP.md) (versión 0.9.0) y `docs/api/mcp/index.md`.

## Naturaleza y lanzamiento

`[HECHO]` Transporte cliente = stdio JSON-RPC 2.0: el cliente MCP lanza el proceso y habla por stdin/stdout (MCP.md). Matiz: el proceso sí levanta internamente un listener HTTP loopback efímero para el lado "writer" y monta el endpoint proxy `POST /api/v2/mcp/proxy` sobre el router axum ([`vantadb-mcp/src/server.rs:237-248`](../../../vantadb-mcp/src/server.rs#L237-L248)) — stdio es el transporte al cliente, no ausencia de HTTP interno. Comando canónico: `vanta-cli server --mcp --db <ruta absoluta>`; avanzado: `vantadb-server --mcp` con `VANTADB_STORAGE_PATH`. La ruta debe ser absoluta (nunca `~` en la config JSON del cliente: llega literal y falla el open — MCP.md §Client configuration).

`[HECHO]` `vantadb-mcp` es librería INTERNAL embebida en `vantadb-server` (SURFACE_MANIFEST); estructura: `server.rs`, `protocol.rs`, `handlers/{initialize,tools,resources,prompts}.rs`, `validation.rs`, `config.rs`, `proxy.rs`, `error.rs`, `metrics.rs`, más dominios `wiki.rs`, `scenes.rs`, `dreams.rs`, `threads.rs`, `skills.rs`, `temporal.rs`, `governance.rs`, `axioms.rs`, `code.rs`, `code_index.rs`, `context.rs`.

## Herramientas

`[HECHO]` Las tools se definen en `handle_tools_list` (`vantadb-mcp/src/handlers/tools.rs:88`) y se despachan en `handle_tools_call` (`:1542`). `[INFERENCIA]` Las capacidades cubren memoria, búsqueda, grafo, wiki, skills, escenas/sueños e IQL (`query_iql`) — inferido de los ficheros de dominio presentes (`scenes.rs`, `dreams.rs`, `wiki.rs`, `skills.rs`, `temporal.rs`) y de MCP.md; el listado exacto por nombre no se enumeró aquí. No extrapolar desde SDKs.

## Límites

`[HECHO]` El motor impone single-writer por base de datos: un segundo MCP sobre la misma ruta no abre el engine directamente; el fallback HTTP multi-instancia está en ADR-0036 y la spec `docs/dev/architecture/mcp-35-http-fallback-spec.md`. `[HECHO]` El envelope `structuredContent` del search MCP es el segundo transporte (junto a HTTP `SearchPageV2`) que transporta la señal de abstinencia (BINDINGS_NAMESPACES §SCH-07, tests `vantadb-mcp/tests/mcp_tests.rs`).

## Fuentes

`vantadb-mcp/src/` · [MCP](../../api/MCP.md) · [mcp/index](../../api/mcp/index.md) · ADR-0036 · [mcp-35-http-fallback-spec](../../dev/architecture/mcp-35-http-fallback-spec.md)
