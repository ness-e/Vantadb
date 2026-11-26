# PRX-12 Compat Fixtures — coding-agent wire shapes

> Provenance: **representative, protocol-faithful fixtures — NOT live captures.**
> No live traffic from Claude Code / Codex / OpenCode exists in this repo, so per
> the task stop condition ("fixtures sin fuente real → no inventar") these files do
> NOT claim to be verbatim recordings. Each file mirrors the documented wire shape
> of the protocol its agent speaks, and the regression suite (`../prx12_compat.rs`)
> asserts only **proxy behavior** (verbatim passthrough + route reachability), never
> upstream behavior.

## Agent → protocol mapping (fuente: docs oficiales + suites propias)

| Fixture prefix | Agent | Protocol / route | Fuente del shape |
|---|---|---|---|
| `claude_code_*` | Claude Code | Anthropic Messages `POST /v1/messages` | `proxy_wire.rs` test (b) + Anthropic Messages API (`model`, `max_tokens`, `messages`, `tools`, `tool_choice`) |
| `codex_*` | Codex CLI | Responses `POST /v1/responses` | `proxy_wire.rs` test (c) + OpenAI Responses API (`model`, `input`, `instructions`, `tools`) |
| `opencode_*` | OpenCode | OpenAI Chat `POST /v1/chat/completions` | `proxy_wire.rs` test (a) + OpenAI Chat Completions API (`model`, `messages`, `tools`, `tool_choice`) |

## Sanitización (pre-mortem #1)

- Cero secrets: no hay API keys, tokens ni `Authorization` en ningún fixture
  (`grep -riE 'sk-(ant|live)|bearer [A-Za-z0-9]{20,}|api[_-]?key' fixtures/` → 0 hits).
- IDs (`msg_*`, `resp_*`, `chatcmpl_*`, `toolu_*`, `call_*`) y timestamps son sintéticos.
- Los tests inyectan credenciales efímeras (`Bearer test-key-123`, `sk-compat-test`)
  solo en memoria, nunca en disco.

## Actualización (pre-mortem #2 — fixtures stale)

1. Ante un release de un agente que cambie su wire shape, actualizar el par
   `*_request.json` / `*_response.json` correspondiente.
2. Re-correr `cargo test -p vanta-proxy --test prx12_compat`.
3. Si el proxy necesita un cambio para seguir pasando → nuevo task (no scope creep aquí).
4. Registrar fecha de revisión en esta tabla:

| Fecha | Revisor | Cambio |
|---|---|---|
| 2026-09-09 | worker (PRX-12) | creación inicial (shapes protocolares, sin capturas live) |
