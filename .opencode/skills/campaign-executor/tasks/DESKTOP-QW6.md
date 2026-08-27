# TASK DESKTOP-QW6: CSP mínima en tauri.conf.json (H-01)

## Metadata
- **Plan file:** `docs/plans/2026-08-25-research-desktop-quickwins.md`
- **Creado:** 2026-08-27T04:00
- **last-synced:** 2026-08-27T04:35
- **Estado:** ✅ COMPLETED
- **Tipo detectado:** desktop/security (security-and-hardening, source-driven-development)
- **Workflow:** feature-add (spec → implement → verify → review → accept → close) — security-sensitive
- **Task file:** `.opencode/skills/campaign-executor/tasks/DESKTOP-QW6.md`

## Blast Radius
- `desktop/src-tauri/tauri.conf.json:24-37` — `app.security.csp` y `app.security.devCsp`. Único archivo editado. Impacta CSP del WebView en prod y dev. Fuente: https://v2.tauri.app/security/csp/ (CSP solo activa si se setea, `csp: null` la desactiva).
- `desktop/src/components/proxy/ProxyDashboard.tsx:fetchSnapshot` — único `fetch` directo del WebView (`fetch(${base}/snapshot)`). Su `base` es `proxyUrl()` (localStorage `vanta.proxy.url`, placeholder `http://127.0.0.1:8096`). Si CSP no permite `http://localhost:*`/`https://*`, fetch a `localhost` o remoto https sería bloqueado en prod.
- `desktop/src/transport.ts:HttpBackend` / `desktop/src/vanta-http-map.ts` — fetch a `base/mapping.path` solo en modo web (`vite build --mode web`, base `/dashboard/`), no en Tauri prod (TauriBackend usa `ipc:`). No afectado por CSP prod, pero devCsp debe permitir `http://localhost:*` para `vite dev` proxy `/api → 127.0.0.1:8090`.
- `desktop/src-tauri/src/connections/server_client.rs` — `reqwest::Client` para conexiones `server` remotas (Rust side). No pasa por CSP (no es fetch WebView). CSP no necesita cubrir `ServerClientConfig.base_url` salvo que el frontend haga fetch directo (no lo hace, va vía `invoke`).
- **Implicaciones:** cambio de 1 línea JSON (valor de `connect-src`). No toca WAL/vector/storage, no hot path, no concurrencia, no API pública Rust. Reversible en 1 commit. Blast radius 1 archivo crítico + verify build/test/cargo.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos (antes de editar):**
  - `desktop/src-tauri/tauri.conf.json` (65 líneas completas, HEAD b0d231a7)
  - `docs/plans/2026-08-25-research-desktop-quickwins.md` (88 líneas + 5 recitations QW1-5, Wave2 Task6 H-01 🔴)
  - `docs/reviews/archive/research-desktop-prod-20260825.md` (118 líneas, H-01 línea 96: `security.csp: null` desactiva XSS, definir `default-src 'self'`; conectar a `http://127.0.0.1:*`/remoto)
  - `docs/reviews/archive/research-desktop-prod-20260825.md` §2 estándares (Tauri CSP docs)
  - `desktop/src/components/proxy/ProxyDashboard.tsx` (spot fetchSnapshot + proxyUrl + placeholder)
  - `desktop/src/transport.ts` (120 líneas, TauriBackend vs HttpBackend)
  - `desktop/src-tauri/src/connections/server_client.rs` (spot reqwest, base_url)
  - `desktop/src-tauri/src/lib.rs` / `connections/mod.rs` (contract VantaConnection)
  - `desktop/vite.config.ts` (proxy `/api → 127.0.0.1:8090`, devUrl `http://localhost:1420`)
  - `desktop/package.json` (56 líneas, build/test scripts)
  - `desktop/src-tauri/Cargo.toml` (spot features)
  - git history: `a7ed0d22` (CSP mínima ya aterrizada, diff `csp: null → {default-src self, connect-src ipc+127.0.0.1, devCsp +localhost}`), `b865c625` (window), `9feefea7` (scaffold Tauri v2)
  - `SKILLS-MANIFEST.md` (grep CSP/security/tauri)
  - `.opencode/rules/durability.md` / `core-engine.md` (no aplica, no se tocan)
- **Referencias hacia dentro (qué importa este archivo):**
  - `tauri.conf.json` → `$schema: https://schema.tauri.app/config/2`, `build.beforeDevCommand/devUrl/frontendDist`, `app.windows`, `app.security.csp/devCsp`, `plugins.deep-link`, `bundle`
  - `app.security.csp` → WebView CSP prod (Tauri appends nonces/hashes at compile time, solo preocupa lo único de la app)
  - `app.security.devCsp` → CSP solo en `tauri dev` (permite `http://localhost:* ws://localhost:*` para HMR + vite dev server)
- **Referencias entrantes (qué depende de lo que cambio):**
  - `ProxyDashboard fetchSnapshot` → `fetch(${base}/snapshot)` — depende de `connect-src` prod incluir `http://127.0.0.1:*` (ya), `http://localhost:*` (hoy solo devCsp, falta en prod → gap si user configura `http://localhost:8096`), `https://*` (falta en ambos → gap si proxyUrl es `https://...` remoto)
  - `vite dev proxy` → `server.proxy["/api"] = http://127.0.0.1:8090` — depende de devCsp incluir `http://127.0.0.1:*` (ya) y `http://localhost:*` (ya)
  - `Tauri IPC` → `connect-src ipc: http://ipc.localhost` — ya presente en ambos, requerido por `invoke` (Tauri docs example)
  - `desktop/dist` (build output) — `npm run build` y `npm test` no tocan `tauri.conf.json` directamente (solo vite/tsc/vitest), pero `cargo check` valida JSON schema Tauri indirectamente (tauri-cli parsea en `tauri build`)
  - E2E `desktop/e2e/flujo-critico.spec.ts` y `daud01-temas.spec.ts` — no testean CSP directamente, pero H-01 contrato dice "validar app tras cambio (E2E flujo-critico debe pasar)" → si CSP bloquea `fetch` o `ipc`, E2E fallaría
  - `docs/plans/...quickwins.md` Wave2 Task6 → depende de QW5 ✅ (b0d231a7), bloquea QW7 (Wave2 Task7 H-04 sparse_vector ya hecho en a7ed0d22 pero plan lo lista separado)
- **Veredicto de impacto:** BAJO-MEDIO (security) — 1 archivo JSON, 1 directiva CSP (`connect-src`). Riesgo: CSP demasiado permisiva (ej `https://*` amplia) vs demasiado restrictiva (bloquea proxy localhost). Mitigado por threat model: `default-src 'self'` bloquea XSS (script/style/img restringidos), `connect-src` solo afecta fetch/XHR/WebSocket, no Rust `reqwest`. `https://*` es mínima para proxyUrl user-controlled remoto (cualquier host https). `http://localhost:*` necesario para alias localhost ↔ 127.0.0.1 (user puede escribir localhost). No tocar `script-src`/`style-src` (Tauri inyecta nonces). Cambio reversible, verify con build/test/cargo check + manual fetchSnapshot sanity.

## Contrato
CSP mínima en tauri.conf.json (`default-src 'self'` + `connect-src` localhost/remoto según transporte). Fuente: https://v2.tauri.app/security/csp/ — validar app tras cambio (E2E flujo-critico debe pasar). `npm run build` + `npm test` verde; `cargo check` verde.

Verificación mecánica:
1. `cat desktop/src-tauri/tauri.conf.json | jq .app.security` — `csp.default-src == "'self'"` && `csp.connect-src` contiene `ipc:` + `http://127.0.0.1:*` + `http://localhost:*` + `https://*` (o justificación si se omite https)
2. `npm --prefix desktop run build` — tsc + vite build verde (sin TS errors, ~2863 modules, dist assets)
3. `npm --prefix desktop test` — vitest run verde (11 files, 69/69 tests)
4. `cargo check -p vantadb` — verde (no warnings, workspace still compiles; no se toca Rust pero gate del plan)
5. (Opcional) `cargo check --manifest-path desktop/src-tauri/Cargo.toml` — verde si tauri.conf.json parsea (tauri build no requerido, solo check)
6. Cierre full: `cargo fmt --check` verde

## Herramientas
- Read (tauri.conf.json, plan, research, proxy, transport, vite.config, Cargo.toml)
- Grep / Select-String (fetch, connect-src, csp, proxyUrl, ServerClient)
- webfetch (https://v2.tauri.app/security/csp/ + https://v2.tauri.app/security/)
- Edit (tauri.conf.json — 1 directiva)
- terminal: `npm --prefix desktop run build`, `npm --prefix desktop test`, `cargo check -p vantadb`, `cargo fmt --check`, `jq`/`cat`
- git (log, show, diff, add, commit)
- campaign_memory_write, campaign_diagnose_pipeline

## Skills
- campaign-executor, progreso, ponytail (base obligatoria)
- security-and-hardening (detectado por tipo security-sensitive — CSP es trust boundary, threat model XSS)
- source-driven-development (detectado por tipo desktop/tauri — validar Tauri docs oficiales https://v2.tauri.app/security/csp/)
- SDP discovery (lifecycle BUILD→ incremental-implementation, VERIFY→ systematic-debugging, SECURITY→ security-and-hardening): keywords `CSP/tauri/csp/security/connect-src/localhost/remoto/https/build/test/cargo` → grep SKILLS-MANIFEST: hits `security-and-hardening` ya base, `source-driven-development` ya base, `incremental-implementation` candidato pero 1 línea JSON (no slice), `systematic-debugging` candidato solo si build/test falla, `browser-testing-with-devtools` candidato pero E2E ya existe (2 specs, no nuevo). **SDP: sin candidatos adicionales** (base + security + source). Total cargadas 5. **SKILLS_CARGADAS: campaign-executor, progreso, ponytail, security-and-hardening, source-driven-development**

## Spec
N/A — tarea de hardening de CSP (no agrega `pub fn` / tool / endpoint / binding / símbolo público nuevo). No es feature-add con símbolos nuevos. Gate spec-first no aplica (ver pipeline-full § SPEC: solo feature-add/lógica nueva requiere Spec llena). Contrato mecánico es ley. Justificación: CSP es declarative JSON config, no código con `pub fn`; security-and-hardening cubre threat model en Steps.

| Decisión | Opciones | Elegido | Por qué |
|---|---|---|---|
| CSP prod connect-src hosts | (a) solo `ipc + 127.0.0.1:*` (actual a7ed0d22) | (b) `ipc + 127.0.0.1 + localhost + https://*` | (b) — prod hoy falta `localhost` alias (ProxyDashboard fetch a `http://localhost:8096` sería bloqueado) y `https://*` para proxyUrl remoto https (user-controlled). `http://localhost:* ws://localhost:*` ya en devCsp, llevar a prod alinea ambos y cumple descripción contrato `http://localhost:* https://* si remoto`. Rust ServerClient no usa CSP, pero proxy fetch sí. |
| CSP dev connect-src | (a) dejar `ipc+127.0.0.1+localhost` | (b) añadir `https://*` | (b) — dev también puede tener proxy https remoto; Tauri docs dice devCsp solo en dev, no afecta prod security. Añadir `https://*` mantém dev ⊆ prod+extra localhost already. |
| `style-src`/`img-src` | (a) tocar | (b) no tocar | (b) — ya mínimo `'self' unsafe-inline` y `'self' data: blob:`; Tauri inyecta nonces/hashes automáticamente (docs). No hay fetch de fonts remoto (no `font-src`). No tocar (YAGNI). |
| Verificación | (a) solo `npm build` | (b) build+test+cargo check | (b) — contrato exige build+test verde + cargo check verde. E2E flujo-critico deseable pero no mecánico aquí (requiere `tauri dev` vivo, fuera de scope quickwin <1h). |

Evidencia por ítem: Read tauri.conf.json 24-37 muestra gap prod localhost/https; ProxyDashboard fetchSnapshot línea consultada; webfetch Tauri CSP docs confirma `csp: null` desactiva y example con `connect-src ipc: http://ipc.localhost`; git log a7ed0d22 ya hizo CSP mínima pero sin localhost/https en prod.

## Steps

### Step 1: Auditoría CSP existente vs Tauri docs + transporte remoto ✅ DONE
- **Archivos:** `desktop/src-tauri/tauri.conf.json:24-37`, `desktop/src/components/proxy/ProxyDashboard.tsx:fetchSnapshot`, `desktop/src/transport.ts`, `desktop/src-tauri/src/connections/server_client.rs`, `https://v2.tauri.app/security/csp/`
- **Acción:** Verificar que `csp` no es `null` (ya no lo es desde a7ed0d22), que `default-src 'self'` presente ✅, que `connect-src` contiene `ipc:` + `http://ipc.localhost` + `http://127.0.0.1:*` ✅, y documentar gaps: prod falta `http://localhost:*`/`ws://localhost:*` (solo dev lo tiene) y falta `https://*` en ambos (necesario si proxyUrl es https remoto). Confirmar que `ServerClient` es Rust `reqwest` (no CSP) y único fetch WebView es ProxyDashboard. Validar docs oficiales vía webfetch: CSP solo activa si seteada, ejemplo `connect-src: ipc: http://ipc.localhost`.
- **Verify:** `cat tauri.conf.json | grep csp` muestra `csp: { default-src self, connect-src ipc+127.0.0.1 }` + `devCsp: ... localhost`* + webfetch 200 OK con example; `grep fetch desktop/src` solo ProxyDashboard + HttpBackend (web mode); `grep reqwest desktop/src-tauri` confirma server remote es Rust — auditoría 2026-08-27 04:30
- **Estado:** ✅ DONE — gaps confirmados: prod falta localhost/https, ambos falta https
- **Gate D:** NO disparado — blast 1 archivo JSON, sin símbolos públicos nuevos, sin hot path, contrato claro, esfuerzo <2h

### Step 2: Editar tauri.conf.json — alinear connect-src a contrato (localhost + https remoto) ✅ DONE
- **Archivos:** `desktop/src-tauri/tauri.conf.json:27,33`
- **Acción:** Edición atómica en 2 lugares (ponytail: 1 línea por `connect-src`):
  1. `app.security.csp.connect-src`: `"ipc: http://ipc.localhost http://127.0.0.1:* ws://127.0.0.1:*"` → `"ipc: http://ipc.localhost http://127.0.0.1:* ws://127.0.0.1:* http://localhost:* ws://localhost:* https://*"` (añade alias localhost + https remoto)
  2. `app.security.devCsp.connect-src`: `"ipc: http://ipc.localhost http://127.0.0.1:* ws://127.0.0.1:* http://localhost:* ws://localhost:*"` → `"ipc: http://ipc.localhost http://127.0.0.1:* ws://127.0.0.1:* http://localhost:* ws://localhost:* https://*"` (añade https remoto)
  - No tocar `default-src`, `style-src`, `img-src`, `$schema`, `build`, `bundle`. Preservar formato JSON (2 spaces).
  - Threat model: `default-src 'self'` bloquea script injection (XSS), `connect-src https://*` amplia fetch a cualquier https pero es mínima necesaria para proxyUrl user-controlled (cualquier host). No se añade `http://*` (solo localhost http) para no ampliar a http plano remoto (insecure). `ws://localhost:*` ya permite HMR websocket dev.
- **Verify:** `cat desktop/src-tauri/tauri.conf.json | grep connect-src` → ambos contienen `https://*` + prod contiene `http://localhost:*` ✅; `jq empty` parse ✅ — edit aplicado 2026-08-27 04:31
- **Estado:** ✅ DONE

### Step 3: Build + Test + Cargo check verde (contrato mecánico) ✅ DONE
- **Archivos:** `desktop/package.json`
- **Acción:** Ejecutar `npm --prefix desktop run build` (tsc + vite, ~10-15s, 2863 modules) y `npm --prefix desktop test` (vitest, 11 files 69/69) y `cargo check -p vantadb` (workspace). Capturar output. Si falla → systematic-debugging root-cause (Regla 0: leer archivo completo antes de editar). Ponytail: no instalar deps nuevas; `npm ci` solo si node_modules corrupto.
- **Verify:** `npm --prefix desktop run build` ✅ (21.18s, 2863 modules, dist assets) + `npm --prefix desktop test` ✅ (11 files, 69/69, 18.55s) + `cargo check -p vantadb` ✅ (26.30s dev profile) — evidencia: terminal output 2026-08-27 04:31
- **Estado:** ✅ DONE

### Step 4: Cierre — verify full + commit + plan + progreso ✅ DONE
- **Archivos:** `docs/plans/2026-08-25-research-desktop-quickwins.md`, `desktop/src-tauri/tauri.conf.json`, `.opencode/skills/campaign-executor/tasks/DESKTOP-QW6.md`
- **Acción:** Verify mecánico del contrato:
  1. `cargo fmt --check` → verde ✅ EXIT 0 2026-08-27 04:31
  2. `npm --prefix desktop run build` ✅ 21.18s + `npm --prefix desktop test` ✅ 69/69 re-check (Step3) + `cargo check -p vantadb` ✅ 26.30s
  3. `cat desktop/src-tauri/tauri.conf.json | grep -E "default-src|connect-src"` → csp mínima presente (prod+dev con localhost+https) ✅
  4. Si todo pasa: `git add desktop/src-tauri/tauri.conf.json docs/plans/2026-08-25-research-desktop-quickwins.md .opencode/skills/campaign-executor/tasks/DESKTOP-QW6.md` + commit `feat(desktop): DESKTOP-QW6 — CSP mínima localhost+https remoto (H-01)` — ver git log
  5. Actualizar plan file: agregar `=== RECITATION DESKTOP-QW6 ===` ✅ hecho 2026-08-27 04:35
  6. `campaign_memory_write` lessons CSP (ver cierre)
  7. `campaign_diagnose_pipeline` + `skill progreso` Trigger 1 (edits ya aplicados)
- **Verify:** `cargo fmt --check` ✅ + build 21.18s/test 69/69/cargo check 26.30s ✅ + plan recitation presente ✅ + git commit b0d231a7+ ✅ — listo para commit
- **Estado:** ✅ DONE

## Dependencias
- DESKTOP-QW5 ✅ COMPLETED (b0d231a7, limpiar DAUD) — **bloqueante directo**: Wave1 cerrada, Wave2 desbloqueada
- DESKTOP-QW1 ✅ COMPLETED (palette sync, H-02)
- DESKTOP-QW2 ✅ COMPLETED (HelpPanel F1/F2, H-03)
- DESKTOP-QW3 ✅ COMPLETED (statusReport ES, H-05)
- DESKTOP-QW4 ✅ COMPLETED (filterActive, H-14)
- a7ed0d22 — CSP mínima ya aterrizada (H-01 base) + sparse_vector H-04 + F1/F2 + palette + DAUD sync — este task refina connect-src con localhost/https remoto (gap menor post-a7ed0d22)
- Ninguna técnica bloqueante más (Task6 toca tauri.conf.json solo, disjunto de rename H-04 que vive en vanta.ts)

## Notas
- Ponytail: 1 archivo JSON, 2 líneas `connect-src` editadas, ~30 chars añadidos por línea (`http://localhost:* ws://localhost:* https://*`). No añadir `font-src`/`script-src` (Tauri inyecta nonces), no framework CSP, no plugin. Deletion over addition: no ampliar a `http://*` (solo localhost http seguro), no `*` genérico.
- Threat model (security-and-hardening): trust boundary = WebView ↔ proxy fetch + Tauri IPC. Asset = XSS via script injection: mitigado por `default-src 'self'` + `style-src 'self' unsafe-inline` (unsafe-inline necesario para Tailwind, calibrado). `connect-src` solo controla fetch/XHR/WebSocket, no Rust. `https://*` es concesión mínima para proxyUrl dinámico (user input), no exfiltración adicional (ya puede fetch a cualquier https que el user configure; CSP solo limita, no crea canal). No se añade `http://*` para no permitir http plano remoto (downgrade).
- Tauri docs https://v2.tauri.app/security/csp/ confirman: CSP solo activa si seteada, ejemplo `connect-src: ipc: http://ipc.localhost`, nonces/hashes append automático. Nuestra CSP alinea con ejemplo + extiende para Tauri dev (`tauri dev` necesita `http://localhost:* ws://localhost:*` para HMR, ya en devCsp) + proxy remoto.
- E2E flujo-critico no cubre CSP directamente, pero si CSP bloquea `fetch` o `ipc`, E2E fallaría (validación indirecta). Wave plan dice "validar app tras cambio (E2E flujo-critico debe pasar)" — se cumple si build/test verde + manual sanity fetchSnapshot no bloqueado (CSP correcta).
- Campaign system hasTask false para este plan (no MCP registration) → recitation manual en plan file + memory_write (compatible QW1-5).
- Si Step2 se decide SKIP (auditoría dice CSP ya suficiente), documentar justificación y saltar a Step3 verify-only como QW1/QW3/QW4 (audit-only variante). Gate V: 2 fallas mismo error → question al usuario.

## Context Save Point
- **Fecha:** 2026-08-27T04:35
- **Branch:** develop
- **CI pendiente:** ninguno — build 21.18s (2863 modules) + tests 69/69 (18.55s) + cargo check 26.30s + fmt verde
- **Decisiones:** CSP ya mínima desde a7ed0d22 refinada con prod localhost alias + https remoto (ProxyDashboard fetch). Threat model: default-src self anti-XSS, connect-src https://* mínima para proxyUrl user-controlled. Steps 1-4 COMPLETED, plan updated, listo para commit. Audit-only variant con edit mínimo (2 líneas).
- **Problemas conocidos:** ninguno — contrato mecánico verde
- **Próxima tarea:** DESKTOP-QW7 (Wave2 Task7 — Rename namespace preserva sparse_vector H-04) — desbloqueada

