---
title: "Backlog de Negocio — VantaDB"
type: backlog-tracking
status: active
tags: [vantadb, backlog, negocio, gtm, legal]
last_reviewed: 2026-09-03
verified_by: "Split ejecutado 2026-09-03 por RES-15-C desde docs/Backlog.md (criterio Gate P)"
---

# Backlog de Negocio — VantaDB

> **Propósito:** filas del backlog que **no** son ejecutables por agentes: requieren abogado, pago, identidad humana, decisión de negocio o publicación manual. Vivían mezcladas en `docs/Backlog.md` y distorsionaban cualquier métrica de prioridad técnica.
> **Criterio de separación (Gate P, RES-15-C 2026-09-03):** lo que requiere agente/código → técnico (`docs/Backlog.md`); lo que requiere abogado/plata/decisión humana/publicación → aquí.
> **Backlog técnico:** [`docs/Backlog.md`](Backlog.md) — fuente del parser de `/pipeline plan`. Las filas de este archivo **no** entran al triage técnico a propósito (regla documentada en `docs/avance/meta.md`).
> **Total open items:** 15 activas (verificadas contra `rg -c "^\| \`?[A-Z]" docs/Backlog-negocio.md` el 2026-09-03; regla anti-drift GOV-C7 — actualizar este contador con `rg` en cada movimiento)

## Criterio por fila borderline (decisiones del split)

| Fila | Decisión | Por qué |
|------|----------|---------|
| `PRO-01..06` | Negocio | Implementables por agentes **cuando arranque Pro** — pero el trigger de inicio es decisión de negocio (repo privado, licensing, pricing). Si Pro arranca y una fila pasa a ejecución por agente, se devuelve a `docs/Backlog.md` |
| `MKT-18f` / `MKT-18i` | Técnico (NO movidas) | Lado código cerrado y verificado en `docs/Backlog.md`; solo el último paso es humano, sigue siendo ticket de release del pipeline |
| `BLOG-CTA` | Técnico (NO movida) | Fix CTA + metadata + redactar posts 6-7 = contenido markdown escribible por agente en `web/`; sólo publicar es humano, como en todo contenido |
| `DISC-03` | Técnico (NO movida) | ICEBOX — no cuenta como activa |

## P5 — Community (UI manual Discord, no-API-accessible)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado Real |
|----|-------------|----------|----------|------|-------------|
| `DISC-01` | **Configurar Discord: reaction roles, autorole, logging, welcome DM, onboarding** | `docs/discord/todo.md` + assets SVG + server activo | 🟡 2-3d | 🟢 | ⚠️ Docs + assets OK. Config pendiente — requiere Discord UI manual |
| `DISC-02` | **Discord: AutoMod, stickers/emojis, forums seed** | — | 🟢 4-6h | 🟢 | ⚠️ Forums seedeado (9 threads: FAQ/Showcase/Ideas/Bug). AutoMod/stickers/emojis requieren Discord UI manual — no API-accessible |

## P6 — Launch Campaign (humanas)

| ID | Descripción | Estimación real | Prioridad | Estado Real |
|----|-------------|-----------------|-----------|-------------|
| `LEG-01` | **Registrar trademark "VantaDB" (USPTO + EUIPO)** — Requiere abogado, pago (~$250-350/clase USPTO, ~€850 EUIPO), identidad legal. Estimación original "2-4h" irreal. | semanas, $2-5K | 🔴 | ❌ No iniciado — mover a `docs/strategy/GO_TO_MARKET.md` cuando exista |

| ID | Descripción | Esfuerzo | Prio | Estado Real |
|----|-------------|----------|------|-------------|
| `MKT-04` | **Publicar 3 drafts de Reddit (r/rust, r/MachineLearning, r/LocalLLaMA)** — drafts listos en `docs/strategy/REDDIT_POSTS.md` (status: ready-to-publish), NUNCA publicados — requieren identidad Reddit del owner. Claims corregidos 2026-09-02 (ver REDDIT_POSTS.md). | 🟢 2-4h | 🟠 | ❌ Pendiente (humano) |
| `CLD-01` | **VantaDB Cloud beta on Fly.io** — checkbox vacío en `GO_TO_MARKET.md:420`; cero archivos de infra. Verificado 2026-08-17: no existe nada. Requiere cuenta/pago Fly.io + decisión de producto. | 🟠 1-2 sem | 🔵 | ❌ Pendiente |
| `CLD-02` | **Pitch deck + one-pager** — checkbox vacío en `GO_TO_MARKET.md:408`; cero archivos `*pitch*`/`*deck*`. | 🟡 3-5d | 🔵 | ❌ Pendiente |
| `CLD-04` | **Case study #1 (enterprise pilot)** — checkbox vacío en `GO_TO_MARKET.md:409`; cero archivos. Depende de pilot real. | 🟠 1 sem | 🔵 | ❌ Pendiente |

## P8 — Post-Launch & Enterprise

| ID | Descripción | Esfuerzo | Prio |
|----|-------------|----------|------|
| `BIZ-01b` | **Enterprise features: encryption + RBAC ya en crate principal. Audit/replication/enterprise crate separado no existen** | 🟡 3-5d | 🟡 ⏳ |

## P23 — VantaDB Pro (Open Core)

> Origen: `docs/strategy/VANTADB-PRO-FEATURES.md` § "Backlog Pro" (detalle técnico en `docs/Backlog.md` §P23). **Implementables por agentes cuando arranque Pro** — el inicio es decisión de negocio (repo privado `vantadb-pro`, pricing, licensing, D5 entrega manual).

| ID | Descripción (Feature Pro sugerida → qué clava) | Código actual | Esfuerzo | Prio | Estado |
|----|-------------|----------|------|--------|--------|
| `PRO-01` | **Multi-tenancy / RBAC** — aislamiento cifras org | `vantadb-pro`: solo `lib.rs`+`license.rs` | 🔴 2-3 sem | 🔵 | ❌ Sin código |
| `PRO-02` | **Replicación multi-copy / Sync** — DR | ídem | 🔴 3-4 sem | 🔵 | ❌ Sin código |
| `PRO-03` | **WAL shipping + PITR (gates ya existen en core)** — failover | gates `wal-shipping`/`pitr` en core (`src/lib.rs:138,142`) | 🟠 2-3 sem | 🔵 | ❌ Sin código |
| `PRO-04` | **TTL / retention policies** — compliance | ídem | 🟡 1-2 sem | 🔵 | ❌ Sin código |
| `PRO-05` | **Admin server + dashboard** — UX enterprise | ídem | 🟠 2-3 sem | 🔵 | ❌ Sin código |
| `PRO-06` | **Audit trail / compliance** — ídem | ídem | 🟡 1-2 sem | 🔵 | ❌ Sin código |

## GOV — Acción externa del owner

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `BND-07` | **Discord invite inválido + vantadb.dev sin DNS** (GOV-F1 🔴×2) — requieren acción externa del owner: crear invite nuevo de Discord y configurar DNS de vantadb.dev; luego actualizar README/CONTRIBUTING/SECURITY con los valores reales. Registrado en auditoría raíz pública GOV-F1 (commit dc3775ef). | README.md, CONTRIBUTING.md, SECURITY.md (externo al repo) | 🟡 | 🟠 | ⏳ Externo owner |
