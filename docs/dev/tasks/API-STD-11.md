# Task API-STD-11 — INDIVIDUAL (10/11) vanta-proxy

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (inline, sin subagentes)
> **Fecha:** 2026-09-24

## 1. Objetivo + contrato

Ficha individual proxy: funcionamiento + uso + código + veredicto.

## 2. Funcionamiento

Proxy LLM transparente: forward bytes al upstream + opt-ins vía `config.toml` (D31 TOML+serde). 8 endpoints lógicos (`PROXY.md:27-30`, router `:741-772`). Auth por headers en pipeline (`:185,257,297,306,337,430`) + rate-limit sliding-window por spaceId×model (`:355`).

## 3. Uso (ejemplo mínimo)

```toml
# vanta-proxy/config.toml
[upstream]
url = "http://127.0.0.1:8096"  # ⚠️ default = self-loop, ver §4
```

```bash
curl localhost:8096/v1/chat/completions -H 'Authorization: Bearer ...' -d '{...}'
```

## 4. Código (re-verificado 2026-09-24, lectura directa)

- **X1 `/snapshot` sin auth CONFIRMADO:** handler `:816-840` no recibe headers ni llama `authenticate` (vs `session_advance` que SÍ (`:789`), vs pipeline `:297`). Expone turns/sessions/writeback/rate_limit/cost + `default_budget_usd/enforce` (`:829-838`). Singular vs `/api/v2/snapshots` plural.
- **X2 Verbo en path CONFIRMADO:** `POST /session/advance` (`:745`, doc PRX-01 `:778`).
- **X3 `spaceId` camel + `/v1/responses` sin prefijo CONFIRMADO:** `:747-754` vs resto snake_case; `:756-768` models/messages con doble forma, responses sin ella.
- **X4 Self-loop default CONFIRMADO:** `url 127.0.0.1:8096` (`config.rs:272`) = `DEFAULT_PORT` (`:12`), con test que lo bendice (`:288-289`) + guard `points_at_self` (`:245-251`).
- **X5 `ttl_secs=0` + rate unused CONFIRMADO:** default 0 (`:129`, campo `:117`) = nunca expira (convención invertida); `rate_limit_per_minute` default 60 (`:201`, campo `:193`; auditoría previa: parsed-but-unused en proxy vs governor real en server).

## 5. Veredicto + implicaciones

5/5 confirmados (el más grave: X1 auth). Propuestas 15: auth en `/snapshot` YA (`feat!:` seguridad), plurales sin verbos, `space_id` snake, default upstream vacío + error claro, `ttl_secs=0` = desactivado, rate-limit en UN punto, Unix-socket solo si benchmark (Regla 9). Blast radius: `server.rs` + `config.rs` + `config.toml` + callers LLM.

## 6. DoD

- [x] Contrato ✅ · Task file sync · Recitation: proxy ficha completa

## Context Save Point

API-STD-11 DONE. Next: API-STD-12 (vanta-memory). Deuda: ninguna. WIP: ninguno.
