---
title: Backlog de Negocio — VantaDB
kind: research
status: active
tags: [vantadb, backlog, negocio, gtm, legal]
verified_by: Split ejecutado 2026-09-03 por RES-15-C desde docs/dev/Backlog.md (criterio Gate P). Formato migrado a esquema canónico 10 columnas 2026-09-30
---

# Backlog de Negocio — VantaDB

> **Propósito:** filas del backlog que **no** son ejecutables por agentes: requieren abogado, pago, identidad humana, decisión de negocio o publicación manual. Vivían mezcladas en `docs/dev/Backlog.md` y distorsionaban cualquier métrica de prioridad técnica.
> **Criterio de separación (Gate P, RES-15-C 2026-09-03):** lo que requiere agente/código → técnico (`docs/dev/Backlog.md`); lo que requiere abogado/plata/decisión humana/publicación → aquí.
> **Backlog técnico:** [`docs/dev/Backlog.md`](Backlog.md) — fuente del parser de `/pipeline plan`. Las filas de este archivo **no** entran al triage técnico a propósito (regla documentada en `docs/dev/avance/meta.md`).
> **Formato:** esquema canónico de 10 columnas (`.opencode/references/backlog-format.md`, migrado 2026-09-30).
> **Total open items:** 39 activas (P5 ×2 · P6 ×5 · P8 ×1 · P23 ×6 · GOV ×3 · BIZ ×5 · COBRO ×5 · STRAT+OWNER ×9 · GTM & STEALTH ×3). Tareas no-delegables a agentes (requieren identidad legal, pago, acuerdos comerciales o cuentas externas del propietario). Filas resueltas o descartadas archivadas en el historial de decisiones.

## Criterio por fila borderline (decisiones del split)

| Fila | Decisión | Por qué |
|------|----------|---------|
| `PRO-01..06` | Negocio | Implementables por agentes **cuando arranque Pro** — pero el trigger de inicio es decisión de negocio (repo privado, licensing, pricing). Si Pro arranca y una fila pasa a ejecución por agente, se devuelve a `docs/dev/Backlog.md` |
| `MKT-20` / `MKT-21` (ex MKT-18f/i) | Técnico (NO movidas) | Lado código cerrado y verificado en `docs/dev/Backlog.md`; solo el último paso es humano, sigue siendo ticket de release del pipeline |
| `BLOG-CTA` | Técnico (NO movida) | Fix CTA + metadata + redactar posts 6-7 = contenido markdown escribible por agente en `web/`; sólo publicar es humano, como en todo contenido |
| `DISC-03` | Técnico (NO movida) | ICEBOX — no cuenta como activa |

## P5 — Community (UI manual Discord, no-API-accessible)

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `DISC-01` | — | **Configurar Discord: reaction roles, autorole, logging, welcome DM, onboarding** | `docs/user/discord/todo.md` + assets SVG | 🟡 2-3d | 🟢 Nice-to-have | ⏸️ Bloqueada: Discord UI manual (no API) | Docs + assets OK; config pendiente — requiere Discord UI manual. | Origen: P5 (split RES-15-C) | — |
| `DISC-02` | — | **Discord: AutoMod, stickers/emojis, forums seed** | — | 🟢 4-6h | 🟢 Nice-to-have | ⏸️ Bloqueada: Discord UI manual (no API) | Forums seedeado (9 threads: FAQ/Showcase/Ideas/Bug). AutoMod/stickers/emojis requieren Discord UI manual — no API-accessible. | Origen: P5 (split RES-15-C) | — |

## P6 — Launch Campaign (humanas)

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `LEG-01` | — | **Registrar trademark "VantaDB" (USPTO + EUIPO)** | — | 🟠 semanas · $2-5K | 🔴 Alta | 🆕 Pendiente | Requiere abogado, pago (~$250-350/clase USPTO, ~€850 EUIPO), identidad legal. Estimación original "2-4h" irreal. Registro pendiente en `docs/dev/strategy/GO_TO_MARKET.md` (ya existe, verificado 2026-10-01). | Origen: P6 · Dueño: owner | Dep: abogado + pago |
| `MKT-04` | — | **Publicar 3 drafts de Reddit (r/rust, r/MachineLearning, r/LocalLLaMA)** | `docs/dev/strategy/REDDIT_POSTS.md` | 🟢 2-4h | 🟠 Media | 🆕 Pendiente | Drafts listos (status: ready-to-publish), NUNCA publicados — requieren identidad Reddit del owner. Claims corregidos 2026-09-02. | Origen: P6 · Dueño: owner (humano) | — |
| `CLD-01` | — | **VantaDB Cloud beta on Fly.io** | `docs/dev/strategy/GO_TO_MARKET.md:426` | 🟠 1-2 sem | 🔵 Futuro | 🆕 Pendiente | Checkbox vacío; cero archivos de infra (verificado 2026-10-01: 0 fly.toml/workflows). Requiere cuenta/pago Fly.io + decisión de producto. | Origen: P6 | Dep: decisión producto |
| `CLD-02` | — | **Pitch deck + one-pager** | `docs/dev/strategy/GO_TO_MARKET.md:414` | 🟡 3-5d | 🔵 Futuro | 🆕 Pendiente | Checkbox vacío; cero archivos `*pitch*`/`*deck*` (verificado 2026-10-01). | Origen: P6 | — |
| `CLD-04` | — | **Case study #1 (enterprise pilot)** | `docs/dev/strategy/GO_TO_MARKET.md:415` | 🟠 1 sem | 🔵 Futuro | 🆕 Pendiente | Checkbox vacío; cero archivos (verificado 2026-10-01). Depende de pilot real. | Origen: P6 | Dep: pilot real |

## P8 — Post-Launch & Enterprise

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `BIZ-01b` | — | **Enterprise features: encryption + RBAC + audit ya en crate principal; replication/enterprise crate separado no existen** | — | 🟡 3-5d | 🟡 Baja | 🆕 Pendiente | Audit ya existe (`src/audit.rs`, JSONL opt-in, `src/lib.rs:65`); replication/enterprise crate ausentes (`Cargo.toml:768-781`). | Origen: P8 · verificación HEAD 2026-10-01 | — |

## P23 — VantaDB Pro (Open Core)

> Origen: `docs/dev/strategy/VANTADB-PRO-FEATURES.md` § "Backlog Pro" (detalle técnico en `docs/dev/Backlog.md` §P23). **Implementables por agentes cuando arranque Pro** — el inicio es decisión de negocio (repo privado `vantadb-pro`, pricing, licensing, D5 entrega manual).

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `PRO-01` | — | **Multi-tenancy / RBAC — aislamiento cifras org** | `vantadb-pro`: solo `lib.rs`+`license.rs` | 🔴 2-3 sem | 🔵 Futuro | 🔮 Futuro: arranque Pro | Feature Pro sugerida. | Origen: VANTADB-PRO-FEATURES.md § Backlog Pro | Dep: decisión Pro |
| `PRO-02` | — | **Replicación multi-copy / Sync — DR** | `vantadb-pro`: ídem | 🔴 3-4 sem | 🔵 Futuro | 🔮 Futuro: arranque Pro | Feature Pro sugerida. | Origen: VANTADB-PRO-FEATURES.md | Dep: decisión Pro |
| `PRO-03` | — | **WAL shipping + PITR (gates ya existen en core) — failover** | gate `wal-shipping` en core (`src/lib.rs:162-164`) | 🟠 2-3 sem | 🔵 Futuro | 🔮 Futuro: arranque Pro | Nota 2026-09-14: el gate `pitr` fue removido (FIND-26, ADR-0014 superseded) — al activar Pro, PITR se rediseña, no se reutiliza. | Origen: VANTADB-PRO-FEATURES.md · Ver: FIND-26, ADR-0014 | Dep: decisión Pro |
| `PRO-04` | — | **TTL / retention policies — compliance** | `vantadb-pro`: ídem | 🟡 1-2 sem | 🔵 Futuro | 🔮 Futuro: arranque Pro | Feature Pro sugerida. | Origen: VANTADB-PRO-FEATURES.md | Dep: decisión Pro |
| `PRO-05` | — | **Admin server + dashboard — UX enterprise** | `vantadb-pro`: ídem | 🟠 2-3 sem | 🔵 Futuro | 🔮 Futuro: arranque Pro | Feature Pro sugerida. | Origen: VANTADB-PRO-FEATURES.md | Dep: decisión Pro |
| `PRO-06` | — | **Audit trail / compliance** | `vantadb-pro`: ídem | 🟡 1-2 sem | 🔵 Futuro | 🔮 Futuro: arranque Pro | Feature Pro sugerida. | Origen: VANTADB-PRO-FEATURES.md | Dep: decisión Pro |

## GOV — Acción externa del owner

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `BND-07` | 🟠 Alta | **`vantadb.dev` sin DNS** (GOV-F1; el invite de Discord resuelve hoy — re-verificar antes de tocar superficies) | `README.md`, `CONTRIBUTING.md`, `SECURITY.md` | 🟡 1d | 🟠 Media | ⏸️ Bloqueada: externo owner | Configurar DNS de vantadb.dev (fetch falla 2026-10-01); el invite `discord.gg/g8nqB3NtXt` resuelve ("VantaDB Community", fetch OK 2026-10-01) → mantener/verificar en README. | Origen: auditoría raíz pública GOV-F1 (commit dc3775ef) · Dueño: owner · verificación HEAD 2026-10-01 | — |
| `BIZ-18` | 🔴 Crítica | **Rotar los 6 tokens de publicacion expuestos en texto plano** | `.github/workflows/release.yml`, `release-adapters.yml`, `release-providers.yml` | 🟡 1d | 🔴 Alta | ⏸️ Pospuesta (owner) | Pospuesto explícitamente por decisión del propietario. Tokens aislados temporalmente; la rotación se ejecutará según `docs/dev/workflow/TOKEN-ROTATION.md` previo al próximo ciclo formal de release `v0.10.0`. | Origen: post-mortem del release v0.9.0 (2026-10-20) §7 · Dueño: owner | - |
| `BIZ-19` | 🟠 Alta | **Token de crates.io con scope `publish` para el primer publish de `vanta-memory`** | crates.io (cuenta del owner) | 🟡 1d | 🟠 Media | 🆕 Pendiente | El token actual devuelve 403 al publicar: Trusted Publishing (OIDC) exige que el crate ya exista, asi que el primer publish de `vanta-memory` necesita un token con scope de publicacion. Crear ese token, hacer el primer publish, configurar Trusted Publishing para `vanta-memory` y tirar el token. Verif: `vanta-memory` visible en crates.io y con Trusted Publishing configurado. | Origen: post-mortem del release v0.9.0 (2026-10-20) §7 · Dueño: owner | Dep: `BIZ-18` |

## BIZ — Facturación y negocio (manual estratégico, 2026-09-14)

> Origen: `docs/dev/strategy/VantaDB_Manual_Estrategico_Unificado.md` bloques C1/C2/C5/C6/C8/C10/C14/C15 + research `docs/dev/research/manual-estrategico-validacion-2026-09-14.md` (§1-7). Todas son humanas (cuentas, decisiones, firmas). BIZ-02/03 reservados: en investigación de alcance, NO crear filas hasta definirlo con el owner.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `BIZ-04` | — | **ToS mínimo viable (Terms + Privacy + Refund)** | — | 🟢 1-2d humano | 🔴 Alta | 🆕 Pendiente | MoR/Stripe los exigen para verificación; sin esto no hay cobro. Desde plantillas + adaptación VantaDB. | Origen: manual estratégico §7.3 · Bloquea cualquier cobro (transversal) | — |
| `BIZ-05` | — | **One-pager comercial + propuesta de valor no-técnica** | — | 🟢 1d | 🟠 Media | 🆕 Pendiente | 1 página para design partners/pilotos. Redactable por agente tras fijar ICP. Alcance = 1 one-pager maestro + 3 anexos por track ICP-01..03. | Origen: manual estratégico §7.3 · Desbloqueado 2026-09-24 (BIZ-08 resuelta) | — |
| `BIZ-06` | — | **Jurisdicción y banca fase 2 (LLC + EIN + Mercury)** | — | 🟡 2-3d humano | 🔵 Futuro | 🆕 Pendiente (fase 2) | Solo si el volumen lo justifica (MoR cubre fase 1). Contactar 2-3 proveedores. | Origen: manual estratégico §2 | — |
| `BIZ-07` | — | **Cadena de titularidad IP escalonada** | — | 🟢 1d | 🟠 Media | 🆕 Pendiente | 1) declaración de autoría propia (hoy); 2) DCO/CLA listo; 3) assignment al constituir entidad. Veracidad validada en §7.2 (assignment ≠ CLA ≠ DCO). | Origen: manual estratégico §7.2 | — |
| `BIZ-09` | — | **ADR session-layer defer-as-scoped (resto de DEC-01)** | — | 🟢 1h owner | 🟡 Baja | 🆕 Pendiente (decisión owner) | DEC-01 resuelta por research pero el owner nunca escribió el ADR citando `docs/dev/research/res03-session-layer-gonogo.md`. Detectado en auditoría backlog 2026-09-14. | Origen: auditoría backlog 2026-09-14 · Dueño: owner | — |

## COBRO — Carriles de recaudación (decisión owner 2026-09-24, actualizada 2026-11-03)

> Origen: decisión D4 del usuario sobre Bloques 1, 2 y 3 (`docs/dev/strategy/ROADMAP.md`).
> **Restricciones Reales Innegociables desde Venezuela (VE):**
> 1. **Ubicación y Titularidad:** Residencia física en Venezuela. VantaDB **no tiene entidad jurídica registrada aún** en ninguna jurisdicción extranjera, ni dirección comercial fuera del país. Todas las cuentas, registros y pasarelas operan a **nombre personal del propietario residente en Venezuela**.
> 2. **Plataformas Descartadas por Bloqueo:** **GitHub Sponsors DESCARTADO** (no disponible para Venezuela; no generar `.github/FUNDING.yml` con GitHub Sponsors). Descartados igualmente **Polar.sh**, **Lemon Squeezy** y **Stripe Directo** por bloqueos de Stripe Connect Express o exclusión bancaria en VE.
> 3. **Únicas Pasarelas / Rieles Aprobados y Verificados para Venezuela:**
>    - **Ko-fi** (`http://ko-fi.com/`): donaciones, membresías y propinas internacionales sin intermediación bancaria hostil.
>    - **Patreon** (`https://patreon.com/`): suscripciones recurrentes de patrocinio individual y desarrolladores.
>    - **Binance (USDT / Cripto P2P)**: cobro directo en stablecoins sin fricción para servicios de integración, licencias y clientes directos, con liquidación P2P inmediata.
> 4. **Precios Oficiales Aprobados:**
>    - **VantaDB Core + MCP:** `$0` (Apache-2.0, local, perpetuo e ilimitado).
>    - **VantaDB Studio Pro (Desktop):** `$9.99/mes` (explorador 3D/grafo de recuerdos, editor visual de contradicciones L1-L3, emisión de certificados de purga).
>    - **VantaDB Team Sync (Cloud E2EE):** `$12 / usuario / mes` (sincronización cifrada de extremo a extremo de espacios de memoria de equipo).
>    - **Soporte Comercial / Integración Enterprise:** **Desde $750/mes o ~$1,000/mes según contexto y esfuerzo real requerido** (SLA 48h, hardening on-prem, asistencia dedicada).
> 5. **Infraestructura Web y Waitlist Doble:**
>    - **Hosting Web Principal:** **Vercel** (1ra opción, donde ya existe boceto de landing) y **Cloudflare Pages** (2da opción).
>    - **Dominio:** `vantadb.dev`.
>    - **Estrategia:** Landing ultrarrápida (<500 ms) con titular *"Persistent Local Memory for Coding Agents (Claude Code, Cursor, OpenCode). Zero Docker. One Binary."*, GIF VHS 60s, un-liner de instalación y **Waitlist Doble** (Lanzamiento Oficial de VantaDB + VantaDB Team Memory Sync).

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| `BIZ-10` | — | **Carril cripto directo (Binance Pay USDT + P2P a bolívares)** | — | 🟢 1-2d humano | 🔴 Alta | 🆕 Pendiente | Link/QR de cobro en USDT para design partners/pilotos sin fricción KYC; facturación propia (cross-ref BIZ-13); definir política de reembolsos y registro contable; documentar riesgos fiscales VE. | Origen: decisión D4 · Refs: merchant.binance.com/es-LA/how-to-accept/USDT, pay.binance.com | Dep: BIZ-04 |
| `BIZ-23` | — | **Alta y configuración de Ko-fi (Donaciones y Membresías VE)** | — | 🟢 1d humano | 🔴 Alta | 🆕 Pendiente (2026-11-03) | Configurar cuenta personal en Ko-fi (`http://ko-fi.com/`) como canal primario de propinas y micro-patrocinios para la comunidad open-source. | Rieles VE aprobados | — |
| `BIZ-24` | — | **Alta y configuración de Patreon (Suscripciones Dev VE)** | — | 🟢 1-2d humano | 🔴 Alta | 🆕 Pendiente (2026-11-03) | Configurar tiers de patrocinio mensual en Patreon (`https://patreon.com/`) para desarrolladores y early-adopters de VantaDB. | Rieles VE aprobados | — |
| `BIZ-13` | — | **Facturación manual: plantilla de invoice/recibo + registro de ventas (carriles sin MoR)** | — | 🟢 1d humano | 🟠 Media | 🆕 Pendiente (2026-09-24) | Para cobros cripto/Patreon/Ko-fi directo: numeración, concepto, fecha, método, tasa; cuaderno de ventas para la meta USD 5.000. | Origen: decisión D4 · Cross-ref BIZ-04 | — |
| `BIZ-22` | — | **Constitución LLC extranjera (desbloqueo bancario internacional)** | — | 🟡 1-2 sem | 🔵 Futuro | 🔮 Futuro (fase 2) | Constituir entidad en el extranjero (Wyoming/Delaware/Estonia) solo si el volumen de ingresos justifica los costes de mantenimiento fiscal y contable. | Fase 2 banca y escala | — |

## STRAT + OWNER — movidas de `Backlog.md` (2026-09-30)

> Movidas por criterio de separación (Gate P): decisiones de negocio/owner y puertas humanas que no entran al triage técnico (auditoría de backlogs 2026-09-30). Registro de la mudanza: `docs/dev/avance/activo/operaciones.md`.
> **Nota de coexistencia de arquitectura:** `VantaDB Studio` (aplicación de escritorio Tauri + frontend web) reside en el worktree independiente `develop-desktop`. En la rama `develop` residen exclusivamente los crates Rust del motor, bindings oficiales (Python, Node, TS, WASM) y el arnés de evals.
> **Nota:** `STRAT-01` (Posicionamiento *"SQLite for agents"*) resuelta y aplicada el 2026-10-31.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| `STRAT-02` | — | **Decisión: monetización** | — | — | 🔴 Alta | 🆕 Pendiente (owner) | Soporte/consulting (único viable como individuo) vs motor embebido de terceros (único que escala) vs hosted (infra, no producto). | Origen: DELTA P3 · Dueño: owner | — |
| `STRAT-03` | — | **Entidad legal + indemnity** | — | — | 🟠 Media | 🆕 Pendiente (negocio) | Techo estructural del comprador enterprise. | Origen: DELTA P3 | — |
| `STRAT-07` | — | **Decisiones owner de campaña** | — | — | 🟠 Media | 🆕 Pendiente (owner) | Q5 (enforcement p99) + calibración runtime de confianza (VER-08 entregó el harness; aplicarla es v0.10.0 / v1.0). | Origen: DELTA P3 · Dueño: owner | — |
| `DX-10` | — | **H-015: verificación MCP con cliente real (humano)** | — | 🟢 5min | 🟠 Media | 🆕 Pendiente (humano) | Conectar un cliente MCP real (Claude/Cursor/OpenCode) a la DB demo; 5 minutos que cierran el último eslabón del track AI-IDEs (pipe verificado: 80 tools listadas — 86 definidas − 6 absorbidas WIRE-02, `docs/api/MCP.md:198`; roundtrip OK). | Origen: H-findings 2026-09-30 · Dueño: owner · verificación HEAD 2026-10-01 | — |
| `FASE-A/R-05` | — | **Puertas owner: gate Fase A + R-05** (consolida `EXE-03` + `EST-12`) | `docs/dev/FASE-A.md` (A.1-A.3) | 🟡 1sem | 🔴 Alta | 🆕 Pendiente (owner) | Correr el gate: 5 installs limpias documentadas + stranger-test 2-3 personas + 0 críticos → veredicto GO/NO-GO de anuncio. Merge #222/tag/publish ✅ 2026-09-25. | Origen: DELTA · Ver: EXE-03, EST-12 (consolidadas) | — |
| `BIZ-14` | — | **Backup offsite (S3/red)** | — | 🟡 2-3d | 🟠 Media | 🔮 Futuro: trigger Pro/Cloud | Exportar instantáneas `.vantadb` a almacenamiento de red (rescatado del Icebox OLD ROAD-02, 2026-09-30); gap DR real. | Origen: Icebox OLD ROAD-02 | Dep: PRO-02/03 |
| `BIZ-15` | — | **Página pública `/licensing`** (qué es gratis vs pago) | — | 🟢 1d | 🟠 Media | 🆕 Pendiente (website) | Reduce fricción comercial pre-Desktop Pro; era P1 en la síntesis OLD y no existe. | Origen: síntesis OLD 2026-06 | — |
| `BIZ-16` | — | **Firmar binarios Windows con Authenticode** | — | 🟠 2-3d técnico + coste cert | 🟠 Media | 🆕 Pendiente | Elimina el aviso SmartScreen y permite adopción enterprise (cualquier pipeline corporativo rechaza binarios no firmados). Acciones: (a) certificado de firma de código EV; (b) `signtool` en el workflow de release; (c) checksums SHA-256 firmados con GPG. | Origen: R7 análisis 2026-07-26 (`VantaDB_AnalisisTecnico_BusinessProfessional`) · Dueño: owner | Dep: certificado EV (compra) |
| `BIZ-17` | — | **Auditoría externa de seguridad** | — | 🔴 4-8sem + coste | 🟡 Media | 🆕 Pendiente | Validar la postura de seguridad más allá del reporte responsable: terceros (p. ej. Cure53, Trail of Bits, NCC Group) centrada en criptografía, WAL, parser y servidor HTTP; publicar el informe; tracker público de hallazgos. | Origen: R8 análisis 2026-07-26 · Dueño: owner | Dep: contratación + presupuesto |

## GTM & STEALTH DISCOVERY — Campaña de Validación y Lanzamiento (Bloques 6 y 7 — 2026-11-03)

> Origen: [`docs/dev/strategy/STEALTH_CUSTOMER_DISCOVERY_MOM_TEST.md`](strategy/STEALTH_CUSTOMER_DISCOVERY_MOM_TEST.md) y [`docs/dev/strategy/LAUNCH_PLAYBOOK_AND_POSTS_KIT.md`](strategy/LAUNCH_PLAYBOOK_AND_POSTS_KIT.md). Todas requieren publicación e interacción humana del propietario con su cuenta personal.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| `GTM-06` | — | **Ejecutar Semana 1 de Stealth Customer Discovery (The Mom Test) en Reddit** | `docs/dev/strategy/STEALTH_CUSTOMER_DISCOVERY_MOM_TEST.md` | 🟡 5h/sem humano | 🔴 Alta | 🆕 Lista para ejecutar | Publicar los 4 posts encubiertos en `r/LocalLLaMA`, `r/ClaudeAI`, `r/cursor` y `r/LangChain` sin mencionar VantaDB. Registrar respuestas en la matriz de hipótesis `H1`–`H5` y aplicar el semáforo de validación (🟢/🟡/🔴). | Bloque 7 §1-4 | — |
| `GTM-07` | — | **Prospección pasiva por DM a leads B2B detectados en el Mom Test** | `docs/dev/strategy/STEALTH_CUSTOMER_DISCOVERY_MOM_TEST.md` §5 | 🟢 2h/sem humano | 🔴 Alta | 🆕 Lista para ejecutar | Contactar por DM a CTOs/Staff Engineers que reporten dolor agudo de latencia, costos de tokens o cumplimiento GDPR/EU AI Act, ofreciendo piloto/integración directa ($750–$1,000/mes pagaderos en USDT vía Binance Pay). | Bloque 7 §5 · BIZ-10 | Dep: `GTM-06` |
| `GTM-08` | — | **Ejecutar el Playbook Secuencial de Lanzamiento (Días 1 a 10)** | `docs/dev/strategy/LAUNCH_PLAYBOOK_AND_POSTS_KIT.md` | 🟡 10d campaña | 🔴 Alta | ⏸️ Bloqueada por Gate D2 | Ejecutar en orden: Día 1 (`r/LocalLLaMA`), Día 2 (`r/rust`), Día 3 (`r/ClaudeAI` + `r/cursor`), Día 5 (Dev.to + Hashnode), Día 7 (`Show HN`), Día 8 (Hilo en X + LinkedIn), Día 10 (Envío de PRs a los 5 directorios MCP y Awesome Lists). | Bloque 6 §4-5 · MKT-04 | Dep: `ACT-01`, `ACT-02` (`recall_all@5 >= 0.85`), `GTM-01` |

---

> **Revisión 2026-11-03 (Consolidación Bloques 1 al 7):** Integrados los rieles exclusivos para Venezuela (Ko-fi, Patreon, Binance USDT), descartados GitHub Sponsors/Polar/Lemon Squeezy, añadida sección GTM & Stealth Discovery (`GTM-06..08`), y resuelta `STRAT-01` ("SQLite for agents").
