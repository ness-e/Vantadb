# API-05: W4 proxy — auth `/snapshot` + endpoints + config (SEGURIDAD)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-24-api-ejecucion.md` (§Task 5)
- **Fuente:** `docs/dev/Backlog.md` P51 fila `API-05`; ficha `docs/dev/tasks/API-STD-11.md` (X1–X5); Gate P síntesis `API-STD-15.md:25-26`
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🔴 (único hallazgo de seguridad de la campaña: exposición `sessions`/`cost` sin auth)
- **Tipo:** Mixto (Rust proxy + TOML + docs API)
- **Turns estimados:** 15-30
- **Creado:** 2026-09-25T00:00
- **last-synced:** 2026-09-25T00:00
- **Estado:** ✅ COMPLETED (2026-09-26) — review P2-01 ✅ tras fix R1; contrato 4/4 (401/200/401 + 291/0); commit local `f0c3f95f` (sin push)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps (review + commit son gates del orquestador/lead)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `desktop/src/components/proxy/ProxyDashboard.tsx` (fetch `GET /snapshot` sin headers — consumidor directo, ver Spec #9), tests `vanta-proxy/tests/prx01_wiring.rs`, `prx05_aux.rs` (auth ya presente), docs `docs/api/PROXY.md`, `docs/user/operations/EXPERIMENTAL_FEATURES.md:57` |
| Callees | `crate::auth::AuthDb::authenticate` (patrón ya usado por `session_advance`), `crate::error::ProxyError::Unauthorized` (401), `crate::config::UpstreamConfig`, `crate::cache::ExactCache` |
| Implicaciones | `/snapshot` pasa de 200 abierto a 401 sin `x-vanta-user-key` (breaking `feat!:`); rename de rutas `spaceId→space_id` NO cambia URLs (placeholder posicional); `/session/advance` → `/sessions/advance` (breaking, sin consumidores fuera de tests); `upstream.url` default vacío → fail-fast al arrancar (breaking); `cache.ttl_secs=0` pasa a "desactivado" (breaking de semántica, cache seguía off por default); rate-limit solo docs (campo ya cableado); desktop dashboard queda 401 hasta migrar (FIND-155) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vanta-proxy/src/server.rs` (1271L), `vanta-proxy/src/config.rs` (514L), `vanta-proxy/config.toml` (19L), `vanta-proxy/src/cache.rs` (985L), `vanta-proxy/src/lib.rs`, `vanta-proxy/src/main.rs`, `vanta-proxy/src/error.rs`, `vanta-proxy/src/auth.rs:1-230`, `vanta-proxy/src/handlers/{openai,anthropic,responses,auxiliary,mod}.rs`, `vanta-proxy/tests/prx01_wiring.rs`, `docs/api/PROXY.md`, `docs/dev/tasks/API-STD-11.md`, `docs/dev/tasks/API-STD-15.md`
- **Archivos referenciados hacia dentro (imports/deps):** `server.rs` → `auth.rs` (authenticate), `error.rs` (ProxyError→401), `config.rs` (UpstreamConfig/CacheConfig), `cache.rs` (TTL), `handlers/*`; `config.rs` → `error.rs`
- **Archivos que referencian a los editados (grep):** `desktop/src/components/proxy/ProxyDashboard.tsx:59-63` (`/snapshot`), `desktop/e2e/proxy-dashboard.spec.ts` (mock `**/snapshot`, no rompe), `vanta-proxy/tests/prx01_wiring.rs:187-255` (`/session/advance` + `/snapshot`), `docs/api/PROXY.md` (citas de línea), `docs/user/operations/EXPERIMENTAL_FEATURES.md:57`
- **Veredicto impacto:** **medio** — auth rompe al consumidor desktop (F1 materializado; stop condition: NO revert → FIND-155 + doc); renames de rutas son breaking sin consumidores externos conocidos; config fail-fast rompe TOML que dependía del default self-loop (explícitamente buscado: era el footgun X4)

## Contrato

`curl /snapshot` sin credencial → 401 **Y** con credencial → 200 (HTTP real, local) **Y** `cargo test --target-dir target/session-api01 -p vanta-proxy` verde **Y** `rg spaceId vanta-proxy/src/server.rs` = 0

## Spec (SDD — feature-add: ruta aditiva `/…/v1/responses` + auth gate)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Esquema de auth para `/snapshot` | (A) `x-vanta-user-key` D34 existente / (B) Bearer JWT Gate P | A | ✅ decidido-por-evidencia: `auth.rs:100` + patrón `session_advance` (`server.rs:836`); el proxy entero usa D34; migrar a Bearer es otra superficie (fuera de contrato) |
| 2 | ¿Excepción loopback que evite 401? | (A) No bypass / (B) bypass loopback documentado | A | ✅ decidido-por-evidencia: contrato exige 401 sin credencial (curl local ES loopback → B lo viola) y bypass reabre la exposición local que este task cierra. El carve-out documentado queda en `validate_startup` (bind loopback sin keys) + FIND-155 |
| 3 | `/snapshot` singular vs plural (`/api/v2/snapshots`) | (A) mantener `/snapshot` / (B) renombrar a `/snapshots` | A | ✅ decidido-por-evidencia: contrato fija `curl /snapshot` + desktop lo consume; rename sería breaking doble sin ganancia. Pluralización aplicada a `/session/advance` → `/sessions/advance` |
| 4 | Alias para `/session/advance` tras rename | (A) sin alias (breaking `feat!:`) / (B) doble ruta | A | ✅ decidido-por-evidencia: política Gate P "quitar alias doble"; único consumidor son tests propios |
| 5 | `{agent}/{spaceId}` camel en paths | renombrar placeholder a `space_id` | — | ✅ decidido-por-evidencia: contrato `rg spaceId` = 0; URL-shape idéntico (placeholder posicional) |
| 6 | `/v1/responses` sin forma prefijada (X3) | (A) añadir `/{agent}/{space_id}/v1/responses` / (B) dejar solo plano | A | ✅ decidido-por-evidencia: finding X3 + parity con messages/models; aditivo puro (`responses_prefixed`, ~10 líneas) |
| 7 | Default `upstream.url` self-loop (X4) | (A) default vacío + error claro / (B) mantener self-loop + guard | A | ✅ decidido-por-evidencia: Risk Register API-STD-11 ("default vacío + error claro"); `points_at_self` queda para URLs explícitas |
| 8 | `cache.ttl_secs=0` = ¿eterno o desactivado? (X5) | (A) 0 = cache desactivada / (B) 0 = nunca expira | A | ✅ decidido-por-evidencia: backlog "`ttl_secs=0` = desactivado"; `enabled=true && ttl_secs>0` para activar |
| 9 | Rate-limit "sin uso" (X5) | (A) docs truth (ya cableado en `process_inner`) / (B) re-cablear | A | ✅ decidido-por-evidencia: `RateLimiter::new(config.server.rate_limit_per_minute)` `server.rs:161` + check `:367` — solo el comentario "placeholder/parsed but unused" (`config.rs:219`) era stale |
| 10 | Desktop `ProxyDashboard` sin key | (A) FIND-155 + doc / (B) migrar UI inline | A | ✅ decidido-scope: desktop fuera de archivos clave API-05 (P12 cerrada); findings.md → ticket (FIND-155) |
| — | Gate D (feature-add) | — | — | ✅ scope fijado por plan Task 5 (Gate Result ✅ DO) + contrato mecánico del orquestador; sin decisión abierta que requiera HITL |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) TODAS las rutas exigen `x-vanta-user-key` válido (D34) — `/snapshot` incluido, sin bypass loopback; (2) wire `/v1/*` byte-identical cuando no hay opt-in (no tocar pipeline `process_inner`); (3) no tocar `vantadb-mcp/**`, `src/server/**`, `vantadb-python/**`, `vantadb-ts/**`, `src/parser/**`, `vantadb-pro`; (4) no commit/push (política owner 2026-09-25); (5) desktop e2e (`page.route` mock) debe seguir verde (no se toca desktop)
- **Comandos de verificación:** `cargo test --target-dir target/session-api01 -p vanta-proxy` (verde) · `rg spaceId vanta-proxy/src/server.rs` (0) · `cargo fmt --check` · `cargo clippy --target-dir target/session-api01 -p vanta-proxy --all-targets -- -D warnings`
- **Deuda pendiente:** FIND-155 (migrar desktop ProxyDashboard a enviar key + DESKTOP-44 validación manual con proxy real); deuda neta del PR ≤ 0 (este cambio paga la exposición X1)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** negativo (cierra exposición activa; no introduce `unsafe`, clones ni dual API). Deuda registrada: ninguna nueva; FIND-155 es migración de consumidor externo, no deuda del core.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato 4/4 + tests del cambio (401/200 e2e + config + ttl) verdes + fmt/clippy |
| **Commit** | Atómico `feat!(api): API-05 — …` (lo hace el LEAD; este worker NO commitea) |
| **Release** | `dev-tools/verify_changed.ps1` verde (scoped) + docs mismo-PR (Regla 3) |

## Herramientas necesarias

- Shell cargo con `--target-dir target/session-api01` (MCP server lockea `target/debug/vanta-cli.exe`)
- `codegraph_explore` (blast radius), `campaign_verify_cmd` por step
- Smoke: binario `vanta-proxy` + `curl.exe` (401/200)
- **Skills cargadas (SDP):** `security-and-hardening` (security-sensitive, checklist de cierre), `api-and-interface-design` (naming/auth de endpoints), `test-driven-development` + `incremental-implementation` + `context-engineering` (ciclo core del worker), `doubt-driven-development` (gate adversarial; review P2-01 la ejecuta `vanta-review`), `source-driven-development` (patrones axum/TOML verificados contra el propio repo, sin API nueva). `ponytail(full)` activo (YAGNI, mínimo diff). Descartadas: `frontend-ui-engineering` (no se toca UI; desktop → FIND-155).

## Investigation Notes

- **X1 CONFIRMADO (re-verificado 2026-09-25):** `snapshot()` `server.rs:863-887` no extrae headers ni llama `authenticate` (vs `session_advance` `:830-858` que sí). Expone turns/sessions/writeback/rate_limit/cost.
- **F1 materializado:** desktop `ProxyDashboard.tsx:60` hace `fetch(${base}/snapshot)` **sin headers**; no existe input de key (`rg x-vanta-user-key desktop/src` = 0). → stop condition NO revert: auth + FIND-155 + nota docs.
- **F2 resuelto:** el rate-limit del proxy SÍ se usa (1 punto: `process_inner` paso 2, `server.rs:367`); "parsed but unused" era comentario stale → fix de docs, sin re-cablear. No hay doble enforcement con el server (superficies distintas).
- **F3:** no aplica (fuera de scope; socket Unix solo con benchmark — Regla 9).
- **Web research:** no requerida (Gate P cita RFC/estándares; sin APIs externas nuevas). Auth/axum/TOML son patrones internos ya existentes.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — F1/F2/F3 resueltos en Discovery; decisiones Spec 1-10 cerradas |
| Pendientes de ejecución (downhill) | 8 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — APPLICA (auth de red, cierre de exposición). `security-and-hardening` cargada. Threat model: activo = sessions/cost/writeback (información operativa + costos); boundary = HTTP `/snapshot`; STRIDE-I (information disclosure) era el fallo → mitigación: auth D34 obligatoria + test 401/200; spoofing → key válida contra `user` store; DoS → rate-limit ya activo en wire (no en /snapshot, read-only barato). Hallazgos: ninguno nuevo post-fix.
- [ ] **PERFORMANCE** — NO aplica: `/snapshot` es read-only sobre estado ya existente; no hot path (search/ingest/serialización intactos). Sin benchmark requerido (Regla 9 no dispara).

## Steps

### Step 1: RED — test e2e 401/200 de `/snapshot` + adaptar helper existente
- **Archivos:** `vanta-proxy/tests/api05_snapshot_auth.rs` (nuevo), `vanta-proxy/tests/prx01_wiring.rs`
- **Acción:** crear test file con seed de engine in-memory + mock upstream + `router()` en puerto efímero: (a) `snapshot_requires_user_key` — GET sin header → 401, con `x-vanta-user-key` → 200 + shape (`sessions`, `writeback`, `rate_limit`, `cost`); (b) `seed_live_smoke_auth_store` — siembra store file-backed en `CARGO_TARGET_TMPDIR/api05-smoke-authdb` (key `sk-smoke`) para el smoke curl; (c) test de ruta prefijada responses (se usará en Step 3). Adaptar `snapshot_degraded` de `prx01_wiring.rs` para enviar el key (compat post-fix). Verificar que `snapshot_requires_user_key` FALLA (RED: hoy 200 sin key).
- **Verify:** `cargo test --target-dir target/session-api01 -p vanta-proxy --test api05_snapshot_auth snapshot_requires_user_key` — esperado: FAIL (assert 401→200)
- **Estado:** ✅ DONE (RED probado: `left: 200, right: 401`; seed helper `#[ignore]` añadido)

### Step 2: GREEN — auth D34 en `/snapshot`
- **Archivos:** `vanta-proxy/src/server.rs`
- **Acción:** `snapshot()` extrae `HeaderMap` y llama `state.auth.authenticate(&headers)?`; devuelve `Result<Json<Value>, ProxyError>` (401 vía `ProxyError::Unauthorized`, misma forma que `session_advance`). Actualizar doc comment (requiere key; D34).
- **Verify:** `cargo test --target-dir target/session-api01 -p vanta-proxy --test api05_snapshot_auth snapshot_requires_user_key` verde + `cargo check --target-dir target/session-api01 -p vanta-proxy`
- **Estado:** ✅ DONE (test 401/200 verde; prx01 5/5 verde)

### Step 3: Endpoints canónicos — `space_id`, `/sessions/advance`, responses prefijado
- **Archivos:** `vanta-proxy/src/server.rs`, `vanta-proxy/src/handlers/responses.rs`, `vanta-proxy/src/handlers/{openai,anthropic,auxiliary}.rs` (docs), `vanta-proxy/src/lib.rs`, `vanta-proxy/tests/prx01_wiring.rs`
- **Acción:** renombrar placeholder `{spaceId}`→`{space_id}` en las 4 rutas + comentario; `POST /session/advance`→`POST /sessions/advance` (rename en router, doc fn y 5 sitios de test); añadir `POST /{agent}/{space_id}/v1/responses` con handler fino `responses_prefixed` (mismo `process`, space del path); docs de módulo `lib.rs` y handlers en snake.
- **Verify:** `rg -c "spaceId" vanta-proxy/src/server.rs` = 0 Y `rg "session/advance" vanta-proxy` = 0 (solo `sessions/advance`) Y tests focused verdes
- **Estado:** ✅ DONE (spaceId server.rs=0; `/sessions/advance` en server+session.rs; `responses_prefixed` verde; prx01 5/5)

### Step 4: Config — default upstream vacío + fail-fast
- **Archivos:** `vanta-proxy/src/config.rs`, `vanta-proxy/config.toml`
- **Acción:** `UpstreamConfig::default().url = ""`; en `validate_startup` (o check hermano) error claro si algún upstream resuelto tiene URL vacía (`upstream.url is empty — set [upstream].url…`); comentario en config.toml; actualizar test que "bendecía" el self-loop por default → default vacío + test de detección de self-loop explícito + test de rechazo por URL vacía.
- **Verify:** `cargo test --target-dir target/session-api01 -p vanta-proxy config::` verde
- **Estado:** ✅ DONE (15/15 config tests verdes; fail-fast probado para `upstream` y `[[upstreams]]`)

### Step 5: Config — `cache.ttl_secs=0` = desactivado + rate-limit docs truth
- **Archivos:** `vanta-proxy/src/cache.rs`, `vanta-proxy/src/config.rs`
- **Acción:** cache activa solo con `enabled=true && ttl_secs>0`; `is_expired(_, 0)` = expirado (defensa en profundidad, elimina convención invertida `TTL_DISABLED`); docs de campo/estructura; ajustar tests (configs de tests con TTL>0, predicado TTL=0→expirado); corregir comentario stale "Rate-limits placeholder … parsed but unused" → enforce real en 1 punto.
- **Verify:** `cargo test --target-dir target/session-api01 -p vanta-proxy cache::` verde + `rg "parsed but unused" vanta-proxy/src` = 0
- **Estado:** ✅ DONE (cache 17/17 verdes; `TTL_DISABLED` eliminado; `rg "parsed but unused"` = 0)

### Step 6: Docs mismo-PR (Regla 3)
- **Archivos:** `docs/api/PROXY.md`, `docs/user/operations/EXPERIMENTAL_FEATURES.md`, `docs/dev/Backlog.md` (fila FIND-155)
- **Acción:** PROXY.md — endpoints (auth en `/snapshot`, `/sessions/advance`, responses prefijado), sección de auth (todas las rutas requieren key; desktop pendiente FIND-155), defaults (`upstream.url=""` fail-fast, `ttl_secs=0` desactivado, rate-limit 1 punto), conteo de registros 11/8 lógicos, refrescar citas de línea movidas; EXPERIMENTAL_FEATURES:57 marcar auth API-05 ✅; alta FIND-155 (desktop).
- **Verify:** conteo `rg -c "\.route\(" vanta-proxy/src/server.rs` = 11 y `rg "ttl_secs"` PROXY.md consistente + `scripts/validate-docs-coverage.ps1` (si aplica)
- **Estado:** ✅ DONE (11 registros; PROXY.md endpoints/auth/defaults/citas; FIND-155 en Backlog; EXPERIMENTAL_FEATURES ✅)

### Step 7: Verify full scoped + smoke live 401/200
- **Archivos:** (verificación)
- **Acción:** `cargo test --target-dir target/session-api01 -p vanta-proxy` completo; fmt; clippy `-D warnings`; contrato greps; build binario `vanta-proxy`; levantar con config smoke (`auth.db_path` = store sembrado en Step 1) y `curl /snapshot` sin key → 401 / con `x-vanta-user-key: sk-smoke` → 200; si el binario no levanta → stop condition del plan (lectura + tests).
- **Verify:** contrato 4/4 + smoke 401/200 reales (evidencia pegada en §Notas)
- **Estado:** ✅ DONE — suite 18/18 binarios ok ×2 (exit 0); fmt package+workspace ok; clippy `--no-deps -D warnings` ok; smokes: no-key=401 / with-key=200 / bad-key=401; `spaceId` en server.rs = 0; coverage docs 0 gaps.

### Step 8: Cierre — OCR advisory + DoD + recitation
- **Archivos:** `docs/dev/tasks/API-05.md`, plan file (estado)
- **Acción:** `pwsh dev-tools/ocr-review.ps1 -Format json` (advisory; Critical/High bloquean) + DoD 3 niveles + sync task file/plan + evidencia para review P2-01 de `vanta-review` (auth de red) + RESULTADO §7 (NO commit/push).
- **Verify:** OCR sin Critical/High + contrato 4/4 + task file sync
- **Estado:** ✅ DONE (OCR delegation spec generado — grupos 5/6 incluyen vanta-proxy; self-review contra Rust rules: 0 Critical/High; checklist `security-and-hardening` completa; plan/task sync; review P2-01 delegada a `vanta-review`)

## Dependencias
- Task API-03 ✅ (W2 HTTP canónico: plurales/cursor/status) — completada 2026-09-25
- Paralela con API-04/06; nextTask: API-06/07/08

## Review (GATE — agente distinto, P2-01)

> Pendiente: la ejecuta `vanta-review` (la invoca el orquestador, no el implementador). Evidencia dejada para el reviewer: contrato 4/4, smoke curl 401/200, decisión loopback (Spec #2), impacto desktop (Spec #10/FIND-155), threat model §SECURITY.

- **Revisor:** vanta-review (sesión `ses_f2410373fffeBmylbBMsKppv2Q`) — dictamen 2026-09-25: ❌ R1 (comentario stale `server.rs:114-115`) + 2 nits → fix aplicado por el lead (comentario + `spaceId`→`space_id` en `rate_limit.rs`/`proxy_wire.rs`) → ✅
- **Enfoque:** auth de `/snapshot` (sin bypass), breaking de endpoints/config, drift de docs
- **Cómo se probó:** e2e real HTTP (test + smoke binario+curl), greps mecánicos, suite proxy completa
- **Veredicto:** ✅ APPROVE tras fix R1 — evidencia del revisor: suite **291/0** (×3), smoke live **401/200/401** reproducido con binario fresco, `rg spaceId`=0, 11 routes, fmt/clippy verdes. DoD Release: fmt/check/clippy del lead ✅ (verify_changed completo al push).

## Notas
- **Smoke live (evidencia, 2026-09-25):** binario `target/session-api01/debug/vanta-proxy.exe` (build `--features vantadb/fjall`) + config `target/session-api01/tmp/api05-smoke.toml` + store sembrado por `api05_snapshot_auth::seed_live_smoke_auth_store` (`tmp/api05-smoke-authdb`, user `usr-smoke`/key `sk-smoke`). Resultado: `GET /snapshot` **sin key → 401**; **con key → 200** (body real con `cost`/`rate_limit`/`sessions`/`turns`); **key inválida → 401**. Logs: `target/session-api01/tmp/api05-smoke.*.log`.
- **Nota entorno:** el build `-p vanta-proxy` no activa `fjall` (dep con `default-features=false`); para el smoke file-backed se usó `--features vantadb/fjall` (sin cambios de manifest). El suite in-memory no lo requiere.
- **Nota comportamiento (F1/F2/F3):** (F1) desktop Proxy Dashboard queda 401 hasta migrar → FIND-155 + nota en PROXY.md; (F2) rate-limit ya estaba cableado en 1 punto — fix de docs; (F3) Unix-socket no aplica (Regla 9, sin benchmark).
- **Nota acoplamiento WIRE-01:** el lado output del cost tracking se observa en el path buffered (cache activa). Con la nueva semántica `ttl_secs=0 → cache off`, un operador con `enabled=true` + `ttl=0` pierde cache y observación output-side hasta poner TTL>0 — breaking documentado en PROXY.md; el test `prx03` se adaptó (`ttl_secs: 3600`) manteniendo su intención (path buffered).
- **Decisión loopback (para el reviewer):** el plan sugería "auth con excepción loopback documentada" si desktop rompía. El contrato exige 401 sin credencial y el smoke es local (loopback) → un bypass loopback violaría el contrato y reabriría la exposición local. Se implementó auth uniforme; el carve-out documentado queda en `validate_startup` (loopback puede arrancar sin keys) y el consumidor desktop se ruteó a FIND-155.
- **Checklist `security-and-hardening` (cierre):** auth en toda ruta incl. `/snapshot` (401 probado e2e+smoke) ✓ · sin bypass loopback ✓ · errores no exponen internals ni keys (`Unauthorized` genérico; el mensaje de config no filtra secretos) ✓ · config fail-closed (upstream vacío rechaza; self-loop explícito rechazado) ✓ · sin deps nuevas (no `cargo audit` requerido) ✓ · sin `unsafe`/locks-across-await nuevos ✓ · rate-limit intacto (fail-open por diseño documentado) ✓ · STRIDE-I mitigado (info disclosure) ✓.
- **DoD 3 niveles:** Task ✅ (contrato + tests + fmt/clippy) · Commit ⏳ (atómico `feat!(api): API-05 — …`; lo hace el LEAD; archivos listados abajo) · Release: `verify_changed.ps1` no corrido (corre core-features + `fmt --all` fuera de scope y el árbol tiene WIP de tareas paralelas; equivalentes scoped ejecutados: `cargo fmt -p vanta-proxy --check`, `cargo fmt --all -- --check`, clippy `--no-deps -D warnings`, suite proxy completa, docs-coverage) — justificado.
- **Archivos para el commit del lead:** `vanta-proxy/src/{server,config,cache,session,lib}.rs`, `vanta-proxy/src/handlers/{responses,openai,anthropic,auxiliary}.rs`, `vanta-proxy/config.toml`, `vanta-proxy/tests/{api05_snapshot_auth.rs,prx01_wiring.rs,prx03_cost.rs,prx09_cache.rs}`, `docs/api/PROXY.md`, `docs/user/operations/EXPERIMENTAL_FEATURES.md`, `docs/dev/Backlog.md` (fila FIND-155), `docs/dev/tasks/API-05.md`, `docs/dev/plans/2026-09-24-api-ejecucion.md`.
