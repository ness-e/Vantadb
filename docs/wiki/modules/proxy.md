---
title: "Módulo: proxy (`vanta-proxy`)"
kind: concept
status: active
description: "Proxy transparente de protocolos LLM con captura de memoria, guardrails y gobernanza: estado frozen, estructura y fuentes."
---

# Módulo: proxy (`vanta-proxy`)

Responsabilidad: forwarding transparente de wire protocols LLM con captura/inyección de memoria. Referencia: [PROXY](../../api/PROXY.md).

## Qué es

`[HECHO]` Forwarding de wire protocols — OpenAI Chat Completions `/{agent}/{space_id}/v1/chat/completions`, Anthropic Messages `/{agent}/{space_id}/v1/messages`, subset genérico `/v1/responses` ([`vanta-proxy/src/lib.rs:4-10`](../../../vanta-proxy/src/lib.rs#L4-L10)) — con módulos que interceptan y transforman el tráfico: captura e inyección de memoria, guardrails, routing, traducción y gobernanza (no es un proxy puramente passthrough).

`[HECHO]` Submódulos (`vanta-proxy/src/lib.rs:11-35`): `auth`, `cache`, `capture`, `config`, `context`, `cost`, `envelope`, `error`, `forward`, `governance`, `guardrails`, `handlers/{anthropic,openai,responses,auxiliary}`, `inject`, `langfuse`, `mem_command`, `memory_tools`, `rate_limit`, `redact`, `report`, `routing`, `server`, `session` (+`session/claude_code`), `sse_intercept`, `translate`, `writeback`; binario `src/bin/vanta-pii-audit.rs`.

## Estado

`[HECHO]` **FROZEN hasta 1.0** (DIST-18 re-scoped): `publish = false`, solo build desde fuente, sin canal de distribución (SURFACE_MANIFEST). `[INFERENCIA]` En la práctica: no es una superficie soportada para usuarios finales en 0.9.0; su uso real es interno/experimental.

## Fuentes

`vanta-proxy/` · [PROXY](../../api/PROXY.md) · [SURFACE_MANIFEST](../../api/SURFACE_MANIFEST.md)
