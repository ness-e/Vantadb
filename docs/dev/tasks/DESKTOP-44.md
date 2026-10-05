---
title: "TASK DESKTOP-44: Validación manual Proxy Dashboard con upstream LLM vivo — prep (fix FIND-155 + guion + checklist)"
kind: task
description: "Prep owner-assisted: fix de auth del dashboard (x-vanta-user-key + acceso a la lente sin URL), guion por panel y checklist ejecutable para la sesión del owner; la sesión E2E queda pendiente (sin mocks como evidencia)"
---

# TASK DESKTOP-44: Validación manual Proxy Dashboard con upstream LLM vivo — prep (fix FIND-155 + guion + checklist)

## Metadata

- **Plan file:** [`docs/dev/plans/2026-10-04-master-plan-0.9.0.md`](../plans/2026-10-04-master-plan-0.9.0.md) (Task 34, F0 expandido)
- **Fuente:** [`Backlog`](../Backlog.md) `DESKTOP-44` (:716, Dueño: owner) + `FIND-155` (:349) + deuda DESKTOP-38 (Step 4 🟡 "Test manual con sesión proxy real NO ejecutable")
- **Esfuerzo:** 🟢 2-4h | **Appetite:** max 1d | **Prioridad:** 🟡
- **Tipo:** Fix + Docs (prep owner-assisted). No feature-add: sin símbolos públicos nuevos (helpers internos del módulo dashboard; desktop no publica API).
- **Turns estimados:** 12-20 | **Creado:** 2026-10-05 (DISCOVERY) | **last-synced:** 2026-10-05
- **Estado:** 🟡 IN PROGRESO — prep completa, sesión owner pendiente (la campaña NO se cierra: taskId `34` queda `in-progress`)
- **Campaign ID:** master-plan-0.9.0-20261004 (taskId `34`)
- **Incógnitas (uphill):** 0 abiertas — resueltas en DISCOVERY: (1) ¿mecanismo de auth? `x-vanta-user-key` contra colección `user` local ([`auth.rs`](../../../vanta-proxy/src/auth.rs) :19,:100-109, D34 sin bypass); (2) ¿primer acceso a la lente? **GAP detectado** — botón sidebar/palette condicionados a `proxyUrl()` y Settings sin sección Proxy → lente inalcanzable sin configurar (fix aplicado, ver Spec #4); (3) ¿cómo sembrar una user key para la sesión? helper ignorado `seed_live_smoke_auth_store` (API-05, camino sancionado).
- **Pendientes (downhill):** 6 steps de prep (6 ✅) + **sesión E2E owner pendiente** (contrato del plan; coordina el orquestador)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers (entrantes) | `WorkspaceShell` monta la lente (:1089) y el botón sidebar (:609-611); `CommandPalette` recibe `proxyConfigured` (:276); `i18n` diccionarios ES/EN (paridad testeada); `ProxyDashboard.test.tsx` importa helpers exportados |
| Callees (salientes) | `fetch` a `GET /snapshot` de `vanta-proxy` (proceso aparte); `localStorage` (`vanta.proxy.url`, nuevo `vanta.proxy.userKey`); `connectionPrefs` (lang) |
| Implicaciones | FIND-155 resuelto en código (pendiente validación E2E en la sesión); lente alcanzable sin configurar (primer setup); sin cambios de wire/proxy; `PROXY_URL_EVENT` intacto |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** [`ProxyDashboard.tsx`](../../../desktop/src/components/proxy/ProxyDashboard.tsx) (290L) · `ProxyDashboard.test.tsx` (51L) · [`auth.rs`](../../../vanta-proxy/src/auth.rs) (277L) · `vanta-proxy/tests/api05_snapshot_auth.rs` (183L) · `vanta-proxy/src/session.rs` (:1-240) · `vanta-proxy/src/server.rs` (:130-439, :780-947) · `vanta-proxy/src/report.rs` (245L) · `vanta-proxy/src/writeback.rs` (:27-212) · `vanta-proxy/src/main.rs` (83L) · `vanta-proxy/config.toml` (21L) · [`PROXY.md`](../../api/PROXY.md) (328L) · `DESKTOP-38.md` (72L) · `store/connections.ts` (136L) · `i18n.test.ts` (97L) · `package.json` (desktop) · secciones de `WorkspaceShell.tsx` (:280-324, :600-615, :1050-1109), `CommandPalette.tsx` (:255-294), `HelpPanel.tsx` (:90-114), `dictionaries.ts` (:400-442, :1215-1256)
- **Lecturas puntuales:** `Backlog.md` (:38, :349, :716) · `src/cli.rs` (:13-43 global `--db`, :372-394 `mcp-call`) · `Cargo.toml` (:352-354 bin `vanta-cli`) · `src/pages/Settings.tsx` (grep `proxy` = 0 matches)
- **Referencias hacia dentro:** `ProxyDashboard` ← `WorkspaceShell.tsx:63,1089`; helpers `proxyUrl`/`PROXY_URL_EVENT` ← `WorkspaceShell.tsx:63,292-296`; keys i18n `proxy.*` ← componente + paridad `i18n.test.ts:20-22`
- **Referencias entrantes a los editados:** `WorkspaceShell` ← render del shell (raíz de superficies); `dictionaries.ts` ← todos los `tt()`/`tp()`; `ProxyDashboard.test.tsx` ← vitest (`npm test`)
- **Veredicto impacto:** **MEDIO-BAJO** — cambios aditivos y localizados: (a) el dashboard gana input/storage/header de key (FIND-155); (b) el botón PROXY del sidebar deja de estar condicionado (la lente ES el formulario de primer setup; ver Spec #4); (c) i18n aditivo con paridad ES/EN. Sin cambios en `vanta-proxy` (el proxy ya exige auth), sin wire, sin contratos públicos. Gate D evaluado: **no dispara** (sin símbolos públicos nuevos; el fix está mandatado por FIND-155 y el plan).

## Contrato

> Del plan (Task 34): "sesión owner+agente ejecutada con upstream LLM vivo; TurnReports/sesiones/write-back/rate-limit verificados end-to-end (FIND-155 resuelto en la misma sesión o registrado como bloqueante con evidencia); hallazgos → FINDs; el resultado queda registrado en el task file con la evidencia de la sesión (no simulado)."
>
> **Stop conditions (plan):** "sin upstream vivo → checklist + FIND del bloqueante; no forzar mocks como evidencia."

Entregables de esta prep (owner-assisted):

1. **Fix FIND-155:** `ProxyDashboard` envía `x-vanta-user-key` (input + storage + header) con TDD scoped + gates. Incluye el fix de alcanzabilidad de la lente (Spec #4) sin el cual el formulario de setup es inalcanzable.
2. **Guion de verificación por panel** (TurnReports / sesiones / write-back / rate-limit) + **checklist ejecutable** paso a paso para la sesión del owner (abajo, §Guion).
3. **Evidencia estática read-only** (polling 5s, LS keys, endpoints que consume cada panel, auth del proxy) — sin simular la sesión.
4. **Sesión E2E pendiente:** el orquestador pregunta al owner y coordina; la campaña queda `in-progress` + recitation PARTIAL. Hallazgos de la sesión → FINDs.

**Comandos de verificación de esta prep:** `cd desktop && npx vitest run src/components/proxy/ProxyDashboard.test.tsx` · `npm test` (suite) · `npm run build` · gates docs (`check-links` / `check-docs` / `gen-index` / `validate-docs-coverage`). `npx tsc --noEmit` corre pero arrastra 6 errores **pre-existentes** en `vanta-wasm-map.ts` (→ FIND-275); los archivos del fix están limpios.

## Spec (SDD — decisiones resueltas por evidencia)

> Gate mecánico: la tarea NO agrega símbolos/contratos públicos (helpers internos del módulo dashboard; desktop no publica API) → feature-add = false. Decisiones técnicas del DISCOVERY:

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Header de auth | A) `x-vanta-user-key` (canónico D34) / B) otro | A — ✅ decidido-por-evidencia: `auth.rs:19` (`USER_KEY_HEADER`), `PROXY.md` §Authentication; el proxy rechaza cualquier otra cosa |
| 2 | Storage de la key | A) `localStorage` (`vanta.proxy.userKey`, patrón existente del dashboard/store) / B) OS keychain (dep nueva, fuera de scope) | A — ✅ decidido-por-evidencia: el desktop ya persiste `ConnectionProfile.token` en localStorage (`connections.ts:20-21`); webview Tauri local, sin contenido remoto; documentado el tradeoff |
| 3 | Requerir key en el save del form | A) Simetría con URL (key vacía permitida; el 401 muestra hint) / B) bloquear el save | A — ✅ decidido-por-evidencia: el form ya devuelve silenciosamente si falta la URL; agregar el hint de 401 da guía accionable sin bloquear el flujo de cambio de URL |
| 4 | Alcanzabilidad de la lente sin URL | A) Botón sidebar incondicional (la lente ES el setup; el dashboard ya renderiza el form) / B) sección Proxy en Settings (más código, duplica el form) / C) FIND + workaround consola | A — ✅ decidido-por-evidencia: `WorkspaceShell.tsx:609-611` + `CommandPalette.tsx:276` condicionan a `proxyUrl()` y `Settings.tsx` no tiene sección Proxy (grep = 0) → primer setup inalcanzable; el HelpPanel ya prometía un lugar de configuración (`help.proxyHint`, corregido a la lente). El fix es 1 condicional + i18n |
| 5 | Seed de la user key para la sesión | A) helper ignorado `seed_live_smoke_auth_store` (API-05) + config `[auth] db_path` / B) escribir un seeder nuevo | A — ✅ decidido-por-evidencia: `api05_snapshot_auth.rs:169-183` es el camino ya usado y verificado por el smoke de API-05; no se construye un seeder nuevo (YAGNI) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) no tocar `opencode.jsonc`, master plan (`docs/dev/plans/2026-10-04-master-plan-0.9.0.md`), `docs/pipeline-state.json`, `providers/**` (PROV-13 en vuelo); (2) **no push** (Regla 7 — commit local); (3) auth D34 intacta: el dashboard es un cliente más, sin bypass; (4) el wire `/v1` y `vanta-proxy` no se tocan; (5) no simular la sesión con mocks como evidencia (stop condition del plan); (6) la key no se loguea ni se muestra en claro en la UI (input `password`; el hint 401 no la repite).
- **Comandos de verificación:** `npx vitest run src/components/proxy/ProxyDashboard.test.tsx` · `npx tsc --noEmit` · `npm run build` — todos exit 0.
- **Deuda pendiente:** sesión E2E owner (el contrato del plan se cierra ahí); FIND-155 queda `Pendiente` en Backlog hasta la validación (la fila documenta el fix aplicado).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto negativo:** el diff **paga** FIND-155 (auth del dashboard, deuda registrada desde API-05) y el gap de alcanzabilidad del primer setup (detectado en este DISCOVERY). No introduce deps, `unsafe`, ni duplicación. Tradeoff documentado (Spec #2): la key vive en localStorage del webview — mismo modelo que el Bearer de perfiles server ya existente.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate | Estado |
|-------|------|--------|
| **Task** | Prep: fix FIND-155 + guion + checklist + evidencia estática + RESULTADO §7 | ✅ (prep completa) |
| **Task (E2E)** | Sesión owner+agente con upstream LLM vivo — **pendiente** (orquestador coordina) | ⏳ pendiente |
| **Commit** | Atómico `fix(desktop):` + `docs(desktop):`, pathspec propio (WIP ajeno fuera) | ✅ `5a8c06bd` + `a674849f` (LOCAL, sin push) |
| **Release** | n/a — desktop sin release en este bloque; sin cambio de wire/versionado | ✅ justificado |

## Herramientas necesarias

- `npx vitest` + `tsc` + `npm run build` (desktop) · `node scripts/docs/*.mjs` (gates docs) · `pwsh dev-tools/ocr-review.ps1`
- Session (owner): `cargo test -p vanta-proxy --features vantadb/fjall … seed_live_smoke_auth_store` · `cargo run -p vanta-proxy --features vantadb/fjall` · `curl.exe` · `cargo run --bin vanta-cli` (evidencia post-sesión)
- codegraph/CBM: cobertura verificada para los 5 paths tocados (sin gaps registrados)

**Skills cargadas (SDP v3):** test-driven-development · frontend-ui-engineering · source-driven-development · security-and-hardening (pinned trust boundary) · incremental-implementation · documentation-skill · context-engineering (lifecycle BUILD) · base auto: campaign-executor/progreso/ponytail.

## Investigation Notes (evidencia estática read-only — 2026-10-05)

1. **Auth del proxy (D34, sin bypass):** `USER_KEY_HEADER = "x-vanta-user-key"` (`auth.rs:19`); `authenticate()` fail-closed con missing/empty/unknown (`auth.rs:100-109`); `GET /snapshot` exige auth (`server.rs:913-916`, doc "no loopback bypass"); resolución contra colección `user` (`user_key` → `UserIdentity{user_id,is_system_admin}`, `auth.rs:130-167`). `/health` **no** requiere auth (`server.rs:870-872`) — sirve de probe de vida.
2. **Bug FIND-155 (pre-fix):** `fetchSnapshot` hacía `fetch(url/snapshot)` sin headers (`ProxyDashboard.tsx:59-63`) → 401 con el proxy post-API-05. El 401 se renderiza como "proxy no disponible: HTTP 401" (:159) sin guía.
3. **Gap de alcanzabilidad (pre-fix):** botón sidebar condicionado a `proxyUrl()` (`WorkspaceShell.tsx:609-611`); palette condicionada (`CommandPalette.tsx:276`); `help.proxyHint` decía "Configurá la URL del proxy en AJUSTES → Proxy" (`dictionaries.ts:55,:868`) pero `Settings.tsx` no tiene sección Proxy (grep `proxy` = 0) → primer setup inalcanzable.
4. **Paneles → endpoint:** los 4 paneles consumen **el mismo** `GET /snapshot`; campos: `turns` (ring cap 100, `report.rs:55`), `sessions` + `sessions_active` (TTL 30m solo pendientes, `session.rs:34-35`), `writeback.pending_labels/pending_count` (cola de escrituras **fallidas** tras retries, `writeback.rs:50-84`), `rate_limit.limit_per_minute/hits_total/degraded` (`server.rs:936-940`). Polling 5s (`ProxyDashboard.tsx:110`); LS keys `vanta.proxy.url` (:13) + `vanta.proxy.userKey` (nuevo).
5. **Pipeline del proxy (por request):** auth → budget → guardrails → **rate-limit** (sliding window por `space_id×model`, default 60/min — `config.toml:9`, `server.rs:394-406`) → mem-command → sesión (`ensure()` crea en `Team`, `session.rs:159-171`) → forward; **todo turno emite TurnReport** con el status real (incluye 401/429) (`server.rs:307-319`). `POST /sessions/advance` valida la entidad contra el store (`team`/`agent`/`task`; sin entidad → 400, `server.rs:879-907`).
6. **Seed sancionado (API-05):** test ignorado `seed_live_smoke_auth_store` recrea `CARGO_TARGET_TMPDIR/api05-smoke-authdb` con user `usr-smoke` / key `sk-smoke` (`api05_snapshot_auth.rs:159-183`); requiere `--features vantadb/fjall` (nota API-05). El proxy lee ese store vía `[auth] db_path` (`config.rs:316-331`).
7. **Evidencia de write-back real (post-shutdown):** los turns capturados se persisten en el namespace `proxy-turns` del store del proxy (`PROXY.md:223-228`); en Windows el store está byte-range-locked mientras el proxy corre (`PROXY.md:162-163`) → la lectura se hace con el proxy detenido.

## Guion de verificación (sesión owner — pendiente)

> **Owner-assisted:** el agente no puede completar la sesión (requiere upstream LLM vivo + decisión del owner). **Prohibido simular con mocks como evidencia** (stop condition del plan).

### Prerrequisitos

| # | Requisito | Cómo |
|---|-----------|------|
| P1 | Upstream LLM vivo | Opción A: Anthropic real (`upstream.url = "https://api.anthropic.com"`, key del cliente o `api_key` en config). Opción B: Ollama local (`http://localhost:11434`, endpoint OpenAI-compatible). Opción C: LiteLLM (`http://localhost:4000`) |
| P2 | User key sembrada | Paso S1 (helper API-05) |
| P3 | Proxy corriendo | Paso S2 |
| P4 | Desktop abierto | `cd desktop && npm run tauri dev` (o binario ya instalado) |

### Checklist ejecutable (paso a paso)

**S1 — Sembrar auth store + build del proxy (fjall):**
```powershell
cargo test -p vanta-proxy --features vantadb/fjall --test api05_snapshot_auth -- --ignored seed_live_smoke_auth_store
```
→ recrea `target/tmp/api05-smoke-authdb` (user `usr-smoke`, key `sk-smoke`). *(El build tarda; el seed imprime ok sin salida extra.)*

**S2 — Config + arranque del proxy** (crear `target/tmp/desktop44-proxy.toml`):
```toml
[server]
host = "127.0.0.1"
port = 8096
rate_limit_per_minute = 60

[upstream]
url = "https://api.anthropic.com"   # P1: ajustar según upstream elegido
api_key = ""                        # vacío → pasa el header del cliente (x-api-key/authorization)
forward_timeout_secs = 600

[auth]
db_path = "target/tmp/api05-smoke-authdb"
```
```powershell
cargo run -p vanta-proxy --features vantadb/fjall -- target/tmp/desktop44-proxy.toml
```
Probe de vida (sin auth): `curl.exe -s http://127.0.0.1:8096/health` → `{"status":"ok"}`.

**S3 — Tráfico de prueba (genera TurnReport + sesión + write-back):**
```powershell
# Opción Anthropic (P1-A): endpoint nativo /v1/messages
curl.exe -sS -X POST "http://127.0.0.1:8096/cc/sp-desktop44/v1/messages" `
  -H "content-type: application/json" `
  -H "x-vanta-user-key: sk-smoke" `
  -H "x-vanta-session: sess-desktop44" `
  -H "x-api-key: $env:ANTHROPIC_API_KEY" -H "anthropic-version: 2023-06-01" `
  -d '{\"model\":\"claude-3-5-haiku-latest\",\"max_tokens\":1,\"messages\":[{\"role\":\"user\",\"content\":\"ping\"}]}'
```
```powershell
# Opción Ollama/LiteLLM (P1-B/C): endpoint OpenAI
curl.exe -sS -X POST "http://127.0.0.1:8096/cc/sp-desktop44/v1/chat/completions" `
  -H "content-type: application/json" `
  -H "x-vanta-user-key: sk-smoke" `
  -H "x-vanta-session: sess-desktop44" `
  -d '{\"model\":\"<modelo-local>\",\"max_tokens\":1,\"messages\":[{\"role\":\"user\",\"content\":\"ping\"}]}'
```
**Control negativo (FIND-155):** el mismo GET **sin** key → 401; **con** key → 200:
```powershell
curl.exe -s -o NUL -w "no-key=%{http_code}\n" http://127.0.0.1:8096/snapshot
curl.exe -s -o NUL -w "with-key=%{http_code}\n" -H "x-vanta-user-key: sk-smoke" http://127.0.0.1:8096/snapshot
```

**S4 — Desktop → lente PROXY:** abrir el botón **PROXY** del sidebar (ahora incondicional); en el formulario de conexión ingresar `http://127.0.0.1:8096` + `sk-smoke` → CONECTAR.

**S5 — Verificación por panel** (esperar ≥1 ciclo de polling; el panel Conexión muestra "último poll … cada 5s"):

| Panel | Qué mirar | Evidencia a capturar |
|-------|-----------|----------------------|
| **TurnReports** | Fila del turn de S3: hora reciente, `protocolo` (`anthropic`/`openai`), modelo, `status` (200), duración ms, `space` = `sp-desktop44`. El contador "· N recientes" sube con cada request | Screenshot + output curl de S3 |
| **Sesiones** | Entrada `sess-desktop44` con stage `team` (◉) y TTL ~`30m` (baja con el tiempo). El header `x-vanta-session` es el que la crea | Screenshot (con TTL visible) |
| **Write-back** | Estado sano: `0` pendientes + "escrituras L0 esperando flush" (la cola solo crece si una escritura L0 falla tras retries). **Post-sesión** (proxy detenido): verificar la captura real en `proxy-turns` (S6) | Screenshot del panel + output S6 |
| **Rate-limit** | `límite` = 60/min (de config), `hits 429` = 0 en estado normal, `estado` = ok. **Opcional** (sin costo con Anthropic `count_tokens`): 65 requests al mismo `space`×modelo → `hits 429` > 0 y filas status 429 en TurnReports | Screenshot antes/después + loop de S5b |
| **Conexión** | URL mostrada = la configurada; "último poll" refresca cada 5s | Screenshot |

**S5b — (opcional) rate-limit hits, sin costo de tokens:**
```powershell
1..65 | ForEach-Object {
  curl.exe -s -o NUL -w "%{http_code} " -X POST "http://127.0.0.1:8096/cc/sp-rl/v1/messages/count_tokens" `
    -H "content-type: application/json" -H "x-vanta-user-key: sk-smoke" `
    -H "x-api-key: $env:ANTHROPIC_API_KEY" -H "anthropic-version: 2023-06-01" `
    -d '{\"model\":\"claude-3-5-haiku-latest\",\"messages\":[{\"role\":\"user\",\"content\":\"ping\"}]}'
}
```
→ los primeros ~60 pasan (200), el resto 429; el panel debe reflejar `hits 429` > 0. *(Con upstream local, cualquier endpoint sirve; el limiter cuenta igual.)*

**S6 — Evidencia de write-back (proxy detenido, store liberado):**
```powershell
# 1) detener el proxy (Ctrl+C) → flush de pendientes
# 2) listar los turns capturados en el store del proxy
cargo run --bin vanta-cli -- --db target/tmp/api05-smoke-authdb list --namespace proxy-turns
```
→ ≥1 registro del turn de S3 (clave `{ms}-{seq}`). *(Alternativa: `vanta-cli mcp-call --tool memory_list --args '{"namespace":"proxy-turns"}'`; el binario instalado puede estar stale — FIND-98 — por eso `cargo run` desde fuente.)*

**S7 — Captura y cierre:** adjuntar screenshots + outputs al task file (§Evidencia de sesión), registrar hallazgos como filas `FIND-*` (routing: `prompts/findings.md`), marcar FIND-155 como cerrado (o bloquear con evidencia), y `campaign_update_task_state(taskId 34, completed)` con payload review P2-01.

### Fuera de alcance de la sesión (documentado, no bloqueante)

- `POST /sessions/advance` team→agent→task: requiere entidades `team`/`agent`/`task` sembradas (el seed API-05 solo crea `user`) → no se cubre; si el owner lo quiere, es una fila FIND nueva (seed ampliado).
- Camino de **fallo** del write-back (pending > 0): requiere forzar un fallo de escritura; la sesión verifica el estado sano + la captura persistida (S6).
- `envelope`/`cost` panels no existen en el dashboard (no son parte de DESKTOP-38/44).

### Qué queda para la sesión (resumen)

1. Ejecutar S1-S6 con upstream vivo (P1) y capturar la evidencia por panel.
2. Confirmar el control negativo 401/200 del snapshot (ya probado por API-05; re-verificado end-to-end desde el dashboard).
3. Cerrar FIND-155 + campaña (review P2-01) o registrar bloqueantes con evidencia.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | **0** — auth/gap/seed resueltos en DISCOVERY |
| Pendientes de ejecución (downhill) | 6 steps (5 ✅ de prep + sesión owner pendiente) |
| % completado | ~85% (prep); sesión = contrato E2E |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — Aplica (trust boundary auth): checklist `security-and-hardening`: la key viaja solo al proxy local por header explícito (nunca query string) ✓; input `password` + sin logging ✓; el hint 401 no repite la key ✓; storage localStorage = mismo modelo que el Bearer de perfiles server ya existente (tradeoff documentado, Spec #2) ✓; sin bypass de D34 ✓; sin deps nuevas (no `npm audit` requerido) ✓.
- [x] **PERFORMANCE** — No aplica: sin hot paths (UI + fetch cada 5s ya existente); el único cambio de runtime es un header por request de polling.

## Steps

### Step 1 — Task file canónico + DISCOVERY completo
- **Archivos:** `docs/dev/tasks/DESKTOP-44.md` (nuevo)
- **Acción:** crear el task file con DISCOVERY (evidencia file:línea), contrato, guion, checklist y steps atómicos.
- **Verify:** archivo existe + `node scripts/docs/check-docs.mjs`
- **Resultado:** ✅ creado (kind `task`; gates docs al cierre)
- **Estado:** ✅

### Step 2 — RED: tests de auth del dashboard (FIND-155)
- **Archivos:** `desktop/src/components/proxy/ProxyDashboard.test.tsx`
- **Acción:** tests que fallan contra el código actual: (a) storage de la key (`proxyUserKey` default/roundtrip + save del form con trim); (b) el fetch de snapshot lleva `x-vanta-user-key`; (c) 401 → hint accionable.
- **Verify:** `npx vitest run src/components/proxy/ProxyDashboard.test.tsx` → FAIL (razón correcta)
- **Resultado:** ✅ RED confirmado — `4 failed | 4 passed (8)`: los 4 nuevos fallan por la razón correcta (helper ausente, input de key ausente, header ausente, hint ausente); los 4 pre-existentes siguen verdes
- **Estado:** ✅

### Step 3 — GREEN: fix FIND-155 + alcanzabilidad de la lente
- **Archivos:** `ProxyDashboard.tsx`, `WorkspaceShell.tsx`, `i18n/dictionaries.ts`
- **Acción:** (a) `LS_USER_KEY` + `proxyUserKey()`/`setProxyUserKey()` + header en `fetchSnapshot` + input password en el form + hint 401 + reset limpia key; (b) botón sidebar PROXY incondicional (Spec #4) + `help.proxyHint` corregido; (c) i18n ES/EN (`proxy.userKeyAria`, `proxy.authHint`, `proxy.setupHint`).
- **Verify:** tests verdes + `npx tsc --noEmit`
- **Resultado:** ✅ GREEN — scoped `8/8` (108ms) + i18n `11/11` (paridad ES/EN intacta); `tsc` sin errores en los archivos del fix (los 6 errores de `vanta-wasm-map.ts` son pre-existentes → FIND-275)
- **Estado:** ✅

### Step 4 — VERIFY: suite desktop + build
- **Archivos:** —
- **Acción:** suite scoped del archivo + suite completa + `npm run build`.
- **Verify:** todos exit 0
- **Resultado:** ✅ — suite completa `90/90` (14 files, 20.6s); `npm run build` exit 0 (12.5s); registrado vía `campaign_verify_cmd`
- **Estado:** ✅

### Step 5 — Guion + checklist + evidencia estática
- **Archivos:** este task file (§Guion, §Investigation Notes)
- **Acción:** completar guion por panel + checklist ejecutable (arriba) + evidencia estática read-only.
- **Verify:** secciones presentes + gates docs
- **Resultado:** ✅ (secciones §Guion / §Investigation Notes)
- **Estado:** ✅

### Step 6 — Gates docs + commit LOCAL + recitation PARTIAL
- **Archivos:** `docs/dev/tasks/DESKTOP-44.md` + generados (`docs/index.md`, `llms.txt`) — **excluidos del commit** (su regen incluye WIP ajeno en vuelo; ver §Notas)
- **Acción:** `check-links` + `check-docs` + `gen-index --write/--check` + `validate-docs-coverage`; OCR advisory; commits LOCALES (`fix(desktop):` + `docs(desktop):`, pathspec propio — WIP ajeno fuera); `campaign_update_task_state(taskId DESKTOP-44, in-progress)` con recitation PARTIAL; RESULTADO §7.
- **Verify:** gates verdes + commits locales (sin push)
- **Resultado:** ✅ — gates docs 4/4 + markdownlint 0 issues + OCR 3 reviewables propios sin Critical/High; commits LOCALES `5a8c06bd` (fix) + `a674849f` (docs+FIND-275); recitation PARTIAL persistida (campaña queda `in-progress`)
- **Estado:** ✅

## Evidencia de verificación (post-implementación, 2026-10-05)

| Gate | Comando | Resultado |
|------|---------|-----------|
| RED | `npx vitest run src/components/proxy/ProxyDashboard.test.tsx` | ✅ FAIL esperado — 4 failed \| 4 passed (8); fallos por la razón correcta |
| GREEN + suite | `npx vitest run src/components/proxy/ProxyDashboard.test.tsx` | ✅ 8/8 (108ms) vía `campaign_verify_cmd` |
| Suite completa | `npm test` | ✅ 90/90 (14 files, 20.6s) vía `campaign_verify_cmd` |
| Type check | `npx tsc --noEmit` | ⚠️ 6 errores **pre-existentes** en `vanta-wasm-map.ts` (archivo NO tocado; committed-as-is) → **FIND-275**; los archivos del fix no aparecen en el reporte |
| Build | `npm run build` | ✅ exit 0 (12.5s) vía `campaign_verify_cmd` |
| check-links | `node scripts/docs/check-links.mjs` | ✅ exit 0 — 0 broken markdown; wikilinks en budget |
| check-docs | `node scripts/docs/check-docs.mjs` | ✅ exit 0 — gating all clear (kind `task` OK) |
| gen-index | `node scripts/docs/gen-index.mjs --write` + `--check` | ✅ exit 0 (1524 docs; entrada DESKTOP-44 indexada) |
| docs coverage | `pwsh scripts/validate-docs-coverage.ps1` | ✅ exit 0 — 0 gaps |
| OCR delegation | `pwsh dev-tools/ocr-review.ps1 -Format json` | ✅ 3 reviewables propios (`WorkspaceShell.tsx`/`ProxyDashboard.tsx`/`dictionaries.ts`, grupo TS); auto-review sin Critical/High; test excluido por diseño (`default_path`) |

## RESULTADO (§7 — contrato de retorno)

```
RESULTADO: 🟡 INCOMPLETO
STEPS_OK: 6/6 steps de prep (sesión E2E owner pendiente — contrato del plan)
PROXIMO_STEP: sesión owner (S1-S7 del §Guion) — coordinar con el owner (requiere upstream LLM vivo); luego cerrar FIND-155 + campaña (review P2-01)
COMMIT_HASH: 5a8c06bd (`fix(desktop):`, LOCAL, sin push) + cierre `docs(desktop):` de este archivo (ver git log)
ARCHIVOS: desktop/src/components/proxy/ProxyDashboard.tsx · desktop/src/components/proxy/ProxyDashboard.test.tsx · desktop/src/components/layout/WorkspaceShell.tsx · desktop/src/i18n/dictionaries.ts · docs/dev/tasks/DESKTOP-44.md · docs/dev/Backlog.md (FIND-275)
VERIFY_CONTRATO: prep pasa (fix FIND-155 + guion + checklist + evidencia estática; tests 8/8 + suite 90/90 + build exit 0 + gates docs verdes); contrato E2E pendiente de sesión
BLOQUEO: ninguno — owner-assisted por diseño (la sesión requiere upstream LLM vivo)
GATES_EVALUADOS: P:no(plan Task 34 ✅ DO) D:no(sin símbolos públicos; fix mandatado por FIND-155) V:no(verde en prep) C:no(sin colaterales; WIP ajeno fuera del commit; FIND-275 registrado)
SKILLS_CARGADAS: test-driven-development, frontend-ui-engineering, source-driven-development, security-and-hardening, incremental-implementation, documentation-skill, context-engineering (SDP v3; base auto: campaign-executor/progreso/ponytail)
```

## Dependencias

- **FIND-155** — prerequisito: resuelto en código por esta prep (validación E2E en la sesión).
- **Upstream LLM vivo** (owner) — requisito de la sesión (P1).
- **DESKTOP-38** (✅) — dashboard + deuda Step 4 (esta tarea la cierra).
- **API-05** (✅) — auth D34 en el proxy + helper de seed (`seed_live_smoke_auth_store`).
- **En vuelo (no tocar):** PROV-13 (`providers/**` + Cargo.lock), master plan (modificado en el tree — pathspec), `opencode.jsonc`.

## Notas

- **(DISCOVERY) Gap extra encontrado y arreglado:** la lente PROXY era inalcanzable sin `vanta.proxy.url` previo (sidebar/palette condicionados; Settings sin sección) — sin el fix, el formulario de setup de FIND-155 sería inalcanzable en un primer uso. Fix mínimo (Spec #4) + `help.proxyHint` corregido a la lente.
- **(DISCOVERY) Evidencia estática dejada, sesión NO simulada:** el guion S5/S6 define qué capturar; los mocks quedan solo para los tests unitarios de la UI (no como evidencia E2E).
- **(S6) FIND-275 (hallazgo colateral, NO tocado):** `npx tsc --noEmit` falla pre-existente en `desktop/src/vanta-wasm-map.ts:84,96,105,141,149,166` (6 errores TS2345/TS2352; archivo committed-as-is, sin diff local) — registrado como fila `FIND-275` en Backlog (routing `prompts/findings.md`). No bloquea: el gate del desktop es `npm run build` (vite) y pasa.
- **(S6) Generados NO commiteados:** `docs/index.md`/`llms.txt` regenerados por `gen-index --write` incluyen WIP ajeno en vuelo (descripción nueva de SHOW-02 + entrada de PROV-13 con archivo aún untracked) → se excluyen del commit (pathspec propio, precedente DESKTOP-43); el próximo cierre regenera. Los gates docs corrieron verdes sobre el working tree completo.
- **(S3) validate_scope advisory:** los 5 paths reportan "OUTSIDE declared blast radius" — el parser del plan lista los archivos de `vanta-proxy` del bloque F0; el blast real de la prep es desktop + task file (declarado en §Blast Radius). Advisory, no bloqueante (mismo precedente DESKTOP-43).
- **(S6) Backlog:** la fila `FIND-155` queda `Pendiente` hasta la validación de la sesión; al cerrar, remover la fila (registro en `avance/`).
- **Learning:** el rate-limit se puede verificar sin costo con `count_tokens` (Anthropic) — el limiter cuenta cualquier request que pase auth, sin importar el status del upstream.
