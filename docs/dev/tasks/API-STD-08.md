# Task API-STD-08 — INDIVIDUAL (7/11) MCP

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (inline, sin subagentes)
> **Fecha:** 2026-09-24

## 1. Objetivo + contrato

Ficha individual MCP: funcionamiento + uso + código + veredicto.

## 2. Funcionamiento

Cara LLM (stdio, Anthropic MCP): tools memory/graph/system/collection/skill en `tools.rs` (~2000 líneas), servidas vía `vantadb-server --mcp` (hijo de `vanta-cli server --mcp`). Consume: opencode local (`opencode.jsonc` + `vanta-mcp-local.ps1`).

## 3. Uso (ejemplo mínimo)

Tool-call `memory_put {namespace, key, payload}` → `{success, record}`; `memory_search {namespace, query_vector…}` → hits con `score`.

## 4. Código (re-verificado 2026-09-24, grep directo)

- **M1 Alias doble CONFIRMADO:** `search_memory` (`:373`, def) y `memory_search` (`:309`) → mismo `dispatch_search_memory` (`:1691,1695`). Whitelist incluye ambos (`:1073-1074`).
- **M2 Doble listado CONFIRMADO:** `memory_list_namespaces` (`:218`) vs `collection_list` (`:703`) (+ whitelist `:1069,1079`); `collection_*` vs `memory_*` vs `skill_list` singular.
- **M3 `thread_id: number` CONFIRMADO + MATIZADO:** schema `number` (`:631`, required `:632`) vs resto u128-string; PERO mitigación AUD-050 existe (`:1970-1973`, tipado erróneo ya no dice "Missing"). Falta: unificar tipo, no solo mensaje.
- **M4 `bulk_import_stream` bypass CONFIRMADO por diseño:** doc propia (`:991` "Bypasses per-record validation… NOT addressable via memory_get/memory_list").
- **M5 `query_iql` string crudo** (`:269` def, dispatch `:1641` — auditoría previa; sin `validate_*` previo).
- **M6 Errores string vs tipados** (auditoría previa): `error_content` (`validation.rs:477-479`) vs `McpError::from_domain` (`error.rs:68-95`); `skill_extract` degrada (`MCP.md:412`).

## 5. Veredicto + implicaciones

6/6 (M3 matizado: mensaje ya mitigado, tipo pendiente). Propuestas 15: 1 nombre canónico por tool (alias → 1 + redirección documentada), `thread_id` u128-string, JSON Schema estricto (Schemars) en todas, `query_iql` con `invalid_params` temprano, errores tipados siempre, separar prompts/resources/tools per spec. Blast radius: `tools.rs` + MCP local opencode (re-listar tools tras cambio).

## 6. DoD

- [x] Contrato ✅ · Task file sync · Recitation: MCP ficha completa

## Context Save Point

API-STD-08 DONE. Next: API-STD-09 (IQL). Deuda: ninguna. WIP: ninguno.
