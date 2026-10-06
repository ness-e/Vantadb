---
title: "STRAT-06 — Licencias comparadas (local-first AI + OSS DBs) y ACV de OSS DBs"
kind: research
status: active
description: "Licencias fetch-verificadas (Khoj/Jan/Reor/OpenWebUI/Letta + OSS DBs) y ACV/bands con fuente por número o marcado modelado, con implicaciones para el modelo open-core/OEM; 32 fuentes 2026-10-06"
tags: [vantadb, research, licencias, open-core, acv]
---

# STRAT-06 — Licencias comparadas (local-first AI + OSS DBs) y ACV de OSS DBs

- **Fecha:** 2026-10-06 · **Tipo:** research (STRAT-06, Task 66, plan 0.9.0) · **Cero implementación**
- **Contrato (plan Task 66 L1895):** research doc con licencias comparadas (Khoj/Jan/Reor/OpenWebUI/Letta + OSS DBs relevantes) + ACV/bands de OSS DBs con fuente por número (o marcado "modelado") + implicaciones para el modelo open-core; fuentes citadas (Regla 11).
- **Origen:** fila `STRAT-06` del [Backlog](../Backlog.md) (removida al cierre de esta tarea — registro en [`../avance/activo/operaciones.md`](../avance/activo/operaciones.md)) + [00-SINTESIS-EJECUTIVA2](./validacion/00-SINTESIS-EJECUTIVA2.md) (decisión Apache-2.0 + CLA) + [validacion/01](./validacion/01-competidores-memoria-agentes-ia.md) (Letta/competidores), [02](./validacion/02-bases-datos-embebidas-modelos-negocio.md) (modelos de negocio OSS DBs), [03](./validacion/03-licenciamiento-y-monetizacion-open-core.md) (tabla de licencias), [08](./validacion/08-monetizacion-local-first-sin-capital.md) (monetización sin capital).
- **Alcance:** fundamentar con evidencia externa las decisiones open-core/OEM (licencia + ACV). **No** es plan GTM; **no** re-litiga la decisión Apache-2.0 + CLA — la cita.
- **Método:** fetch-verificación 2026-10-06 (Regla 11 / TSYS-13): GitHub API para licencias y estado de repos; páginas oficiales para pricing y anuncios de ARR; earnings/10-Q y prensa con atribución para ACV. 32 URLs resueltas. Columna `tipo`: **dato** (fuente primaria/oficial) · **reportado** (prensa/estimación de research firm, citada) · **modelado** (derivado propio, marcado).

## Contenido

[§0 Resumen](#0-resumen-ejecutivo) · [§1 Licencias](#1-licencias-comparadas) · [§2 ACV](#2-acv-de-oss-dbs) · [§3 Implicaciones](#3-implicaciones-para-el-modelo-open-coreoem) · [§4 Fuentes](#4-fuentes-verificadas-2026-10-06) · [§5 Lagunas](#5-lagunas)

## §0. Resumen ejecutivo

| Pregunta | Respuesta |
|---|---|
| ¿Qué licencia usa el grupo local-first AI? | Mayormente copyleft fuerte o licencias propias: Khoj AGPL-3.0, Reor AGPL-3.0, OpenWebUI "Open WebUI License" (BSD-3 + marca, no-OSI). La mitad permisiva: Jan Apache-2.0, Letta Apache-2.0. VantaDB (Apache-2.0 + CLA) está alineado con Jan/Letta — no es un outlier. |
| ¿Cuánto factura un OSS DB exitoso (ACV)? | Cohorte enterprise definida como ≥$100k: MongoDB 2,999 clientes (de 70,600), Elastic 1,800+; top ≥$1M: MongoDB 402, Redis 50+. ARR totales: ClickHouse ~$350M, Redis >$300M, Neo4j >$200M, Supabase ~$170M (est. may-2026). |
| ¿Hay OSS DBs con ACV chico? | Turso (MIT) no publica ARR — solo funding (~$15M raised). El patrón general: el ACV enterprise financia el motor gratuito; el self-serve paga $19-25/mes. |
| ¿Qué implica para VantaDB open-core/OEM? | (1) Licencia: mantener Apache-2.0 + CLA es consistente con el segmento; el precedente OpenWebUI muestra la vía "marca > código" sin cerrar el motor. (2) ACV: el salto a $100k+ exige fase 2 (cloud) o el SKU OEM/dual-license (Neo4j: GPL + comercial = $200M+ ARR). (3) Fase 1 ($79 desktop + OEM $500-2k) es micro-ACV pero coste fijo $0 — consistente con la estrategia sin capital de `validacion/08`. |
| ¿Reabre la decisión de licencia? | **No.** La fundamenta: Apache-2.0 + CLA + marca registrada futura queda respaldado por el patrón del segmento y por la vía OpenWebUI; los gatillos de cambio (reseller real / primer enterprise) siguen siendo los de `00-SINTESIS-EJECUTIVA2.md` Q3. |

## §1. Licencias comparadas

### 1.1 Grupo local-first AI (los 5 del contrato)

> Verificado 2026-10-06 vía GitHub API (licencia + estado) y archivos LICENSE oficiales.

| Producto | Licencia (SPDX/detectada) | Modelo de negocio | ⭐ | Estado | Verificado |
|---|---|---|---|---|---|
| **Khoj** (`khoj-ai/khoj`) | **AGPL-3.0** | Cloud **sunset** (2026-04-15); self-host only; pivote a Open Paper + Pipali | 37.6k | Activo | 2026-10-06 |
| **Jan** (`janhq/jan`, Menlo Research) | **Apache-2.0** (LICENSE: "Copyright 2025 Menlo Research"; attribution requested) | App desktop free (6.8M+ downloads — jan.ai, 2026-10-06); sin pricing público | 44.8k | Activo | 2026-10-06 |
| **Reor** (`reorproject/reor`) | **AGPL-3.0** | — (sin monetización) | 8.5k | **Archivado** (último push 2025-05-13) | 2026-10-06 |
| **Open WebUI** (`open-webui/open-webui`) | **"Open WebUI License"** — BSD-3-Clause + cláusula de marca (v0.6.6+, 2025-04-19); **no-OSI** | Free con marca intacta; **enterprise license** para white-label (>50 usuarios) + sponsors | 154.0k | Activo | 2026-10-06 |
| **Letta** (`letta-ai/letta`) | **Apache-2.0** | Free (3 agentes stateful) / **Pro $20/mes** / usage-based / Teams-Enterprise | 25.0k | Activo | 2026-10-06 |

**Notas de cada caso (lo relevante para VantaDB):**

1. **Khoj — el caso más instructivo.** El 2026-04-15 Khoj apagó su servicio cloud (aviso oficial en `app.khoj.dev`) con una explicación que vale como advertencia directa: *"hand-building data integrations was going to be a long tail of effort, especially while there were challenges in defensibility with all the big labs coming out with similar products"* y *"initial bets we made for Khoj — a subscription, cloud-first service, complex document syncs, custom data integrations, multiple (6!) clients — made it very difficult to scale in utility"*. La licencia AGPL-3.0 se mantiene; el proyecto sigue open-source y self-hostable. **Lección:** la suscripción cloud-first con sync complejo es frágil incluso con 37k⭐; la fase 1 local-first de VantaDB evita ese error de secuencia.
2. **Jan — Apache-2.0 en un producto local-first masivo.** Un desktop AI app de 44.8k⭐ con 6.8M+ descargas eligió Apache-2.0 (con pedido de atribución, no obligación) y no publica pricing — su financiación viene de Menlo Research (modelo de investigación). Confirma que Apache-2.0 no bloquea la adopción masiva en local-first.
3. **Reor — el riesgo de mortandad.** App PKM local-first (AGPL) con 8.5k⭐, archivada en 2025 sin modelo de negocio. El nicho "second brain local" es duro; sin distribución/velocidad + un SKU, el proyecto muere.
4. **Open WebUI — el precedente "marca > código".** Con 154k⭐ cambió su licencia (v0.6.6, 2025-04-19) a un BSD-3 + cláusula de branding: uso/modificación/redistribución libres, **pero** no se puede quitar la marca salvo ≤50 usuarios, permiso escrito o **licencia enterprise**; CLA obligatorio para contribuciones nuevas; el código pre-v0.6.6 queda BSD-3. Su documentación lo declara "semi-copyleft", no-OSI. Monetiza white-label + soporte, **sin nube propia**. Es el análogo más cercano a la vía "Apache-2.0 + CLA + marca registrada" que VantaDB ya tiene como opción futura (`00-SINTESIS-EJECUTIVA2.md` Q3).
5. **Letta — Apache-2.0 con escalera cloud.** El precedente `validacion/01:141` (Apache-2.0, $20) sigue vigente: Free 3 agentes BYOK / Pro $20/mes / usage-based. Modelo de memoria con nube propia — el contraste que VantaDB no necesita hoy.

### 1.2 OSS DBs relevantes (licencia del motor)

> Fresh 2026-10-06 (GitHub API / LICENSE) donde se indica; el resto citado del precedente [validacion/02](./validacion/02-bases-datos-embebidas-modelos-negocio.md) y [03](./validacion/03-licenciamiento-y-monetizacion-open-core.md) (fechados 2026-08-25, sin contradicción detectada).

| Proyecto | Licencia motor | Modelo | Verificado |
|---|---|---|---|
| ClickHouse | **Apache-2.0** | Open-core + Cloud (ARR ~$350M, §2) | 2026-10-06 |
| Qdrant | **Apache-2.0** | Open-core + Cloud (Edge embebido gratis) | 2026-10-06 |
| Turso | **MIT** | Open-core + Cloud (sync facturado; el template de `validacion/02`) | 2026-10-06 |
| Weaviate | **Split: BSD-3-Clause + `wl/` enterprise** (license-key; "Weaviate License") | Open-core con directorio enterprise propietario | 2026-10-06 |
| Neo4j | **GPL-3.0** (Community) + comercial (dual licensing) | Dual license clásico; $200M+ ARR (§2) | 2026-10-06 |
| Supabase | Apache-2.0 (core) | Open-core + Cloud (ARR ~$170M est., §2) | 2026-08-25 → re-citado |
| DuckDB | MIT (Foundation) | Open-core + MotherDuck | 2026-08-25 |
| Chroma / LanceDB / Milvus | Apache-2.0 | Open-core + Cloud | 2026-08-25 |
| MongoDB | **SSPL** (desde 2018) | Source-available + Atlas | 2026-08-25 |
| Elastic | **AGPL-3.0 + ELv2 + SSPL** (tri, 2024) | Open-core + Cloud | 2026-08-25 |
| Redis | **AGPL-3.0** (tri-license desde Redis 8, 2025-05) | Open-core + Cloud | 2026-08-25 |
| SQLite | Dominio público | Soporte/warranty + extensiones (SEE ~$2k) | 2026-08-25 |

**Cambio detectado vs precedente:** Weaviate ya no es "BSD-3" simple — el repo hoy declara split BSD-3 + `wl/` enterprise con license-key (GitHub lo detecta como NOASSERTION). Es una variante del patrón "carpeta EE" (PostHog) dentro de un motor OSS.

### 1.3 Lectura para VantaDB

1. **Apache-2.0 no es outlier:** Jan (44.8k⭐) y Letta (25k⭐) operan en Apache-2.0 en el mismo segmento local-first; en el sector memoria Apache-2.0 es el estándar (6/9 — `validacion/01`). La decisión vigente (Apache-2.0 + CLA ligero) queda respaldada.
2. **La licencia no fue el moat de nadie:** Khoj y Reor (AGPL) murieron/pivotaron igual; OpenWebUI y Letta (154k/25k⭐) viven con licencias opuestas. El determinante es distribución + producto + un SKU que pague, no la licencia.
3. **Tres vías de protección de renta observadas, ninguna basada en cerrar el motor:** (a) marca + enterprise license sin nube (OpenWebUI); (b) directorio enterprise con license key (Weaviate `wl/`, PostHog `ee/`); (c) dual license GPL+comercial (Neo4j). Las tres son compatibles con mantener el core abierto y un CLA ligero que preserve la opción.
4. **Cambio de licencia tardío = daño permanente** (Elastic→OpenSearch, Redis→Valkey, `validacion/03`): la ventana barata es hoy (userbase ≈ 0); el gatillo de cambio sigue siendo reseller real / primer enterprise (`00-SINTESIS-EJECUTIVA2.md` Q3).

## §2. ACV de OSS DBs

> **Regla del pre-mortem #1 (plan Task 66): revenue no público → separar "dato verificado" de "modelado"; NO inventar números.** Columna `tipo`: dato (oficial/SEC) · reportado (prensa/estimación de research firm, citada) · modelado (derivado propio).

### 2.1 Datos con fuente por número (verificados 2026-10-06)

| Empresa | Métrica | Valor | Fecha del dato | tipo | Fuente |
|---|---|---|---|---|---|
| ClickHouse | ARR | **>$350M** (+40% desde may-2026; $250M may-2026; $160M fin-2025) | ago-2026 | reportado | dealroom.co (persona cercana) + sacra.com (estimación) |
| Supabase | ARR | **$70M** (sep-2025) → **~$101M** (fin-2025) → **~$170M** (may-2026) | 2025-2026 | reportado | sacra.com + devgraphiq.com (est. Sacra) + techstartups.com |
| MongoDB | Clientes ≥$100k ARR | **2,999** (de 70,600; +17% YoY); NER 122% | jul-2026 | dato | SEC 10-Q (stocktitan.net) |
| MongoDB | Clientes ≥$1M ARR | **402** (+26% YoY) | Q4 FY26 (mar-2026) | dato | finance.yahoo.com (earnings) |
| Elastic | Clientes ≥$100k ACV | **1,800+** (Q1 FY27; 1,660 Q3 FY26; 1,550+ año previo; ~16% YoY) | ago-2026 | dato | stockstofind.com + stocktitan.net (8-K) |
| Neo4j | ARR | **>$200M** (oficial; "doubling over past 3 years"; val. $2B+) | nov-2024 | dato | neo4j.com/press-releases |
| Redis | ARR | **>$300M** (oficial); **12,000+** clientes pagos; **50+** cuentas >$1M (+20% YoY) | ene-2026 | dato (ARR) + reportado (conteos) | GlobeNewswire/Yahoo Finance + sacra.com |
| Turso | Funding | $15M+ raised (Series A: General Catalyst, 8VC, Amplify) | — | reportado | billiondollarpitchdecks.com [confianza media] |

**Lecturas de cohorte (aritmética simple sobre datos de arriba — dato derivado):**

- MongoDB: 2,999/70,600 ≈ **4.2%** de clientes son ≥$100k ARR; 402 son ≥$1M.
- Redis: 50/12,000 ≈ **0.4%** de clientes son >$1M/año.
- Elastic: ~1,800 clientes ≥$100k ACV (base total no publicada en la misma tabla).

### 2.2 Bandas de ACV (modelado — derivado, no dato)

> **Todas las bandas de esta sección son `modelado (derivado)`:** construidas a partir de los puntos de §2.1 + los precios públicos del segmento (`validacion/01`/`02`: entrada $19-25/mes; mid $100-400/mes; enterprise custom). No son cifras de ninguna fuente.

| Banda | Rango (ACV/año) | Ancla/derivación |
|---|---|---|
| Self-serve | $0 – ~$1k | Entrada $19-25/mes (Mem0/Supabase/Letta → $240-300/año) — `modelado` |
| SMB/equipo | ~$1k – $10k | Mid tiers $100-400/mes; equipos pequeños — `modelado` |
| Mid-market | ~$10k – $100k | Banda entre self-serve y la definición de cohorte enterprise — `modelado` |
| Enterprise | ≥$100k | **Definición de cohorte usada por MongoDB (≥$100k ARR) y Elastic (≥$100k ACV)** — la definición es dato; la banda como tal es `modelado` |
| Top | ≥$1M | MongoDB 402 clientes; Redis 50+ (0.4%) — conteo `dato`; banda `modelado` |

### 2.3 Implicaciones para el modelo open-core/OEM de VantaDB

1. **El ACV grande vive en enterprise, no en self-serve.** Los OSS DBs que llegan a $200-350M ARR lo hacen con cohortes ≥$100k (MongoDB/Elastic/Redis). Para VantaDB: la escalera de fase 2 (Free/$19/$249/Enterprise) apunta correctamente arriba; los $19 no construyen el negocio — lo construyen los contratos enterprise/OEM.
2. **El SKU OEM/dual-license tiene ancla de escala:** Neo4j monetiza GPL + comercial a **>$200M ARR** con el mismo patrón que `validacion/08` propone para VantaDB (OEM desde ~$500-2k/año + garantía/indemnidad). Es la vía de ingresos sin infraestructura más probada del set.
3. **La vía "marca > código" (OpenWebUI) es compatible con Apache-2.0 + CLA:** enterprise license por white-label/seats, sin nube. Si VantaDB mantiene Apache-2.0 hoy y registra marca + CLA, la opción "enterprise license de marca" queda abierta sin relicenciar el core.
4. **Fase 1 es micro-ACV por diseño:** Desktop Pro $79 perpetuo + OEM $500-2k (`validacion/08`) están 2-3 órdenes de magnitud por debajo de las cohortes enterprise — consistente con "cero capital" (fase 1) y con el gatillo de fase 2 (≥500 WAU o ~$500 MRR). Este research no cambia esa secuencia; la fundamenta.
5. **La advertencia de Khoj aplica al diseño de fase 2:** no construir sync/integraciones cloud-first complejas antes de que el ACV enterprise las demande (el error de Khoj fue la secuencia: subscription cloud-first antes de product-market fit).

## §3. Implicaciones para el modelo open-core/OEM

> Cita la decisión vigente sin re-litigarla: `00-SINTESIS-EJECUTIVA2.md` Q3 (Apache-2.0 hoy + CLA ligero; cambio solo ante reseller real o primer enterprise) + regla `.opencode/rules/open-core-licensing.md` (core Apache-2.0 nunca relicenciado; Pro fuera del workspace; artefactos compilados; licencia por nodo).

| Decisión vigente | Evidencia de este research | Veredicto |
|---|---|---|
| Core Apache-2.0 | Jan Apache-2.0 (44.8k⭐); Letta Apache-2.0 (25k⭐); Apache-2.0 estándar del sector memoria (6/9 — `validacion/01`) | **Sostenida** — no es outlier |
| CLA ligero desde el primer commit externo | OpenWebUI exige CLA para contribuciones nuevas (v0.6.6+); sin CLA no se puede relicenciar/dual-license | **Sostenida** — es el habilitador de las 3 vías |
| OEM $500-2k/año (fase 1) | Neo4j dual GPL+comercial → $200M+ ARR; Qt/iText/SEE-SQLite (`validacion/08`); OpenWebUI enterprise license por seats | **Anclada** — con precedente de escala |
| Fase 2 Free/$19/$249/Enterprise | MongoDB/Elastic/Redis monetizan cohortes ≥$100k; Supabase $70M→$170M con self-serve+enterprise mixto | **Anclada** — el ACV real vive en enterprise |
| Gatillo de cambio de licencia (reseller/primer enterprise) | Elastic/Redis/MongoDB retrocesos (`validacion/03`); OpenWebUI logró protección vía marca sin cerrar código | **Refinado:** considerar la vía marca/enterprise-license (OpenWebUI) antes que FSL/AGPL si el gatillo se activa |

## §4. Fuentes verificadas (2026-10-06)

| # | Fuente | Qué se extrajo |
|---|---|---|
| 1 | api.github.com/repos/khoj-ai/khoj | AGPL-3.0; 37.6k⭐; activo (push 2026-08-02) |
| 2 | app.khoj.dev | Aviso oficial: **Khoj Cloud sunset 2026-04-15** + razones (subscription cloud-first, sync/integraciones, 6 clients) |
| 3 | raw.githubusercontent.com/janhq/jan/main/LICENSE | Apache-2.0 (© 2025 Menlo Research; attribution requested) |
| 4 | api.github.com/repos/janhq/jan + jan.ai | 44.8k⭐; NOASSERTION (LICENSE inspeccionado); 6.8M+ downloads (homepage) |
| 5 | api.github.com/repos/reorproject/reor | AGPL-3.0; **archived**; último push 2025-05-13 |
| 6 | raw.githubusercontent.com/open-webui/open-webui/main/LICENSE | "Open WebUI License": BSD-3 + cláusula 4 de marca |
| 7 | docs.openwebui.com/license | v0.6.6 (2025-04-19): branding clause, ≤50 usuarios, enterprise license, CLA, no-OSI ("semi-copyleft") |
| 8 | docs.openwebui.com/enterprise | Enterprise License requerida para white-label/rebranding; quoting por seats; sponsors |
| 9 | api.github.com/repos/open-webui/open-webui | 154.0k⭐; NOASSERTION |
| 10 | api.github.com/repos/letta-ai/letta | Apache-2.0; 25.0k⭐ |
| 11 | docs.letta.com/pricing | Free $0 (3 agentes) / Pro $20/mes / usage-based / Teams |
| 12 | api.github.com/repos/ClickHouse/ClickHouse | Apache-2.0; 50.3k⭐ |
| 13 | api.github.com/repos/qdrant/qdrant | Apache-2.0; 34.9k⭐ |
| 14 | api.github.com/repos/tursodatabase/turso | MIT; 24.6k⭐ |
| 15 | raw.githubusercontent.com/weaviate/weaviate/main/LICENSE | Split BSD-3 + `wl/` enterprise (license-key) |
| 16 | api.github.com/repos/neo4j/neo4j | GPL-3.0; 17.3k⭐ |
| 17 | dealroom.co/news/146838-clickhouse-arr-passes-350m-as-ai-agents-fuel-database-demand/ | ARR >$350M (ago-2026), +40% desde may-2026 |
| 18 | sacra.com/c/clickhouse | $350M ago-2026; $160M fin-2025; $15B val. ($400M Series D ene-2026) |
| 19 | sacra.com/research/supabase-at-70m-arr-growing-250-yoy/ | Supabase $70M sep-2025 (+250% YoY) |
| 20 | devgraphiq.com/supabase-statistics/ | ~$101M fin-2025 → ~$170M may-2026 (est. Sacra) |
| 21 | techstartups.com/2025/10/03/supabase-hits-5-billion-valuation-with-100-million-funding-led-by-accel-and-peak-xv/ | $70M run-rate a inversores; $5B val. (oct-2025) |
| 22 | stocktitan.net/sec-filings/MDB/10-q-mongo-db-inc-quarterly-earnings-report-a74c4e4db6c0.html | 2,999 clientes ≥$100k; 70,600 totales; NER 122% |
| 23 | fool.com/earnings/call-transcripts/2026/05/28/mongodb-mdb-q1-2027-earnings-transcript/ | 2,900 clientes ≥$100k (Q1 FY27 — corrobora la serie) |
| 24 | finance.yahoo.com/news/mongodb-q4-earnings-revenues-surpass-140300593.html | 2,799 ≥$100k; 402 ≥$1M |
| 25 | stockstofind.com/news/elastic-reports-15-revenue-growth-in-q1-fy2027-raises-full-year-guidance + stocktitan.net/sec-filings/ESTC/8-k-elastic-n-v-reports-material-event-81421d704244.html | 1,800+ ≥$100k ACV (Q1 FY27); 1,660 (Q3 FY26); NER ~112% |
| 26 | neo4j.com/press-releases/neo4j-revenue-milestone-2024/ (permalink, verificado 2026-10-06) | >$200M ARR (nov-2024, oficial) |
| 27 | finance.yahoo.com/news/redis-passes-300m-annualized-recurring-140000537.html | >$300M ARR (ene-2026, oficial) |
| 28 | sacra.com/c/redis | 12,000+ pagos; 50+ >$1M (+20% YoY) |
| 29 | billiondollarpitchdecks.com/who-funded/turso | $15M+ raised, Series A [confianza media] |
| 30 | validacion/01 · 02 · 03 · 08 · 00-SINTESIS (internas, 2026-08-25) | Letta/competidores; modelos de negocio; tabla de licencias; sin-capital; decisión Q3 |
| 31 | agenticindex.io/vendors/khoj | Khoj Cloud sunset (secundaria — confirmada por la primaria #2) |
| 32 | github.com/sponsors/open-webui | Sponsors como canal fase 0 (mismo patrón que el FUNDING.yml de VantaDB) |

## §5. Lagunas

1. **ClickHouse/Supabase ARR** son estimaciones de research firms / prensa con atribución ("persona cercana") — etiquetados `reportado`; no hay cifra oficial.
2. **Turso no publica ARR** — solo funding (fuente de confianza media). El "template Turso" de `validacion/02` sigue sin cifra de revenue verificada.
3. **OpenWebUI/Jan** no publican revenue ni precio enterprise (quoting por contacto) — su ACV no es verificable hoy.
4. **Neo4j $200M es de nov-2024** (el dato más antiguo del set); no hay actualización oficial 2025-2026 publicada.
5. **Las bandas de §2.2 son derivación propia** (`modelado`) — no citables como dato de industria.
6. **Precios enterprise custom** de MongoDB/Elastic/Redis no son públicos (solo la definición de cohorte ≥$100k y los conteos).
7. **Ventana de validez:** los datos de revenue/pricing cambian cada 6-18 meses — re-verificar antes de decisiones de pricing/OEM que dependan de ellos.

---

*Research generado por vanta-worker (STRAT-06, Task 66, plan 0.9.0), 2026-10-06. Datos de licencias y revenue extraídos de fuentes primarias/oficiales el día de la fecha; ver columna `tipo` y fechas por fila.*
