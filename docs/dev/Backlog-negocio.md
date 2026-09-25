---
title: "Backlog de Negocio — VantaDB"
type: backlog-tracking
status: active
tags: [vantadb, backlog, negocio, gtm, legal]
last_reviewed: 2026-09-03
verified_by: "Split ejecutado 2026-09-03 por RES-15-C desde docs/dev/Backlog.md (criterio Gate P)"
---

# Backlog de Negocio — VantaDB

> **Propósito:** filas del backlog que **no** son ejecutables por agentes: requieren abogado, pago, identidad humana, decisión de negocio o publicación manual. Vivían mezcladas en `docs/dev/Backlog.md` y distorsionaban cualquier métrica de prioridad técnica.
> **Criterio de separación (Gate P, RES-15-C 2026-09-03):** lo que requiere agente/código → técnico (`docs/dev/Backlog.md`); lo que requiere abogado/plata/decisión humana/publicación → aquí.
> **Backlog técnico:** [`docs/dev/Backlog.md`](Backlog.md) — fuente del parser de `/pipeline plan`. Las filas de este archivo **no** entran al triage técnico a propósito (regla documentada en `docs/dev/avance/meta.md`).
> **Total open items:** 20 activas (15 orig. 2026-09-03 + BIZ-04..08 del manual estratégico 2026-09-14; regla anti-drift GOV-C7) + 4 nuevas BIZ-10..13 (carriles de cobro, decisión owner 2026-09-24)

## Criterio por fila borderline (decisiones del split)

| Fila | Decisión | Por qué |
|------|----------|---------|
| `PRO-01..06` | Negocio | Implementables por agentes **cuando arranque Pro** — pero el trigger de inicio es decisión de negocio (repo privado, licensing, pricing). Si Pro arranca y una fila pasa a ejecución por agente, se devuelve a `docs/dev/Backlog.md` |
| `MKT-18f` / `MKT-18i` | Técnico (NO movidas) | Lado código cerrado y verificado en `docs/dev/Backlog.md`; solo el último paso es humano, sigue siendo ticket de release del pipeline |
| `BLOG-CTA` | Técnico (NO movida) | Fix CTA + metadata + redactar posts 6-7 = contenido markdown escribible por agente en `web/`; sólo publicar es humano, como en todo contenido |
| `DISC-03` | Técnico (NO movida) | ICEBOX — no cuenta como activa |

## P5 — Community (UI manual Discord, no-API-accessible)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado Real |
|----|-------------|----------|----------|------|-------------|
| `DISC-01` | **Configurar Discord: reaction roles, autorole, logging, welcome DM, onboarding** | `docs/user/discord/todo.md` + assets SVG + server activo | 🟡 2-3d | 🟢 | ⚠️ Docs + assets OK. Config pendiente — requiere Discord UI manual |
| `DISC-02` | **Discord: AutoMod, stickers/emojis, forums seed** | — | 🟢 4-6h | 🟢 | ⚠️ Forums seedeado (9 threads: FAQ/Showcase/Ideas/Bug). AutoMod/stickers/emojis requieren Discord UI manual — no API-accessible |

## P6 — Launch Campaign (humanas)

| ID | Descripción | Estimación real | Prioridad | Estado Real |
|----|-------------|-----------------|-----------|-------------|
| `LEG-01` | **Registrar trademark "VantaDB" (USPTO + EUIPO)** — Requiere abogado, pago (~$250-350/clase USPTO, ~€850 EUIPO), identidad legal. Estimación original "2-4h" irreal. | semanas, $2-5K | 🔴 | ❌ No iniciado — mover a `docs/dev/strategy/GO_TO_MARKET.md` cuando exista |

| ID | Descripción | Esfuerzo | Prio | Estado Real |
|----|-------------|----------|------|-------------|
| `MKT-04` | **Publicar 3 drafts de Reddit (r/rust, r/MachineLearning, r/LocalLLaMA)** — drafts listos en `docs/dev/strategy/REDDIT_POSTS.md` (status: ready-to-publish), NUNCA publicados — requieren identidad Reddit del owner. Claims corregidos 2026-09-02 (ver REDDIT_POSTS.md). | 🟢 2-4h | 🟠 | ❌ Pendiente (humano) |
| `CLD-01` | **VantaDB Cloud beta on Fly.io** — checkbox vacío en `GO_TO_MARKET.md:420`; cero archivos de infra. Verificado 2026-08-17: no existe nada. Requiere cuenta/pago Fly.io + decisión de producto. | 🟠 1-2 sem | 🔵 | ❌ Pendiente |
| `CLD-02` | **Pitch deck + one-pager** — checkbox vacío en `GO_TO_MARKET.md:408`; cero archivos `*pitch*`/`*deck*`. | 🟡 3-5d | 🔵 | ❌ Pendiente |
| `CLD-04` | **Case study #1 (enterprise pilot)** — checkbox vacío en `GO_TO_MARKET.md:409`; cero archivos. Depende de pilot real. | 🟠 1 sem | 🔵 | ❌ Pendiente |

## P8 — Post-Launch & Enterprise

| ID | Descripción | Esfuerzo | Prio |
|----|-------------|----------|------|
| `BIZ-01b` | **Enterprise features: encryption + RBAC ya en crate principal. Audit/replication/enterprise crate separado no existen** | 🟡 3-5d | 🟡 ⏳ |

## P23 — VantaDB Pro (Open Core)

> Origen: `docs/dev/strategy/VANTADB-PRO-FEATURES.md` § "Backlog Pro" (detalle técnico en `docs/dev/Backlog.md` §P23). **Implementables por agentes cuando arranque Pro** — el inicio es decisión de negocio (repo privado `vantadb-pro`, pricing, licensing, D5 entrega manual).

| ID | Descripción (Feature Pro sugerida → qué clava) | Código actual | Esfuerzo | Prio | Estado |
|----|-------------|----------|------|--------|--------|
| `PRO-01` | **Multi-tenancy / RBAC** — aislamiento cifras org | `vantadb-pro`: solo `lib.rs`+`license.rs` | 🔴 2-3 sem | 🔵 | ❌ Sin código |
| `PRO-02` | **Replicación multi-copy / Sync** — DR | ídem | 🔴 3-4 sem | 🔵 | ❌ Sin código |
| `PRO-03` | **WAL shipping + PITR (gates ya existen en core)** — failover | gate `wal-shipping` en core (`src/lib.rs:155-156`); nota 2026-09-14: el gate `pitr` fue removido (FIND-26, ADR-014 superseded) — al activar Pro, PITR se rediseña, no se reutiliza | 🟠 2-3 sem | 🔵 | ❌ Sin código |
| `PRO-04` | **TTL / retention policies** — compliance | ídem | 🟡 1-2 sem | 🔵 | ❌ Sin código |
| `PRO-05` | **Admin server + dashboard** — UX enterprise | ídem | 🟠 2-3 sem | 🔵 | ❌ Sin código |
| `PRO-06` | **Audit trail / compliance** — ídem | ídem | 🟡 1-2 sem | 🔵 | ❌ Sin código |

## GOV — Acción externa del owner

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `BND-07` | **Discord invite inválido + vantadb.dev sin DNS** (GOV-F1 🔴×2) — requieren acción externa del owner: crear invite nuevo de Discord y configurar DNS de vantadb.dev; luego actualizar README/CONTRIBUTING/SECURITY con los valores reales. Registrado en auditoría raíz pública GOV-F1 (commit dc3775ef). | README.md, CONTRIBUTING.md, SECURITY.md (externo al repo) | 🟡 | 🟠 | ⏳ Externo owner |

## BIZ — Facturación y negocio (manual estratégico, 2026-09-14)

> Origen: `docs/dev/strategy/VantaDB_Manual_Estrategico_Unificado.md` bloques C1/C2/C5/C6/C8/C10/C14/C15 + research `docs/dev/research/manual-estrategico-validacion-2026-09-14.md` (§1-7). Todas son humanas (cuentas, decisiones, firmas). BIZ-02/03 reservados: en investigación de alcance, NO crear filas hasta definirlo con el owner.

| ID | Descripción | Esfuerzo | Prio | Estado Real |
|----|-------------|----------|------|-------------|
| `BIZ-04` | **ToS mínimo viable (Terms + Privacy + Refund)** — MoR/Stripe los exigen para verificación; sin esto no hay cobro. Desde plantillas + adaptación VantaDB. Research: §7.3. | 🟢 1-2d humano | 🔴 | 🆕 Pendiente (investigación profunda en curso) |
| `BIZ-05` | **One-pager comercial + propuesta de valor no-técnica** — 1 página para design partners/pilotos. Redactable por agente tras fijar ICP (dep: BIZ-08). Research: §7.3. | 🟢 1d | 🟠 | 🆕 Pendiente (desbloqueado 2026-09-24: BIZ-08 resuelta; alcance = 1 one-pager maestro + 3 anexos por track ICP-01..03) |
| `BIZ-06` | **Jurisdicción y banca fase 2 (LLC + EIN + Mercury)** — solo si el volumen lo justifica (MoR cubre fase 1). Contactar 2-3 proveedores. Research: §2. | 🟡 2-3d humano | 🔵 | 🆕 Pendiente (fase 2) |
| `BIZ-07` | **Cadena de titularidad IP escalonada** — 1) declaración de autoría propia (hoy); 2) DCO/CLA listo; 3) assignment al constituir entidad. Veracidad validada en §7.2 (assignment ≠ CLA ≠ DCO). | 🟢 1d | 🟠 | 🆕 Pendiente (investigación profunda en curso) |
| `BIZ-08` | **Decisión ICP/vertical foco** — ✅ **RESUELTA 2026-09-24 (owner):** desarrollar los **3 tracks a profundidad** — (1) AI-IDEs vía MCP (Cursor/Claude Code/OpenCode), (2) devs local-LLM y privacidad, (3) frameworks (LangChain/LlamaIndex) — el núcleo es único (motor de memoria embebido gobernado) y cada track es puerta de entrada. Materializa en P54 (ICP-01..03). Desbloquea BIZ-05, MGR-20, EXE-03. | 🟢 2h owner | 🔴 | ✅ Resuelta 2026-09-24 |
| `BIZ-09` | **ADR session-layer defer-as-scoped (resto de DEC-01)** — DEC-01 resuelta por research pero el owner nunca escribió el ADR citando `docs/dev/research/res03-session-layer-gonogo.md`. Detectado en auditoría backlog 2026-09-14 (no existe ADR session-* en `docs/dev/architecture/adr/`). | 🟢 1h owner | 🟡 | 🆕 Pendiente (decisión owner) |

## COBRO — Carriles de recaudación (decisión owner 2026-09-24)

> Origen: decisión D4 del plan post-investigación (`docs/dev/plans/2026-09-24-post-investigacion-integral.md` §F) + validación de fees en `docs/dev/research/manual-estrategico-validacion-2026-09-14.md` §1/§7.1. Realidad del owner: **PayPal + Binance operativos; Payoneer próxima a crear**. BIZ-02/03 (research de alcance de rieles) quedan cubiertos por BIZ-10..12 — no se crean filas separadas. **BIZ-04 (ToS/Privacy/Refund) bloquea cualquier cobro** (transversal, prioridad 🔴).

| ID | Descripción | Esfuerzo | Prio | Estado Real |
|----|-------------|----------|------|-------------|
| `BIZ-10` | **Carril cripto directo (Binance Pay USDT + P2P a bolívares)** — link/QR de cobro en USDT para design partners/pilotos sin fricción KYC; facturación propia (cross-ref BIZ-13); definir política de reembolsos y registro contable; documentar riesgos fiscales VE. Refs: merchant.binance.com/es-LA/how-to-accept/USDT, pay.binance.com. | 🟢 1-2d humano | 🔴 | 🆕 Pendiente (2026-09-24) |
| `BIZ-11` | **Verificar Merchant of Record con payouts PayPal para merchant VE** — pregunta directa a soporte: Lemon Squeezy (200+ países vía PayPal; bank payouts 79 países con VE no listado) + evaluar Polar/Creem como MoR; documentar fees reales y decisión GO/NO-GO con la cuenta PayPal existente. Refs: docs.lemonsqueezy.com/help/getting-started/supported-countries; validación §7.1. | 🟢 1-2d humano | 🔴 | 🆕 Pendiente (2026-09-24) |
| `BIZ-12` | **Alta Payoneer + ruta Payoneer→Airtm→banco VE** — crear cuenta al volumen actual y verificar retiro de prueba a banco local (Banesco/Mercantil/Provincial/BOD/Bancaribe); documentar fees y tiempos. Refs: payoneer.com/es/resources/business/payoneer-y-airtm · airtm.com/es/blog/economy/payoneer-en-venezuela. | 🟢 2-3d humano | 🟠 | 🆕 Pendiente (2026-09-24; depende de crear la cuenta) |
| `BIZ-13` | **Facturación manual: plantilla de invoice/recibo + registro de ventas (carriles sin MoR)** — para cobros cripto/Payoneer/PayPal directo: numeración, concepto, fecha, método, tasa; cuaderno de ventas para la meta USD 5.000 (manual estratégico). Cross-ref BIZ-04 (ToS/Privacy/Refund base). | 🟢 1d humano | 🟠 | 🆕 Pendiente (2026-09-24) |
