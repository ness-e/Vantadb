# VantaDB — Síntesis Ejecutiva Final v2: Competencia, Monetización Sin Nube y Estrategia de Producto

**Fecha:** 2026-08-25 · **v2 — corrige y reemplaza a `validacion/00-SINTESIS-EJECUTIVA.md`**
**Premisas corregidas:** VantaDB es una BD de memoria para IA **100% embebida local-first (sin nube por ahora)** y opera con **cero capital**: ningún coste fijo mensual; el cobro solo puede basarse en plataformas que cobran % sobre ventas reales, hasta tener clientes.
**Método:** 7 investigaciones paralelas (docs 01–07, archivados en `validacion/`) + reinvestigación focalizada de monetización sin infraestructura (doc `08`).

---

## Resumen ejecutivo

VantaDB es **técnicamente superior a lo que su distribución refleja**: motor Rust embebido con búsqueda híbrida real (BM25+HNSW+RRF), grafo+GraphRAG, SDKs Python/TS/WASM que corre hasta en el navegador — con **3★ y ~347 descargas/mes** frente a Mem0 (64k⭐, $24M). El mercado ya validó la demanda del nicho embebido-agéntico (Qdrant Edge, LanceDB default en CrewAI, narrativa agentic de Turso) y nadie ha consolidado "el SQLite de la memoria de agentes". La ventana existe y se cierra.

**Las 5 decisiones recomendadas (versión sin nube y sin capital):**

| # | Decisión | Recomendación |
|---|---|---|
| 1 | **Licencia** | **Mantener Apache-2.0 HOY** + CLA ligero desde el primer commit externo. La ventana para cambiar a FSL/AGPL+dual existe solo mientras la userbase ≈ 0; decidir si aparece un reseller o antes del primer enterprise. No gastar en abogados todavía |
| 2 | **Producto de pago #1** | **VantaDB Desktop Pro: $79 pago único (licencia perpetua)** — anclado en TablePlus ($99/$129 verificado). La app Tauri ya existe; el Pro añade grafo visual, multi-BD, backups/export manager, benchmarks |
| 3 | **Sistema de cobro** | GitHub Sponsors hoy ($0) → **Lemon Squeezy al lanzar Desktop Pro (5% + $0.50 solo por venta; license keys incluidas)** → Stripe directo cuando MRR >$1k. Coste fijo siempre $0 |
| 4 | **Otros SKUs sin servidores** | Licencia OEM/comercial B2B por cotización (~$500–2k/año) + soporte/warranty (modelo SQLite). Ambos son papel, no infraestructura |
| 5 | **Fase 2 (con ingresos)** | Construir cloud sync/backups/embeddings financiado por fase 1 → activar la escalera Free/$19/$249/Enterprise ya diseñada (doc 03, intacta como plan futuro) |

---

## 1. Panorama competitivo (datos válidos, interpretación corregida)

### Memoria para agentes IA (competencia directa)

| Producto | Licencia | ⭐ | Free | Entrada | Mid | Cobro por |
|---|---|---|---|---|---|---|
| Mem0 | Apache-2.0 | 64.0k | 10k add/mes | $19 | $249 | requests |
| Zep/Graphiti | Apache-2.0 | 30.3k | 10k créditos | $125 | $375 | créditos ingesta |
| Cognee | Apache-2.0 | 30.3k | 1M tokens | $5/ws | — | tokens/workspaces |
| Supermemory | MIT | 29.1k | ~$5 uso | $19 | $100/$399 | SM-tokens |
| Letta | Apache-2.0 | 24.4k | 3 agentes | $20 | créditos | compute |
| memU | Apache* | 14.3k | ilimitado cloud | — | — | aún no |
| Honcho | AGPL-3.0 | 6.8k | retrieval gratis | usage | usage | ingesta $2/M |

Muertos: Memary, Memobase (<3k⭐, sin SaaS). ChatGPT/Claude dan memoria gratis dentro de su walled-garden → la memoria portable ENTRE apps es el hueco.

### Lectura estratégica correcta para un producto SIN nube
Toda la competencia cobra por su nube. Eso significa dos cosas:
1. **Nadie compite en local-first puro** — es el posicionamiento defendible de VantaDB ("tus datos nunca salen de tu máquina" es un claim que Mem0/Zep/Letta NO pueden copiar sin contradecir su modelo).
2. **VantaDB no debe pelear en su terreno (cloud pricing)** sino ganar el terreno donde la competencia no puede entrar: embebido, navegador (WASM), desktop. El dinero de fase 1 viene de productos locales + papel legal; el cloud es fase 2 opcional.

---

## 2. Respuestas corregidas a las preguntas originales

### Q1 — Sistema de cobro (corregido)
- **Fase 0 (ya):** GitHub Sponsors + OpenCollective — $0 fijo.
- **Fase 1 (al publicar Desktop Pro):** storefront **Lemon Squeezy** — 5%+$0.50 por venta, cero coste mensual, maneja VAT/facturación global, license keys nativas para apps desktop offline.
- **B2B:** cotización manual + invoice para OEM/soporte (sin pasarela compleja al inicio).
- **Stripe propio:** solo cuando MRR >~$1k justifique gestionar impuestos directamente.
- Detalle completo en doc `08`.

### Q2 — Precios (corregido)
**Fase actual (sin nube):**
| SKU | Precio | Anclaje |
|---|---|---|
| VantaDB Desktop Free | $0 | embudo |
| **VantaDB Desktop Pro** | **$79 pago único, perpetuo, 1 año updates** | TablePlus $99 Basic verificado (precio agresivo de entrada para ganar tracción) |
| Licencia OEM/comercial | desde ~$500–2.000/año según tamaño/redistribución | Qt/iText/SEE-SQLite [media] |
| Soporte Priority | ~$49–99/mes cuando haya demanda | estándar sector |

**Fase 2 (cuando exista el cloud, financiado por fase 1):** Free $0 · Starter $19 · Pro $249 · Enterprise custom — escalera anclada en Mem0/Zep/Turso, documentada intacta en `validacion/03`.

### Q3 — Licencia para versión gratuita y paga (corregido)
1. **HOY: Apache-2.0 se mantiene** (coste legal $0, fricción cero, estándar del nicho). Con 347 descargas/mes el riesgo real de resale es ≈ 0; el enemigo es la invisibilidad.
2. **CLA ligero desde el primer commit externo** ($0): preserva el derecho futuro a relicenciar/dual-license/OEM.
3. **Cambio a FSL o AGPL+dual solo si:** aparece un reseller, o antes del firmar el primer cliente enterprise que exija garantías. Cambiar licencia con userbase≈0 es barato; tarde es el caso Elastic→OpenSearch. Ventana abierta, no gastarla antes de tiempo ni forzarla.
4. Marca registrada: diferida al primer ingreso significativo (~$350 USPTO).

### Q4 — Qué es gratis y qué es pago (corregido)
**Gratis para siempre (fase 1):** motor completo (hybrid search, WAL, ACID, grafo, GraphRAG local), SDKs Python/TS/WASM completos, CLI, MCP server, embeddings BYO-key, cifrado local, app Desktop básica.
**Pago (fase 1, sin servidores):** features Pro de la app Desktop (grafo visual, multi-BD, backup/export manager, benchmark suite), licencia OEM/comercial, soporte/warranty.
**Pago (fase 2, requiere nube propia):** sync multi-dispositivo, backups gestionados, embeddings gestionados, consola web, SSO/HIPAA, SLA.

### Q5 — Cuántos planes pagos (corregido)
**Fase 1: UN producto de pago simple** (Desktop Pro pago único + venta directa B2B). No hay "planes mensuales": sin servicio recurrente detrás no hay razón honesta para suscripción (lección verificada en doc 02: los flat fees funcionan SOLO con un cloud detrás).
**Fase 2: tres planes pagos** (+Free = 4 total), cuando haya servicio que facturar.

### Q6 — Marketing (se mantiene casi íntegro)
El playbook de `validacion/04` ya estaba diseñado a presupuesto cero. Corrección única: **nada depende de beta-sync**. Secuencia vertebral:
1. S1: quickstart <60s blindado en CI + FUNDING.yml + publicar adapters PyPI/npm (están 404).
2. S1–S2: `vantadb-mcp` al MCP Registry + snippet Claude Desktop/Cursor en README (palanca #1 de distribución agent-native 2026).
3. S2–S4: llms.txt + playground WASM embebible (memoria corriendo EN tu browser, sin signup — requisito Show HN).
4. S3–S4: BENCHMARKS.md público reproducible (patrón uv).
5. S4–S6: r/LocalLLaMA + awesome-lists + build-in-public X.
6. S8–S9: Show HN → Product Hunt.

### Q7 — Branding (sin cambios)
Tagline H1: **"Local-first memory for AI agents."** · Soporte técnico: "The embedded memory engine for AI agents — like SQLite for what your agents remember." · Demostración: "Runs wherever your agent runs — server, laptop, or browser tab." Un solo nombre (`vantadb`) en todas las superficies. Paleta near-black `#0A0A0C` + acento eléctrico, Inter/Geist + JetBrains Mono, og:images desde el logo-reveal.gif existente. Tono dev-to-dev, código primero, límites admitidos.

### Q8 — Valor actual y cómo aumentarlo (sin cambios en el diagnóstico)
Valor actual: MVP 0.5–0.6.x honesto ("SQLite para agentes IA", durabilidad certificable, hybrid+grafo en-proceso único). Brecha: ingeniería >> distribución (adapters 404 en PyPI, CHANGELOG [Unreleased] ~700 líneas sin cortar). Aumentar valor percibido = distribuir lo construido: paquetes publicados → release 0.6.0 cortada → MCP como producto → playground WASM → benchmarks públicos → migrador Chroma promocionado.

### Q9 — Qué falta como producto (sin cambios, prioridad ajustada)
1. Publicar adapters/paquetes (bloqueo #1 del funnel) · 2. Release 0.6.0 + changelog · 3. Desktop Free pulido (es también el escaparate) · 4. Playground in-browser · 5. MCP productizado · 6. Benchmarks públicos · 7. Integraciones LangChain/LlamaIndex/CrewAI visibles · 8. Deuda crítica viva ANTES de cobrar: CRIT-01 atomicidad (`txn.rs:119-191`), panics en `sync_ext.rs`/`llm.rs`.

### Q10 — Archivos indispensables del repo (ajuste mínimo)
Base ya buena (LICENSE Apache, README bilingüe, CONTRIBUTING/SECURITY/COC, CI matriz win/mac/linux, ejemplos en CI). **P0:** `.github/FUNDING.yml` (**sube a P0: es el canal de cobro fase 0**), CITATION.cff, firma de binarios Windows (SmartScreen), smoke de ejemplos en las 3 plataformas. P1: página `/licensing` explicando qué es gratis y qué no, ROADMAP.md público.

### Q11 — Cómo lograr que prueben el producto (sin cambios)
Friction=0 (quickstart copy-paste sin registro, WASM en-browser sin backend ni email — nadie más lo ofrece), MCP server visible arriba del fold, benchmarks honestos, migrador Chroma como caballo de Troya, respuesta a issues <24h. Show HN cumple sus 4 reglas oficiales con el playground vivo.

### Q12 — Qué NO debe fallar nunca (sin cambios, sigue siendo crítico)
1. Namespace vacío devuelve `[]` silencioso → parece roto; mitigar con UX/datos ejemplo. 2. Instalación limpia Win/mac/Linux + binarios Windows firmados. 3. put/get roundtrip + persistencia tras reinicio como smoke canónico (hoy solo chaos). 4. Smoke de relevancia híbrida ("busca X, devuelve X") en CI. 5. Cold-open de DBs grandes medido y publicado. 6. Docs=API (verificado ✓ — mantener con tests de snippets). 7. Cero panics feos. 8. CRIT-01 corregido antes de facturar durabilidad.

---

## 3. Validación de los 6 documentos internos (vigente — detalle en `validacion/05`)

| Documento | Fiabilidad | Veredicto |
|---|---|---|
| experimental-quarantine-2024-06.md | ~85% | Lápida histórica válida; título confuso, 2 refs rotas |
| AnalisisTecnico_BusinessProfessional.pdf | ~70% | ⚠️ Inventa 10 crates inexistentes; cita wal_archiver eliminado |
| Auditoria_Tecnica.md | ~85% | El más preciso (1.773 tests exactos al commit); deuda viva: CRIT-01, CRIT-03, CRIT-04/05 |
| Auditoria_Tecnica.pdf | ~85% | Duplicado exacto del MD |
| Manual_Estrategico_Unificado.md | ~65% | Contradicciones internas; estrategia útil como inspiración, cifras NO |
| vantadb-audit-report.md | ~60% hoy / ~90% en su commit | ~80% de hallazgos YA corregidos; RBAC stub (correcto: doc 3) |

Nuevo no capturado por nadie: 2 `\` faltantes romperían `docker build`.

## 4. Roadmap integrado corregido

**Días 0–30 (fundamento, $0 fijo):** FUNDING.yml + CITATION.cff · adapters PyPI/npm publicados · release 0.6.0 + changelog cortado · fix Dockerfile + CRIT-01/panics · quickstart blindado CI · llms.txt · Sponsors activos.
**Días 31–60 (distribución + primer SKU):** MCP registry + snippet Claude Desktop · playground WASM · BENCHMARKS.md · r/LocalLLaMA + awesome-lists · **Desktop Free pulido y Desktop Pro definido** (grafo visual, multi-BD, backup/export manager) · CLA activado · firma de binarios Windows · Lemon Squeezy configurado.
**Días 61–90 (lanzamiento y primeras ventas):** Show HN (S8) → Product Hunt (S9) · **Desktop Pro $79 a la venta vía Lemon Squeezy** · Discord · primeros contactos B2B OEM/soporte · métricas: descargas, WAU, primeras ventas. Gatillo fase 2 (cloud): ≥500 WAU o ≥200 waitlist sync o ~$500 MRR de Desktop Pro.

## 5. Riesgos y supuestos

1. **Ventana temporal:** Cognee-Rust/Qdrant Edge/LanceDB-CrewAI apuntan al mismo hueco. GTM >90 días erosiona el diferenciador.
2. **Riesgo del modelo sin nube:** el techo de ingresos fase 1 es bajo (licencias perpetuas + sponsors). Es aceptable como arranque; el plan fase 2 existe y queda documentado.
3. **Apache-2.0 permite resale teórico:** aceptado conscientemente pre-PMF; CLA preserva la opción de endurecer licencia temprano (barato ahora, carísimo después).
4. **Datos mercado = 2026-08-25:** re-verificar precios ajenos antes de decisiones finales.
5. **Capacidad founder-led:** si solo se hacen 3 cosas: adapters publicados, MCP registry, Desktop Pro a la venta.

## 6. Índice de investigación

| Archivo | Estado |
|---|---|
| `00-SINTESIS-EJECUTIVA.md` | **Este archivo (v2 vigente)** |
| `08-monetizacion-local-first-sin-capital.md` | **Reinvestigación que corrige la monetización (MoR, Desktop Pro, licencia sin capital)** |
| `01–04` (en `validacion/`) | Datos de mercado válidos; recomendaciones de monetización SUPERSEDEDAS por `08` |
| `05–07` (en `validacion/`) | Vigentes completos (hechos sobre código/repo, no supuestos de negocio) |

---
*Investigación generada por sub-agentes coordinados (vanta-lead orquestador), 2026-08-25. Datos de mercado extraídos de páginas oficiales ese día; hallazgos de código verificados contra el workspace indexado.*
