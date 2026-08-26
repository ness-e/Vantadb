# Competidores: Memoria para Agentes de IA — Mapeo de Mercado
**Fecha:** 2026-08-25 (datos verificados en vivo el 25-08-2026 UTC) · Autor: vanta-research (Agente A) · Confianza marcada por dato

## Resumen ejecutivo
1. El mercado está dominado por open-core Apache-2.0 con SaaS cloud: Mem0 (64k⭐, $24M fnd) lidera tracción; Cognee/Zep/Supermemory (~30k⭐ cada uno) son el pelotón perseguidor bien capitalizado.
2. Todos los competidores tienen freemium con escalera de 3-4 niveles: Free generoso → entrada $19-25/mes → mid $100-400/mes → Enterprise custom (SOC2/HIPAA/on-prem).
3. La monetización migró a **créditos por ingesta**: se cobra el procesamiento LLM al escribir (Zep créditos, Supermemory SM-tokens, Honcho $2/M, LangSmith LCUs); la recuperación es gratis/ilimitada porque su costo marginal es bajo.
4. Features avanzadas se gated detrás del pago: grafo de entidades (Mem0 Pro $249), consolidación ("Dream"/"Dreaming" en Mem0 y ChatGPT), extracción custom (Zep Flex Plus).
5. MCP y plugins de coding-agents (Claude Code, Codex, Cursor, OpenClaw) son el canal de distribución de 2026 — todos lo tienen.
6. Referencias cerradas: ChatGPT y Claude dan memoria en TODOS los planes (Claude desde mar-2026 incluso Free); la diferenciación es walled-garden vs. capa portable.
7. Vacíos que VantaDB puede atacar: núcleo Rust real (solo Cognee lo anunció, aún Python), WASM/navegador (nadie lo tiene), y memoria local-first portable entre apps.

---

## 1. Mem0 — github.com/mem0ai/mem0
Capa de memoria universal: extracción/actualización LLM de hechos en stores vectoriales + grafo de entidades opcional. YC S24.

| Dimensión | Dato |
|---|---|
| Modelo negocio | Open-core: lib OSS + Platform cloud |
| Licencia | **Apache-2.0** ✅ verificada |
| Precios (mem0.ai/pricing) | Hobby $0: 10k add + 1k retrieval req/mes, 1 proyecto · Starter **$19/mes**: 50k add + 5k ret · Pro **$249/mes**: 500k add + 50k ret, grafo de entidades, "Dream" (consolidación), proyectos ilimitados · Enterprise custom: on-prem, SLA, audit, SSO. Opción usage-based disponible |
| Gratis vs pago | Gratis = volumen bajo + 1 proyecto; grafo/consolidación/on-prem = pago |
| Distribución | pip (`mem0ai`), npm (`mem0ai`), API hosted, self-host OSS, on-prem Enterprise |
| Tracción | **64,039 ⭐**, 7.5k forks · **$24M** (oct-2025: $3.9M seed Kindred + $20M Series A Basis Set; Peak XV, GitHub Fund, YC) [confianza alta] · AWS Agent SDK memory provider |

## 2. Zep / Graphiti — github.com/getzep/graphiti
Grafo temporal ("Context Graph"): entidades, relaciones y observaciones con vigencia temporal. Motor OSS = Graphiti; el viejo servidor OSS `getzep/zep` fue retirado (repo queda solo con ejemplos) → cloud-first.

| Dimensión | Dato |
|---|---|
| Modelo negocio | OSS engine (Apache) + SaaS cloud obligatorio para Zep completo |
| Licencia | **Apache-2.0** (graphiti) ✅ |
| Precios (getzep.com/pricing) | Free: 10k créditos/mes, 2 proyectos · Flex **$125/mes** ($104/mes anual): 50k créditos, 600 RPM, 5 proyectos · Flex Plus **$375/mes**: 200k créditos, 1k RPM · Enterprise custom: BYOC, SOC 2 II, HIPAA. Crédito = ingesta (1 crédito/350 bytes); **retrieval/storage/users gratis** |
| Gratis vs pago | Retrieval gratis siempre; se paga ingestión; observaciones/extracción custom solo Flex Plus+ |
| Distribución | API cloud, Graphiti self-host (Python/Go), MCP server oficial, BYOC Enterprise |
| Tracción | **30,309 ⭐** (graphiti) · **$2.3M** total (YC W24, Engineering Capital) [confianza media-alta] |

## 3. Letta (ex-MemGPT) — github.com/letta-ai/letta
Agentes stateful con memoria self-editing (arquitectura OS-papel de MemGPT). Spin-off de UC Berkeley.

| Dimensión | Dato |
|---|---|
| Modelo negocio | Plataforma OSS + Cloud/ADE |
| Licencia | **Apache-2.0** ✅ |
| Precios (letta.com/pricing) | Free: 3 agentes stateful, BYOK · Pro **$20/mes** personal: 20 agentes, cuota "Letta Auto" · Developer: pay-as-you-go por créditos (tools $0.00015/s) · Teams/Enterprise custom |
| Gratis vs pago | BYOK hace casi todo gratis salvo cuota de agentes; Auto/cuotas = pago |
| Distribución | pip/Docker self-host, Letta Cloud, CLI, ADE web |
| Tracción | **24,439 ⭐** · **$10M** seed (sep-2024, Felicis, val. $70M) [alta] |

## 4. Cognee — github.com/topoteretes/cognee
Memory-graph auto-construido: pipeline ECL (Extract-Cognify-Load) sobre grafo + vector + relacional unificados (GraphRAG). Berlin.

| Dimensión | Dato |
|---|---|
| Modelo negocio | OSS engine + cloud platform (beta) |
| Licencia | **Apache-2.0** ✅ |
| Precios (cognee.ai/pricing) | Free: 1M tokens/mes, 1 workspace, users/API ilimitados · Standard: +**$5/workspace adicional**/mes, integraciones Slack/Notion/Drive · Business: SLA, ingeniero dedicado, BYO-cloud. *(Cifra flat mensual exacta no visible en extracción — confianza media)* · Self-host OSS: gratis para siempre |
| Gratis vs pago | Engine completo gratis; conveniencia cloud + workspaces + soporte = pago |
| Distribución | pip, TS client, **Rust client**, plugins Claude Code/MCP/OpenClaw, cloud |
| Tracción | **30,264 ⭐** · **$9.09M total** ($7.5M seed feb-2026, Pebblebed/42CAP/Vermilion) [alta] · Planean **motor Rust para edge** |

## 5. Supermemory — github.com/supermemoryai/supermemory
Motor memoria + contexto multimodal; cobra "SM tokens" con deduplicación (re-ingesta no se factura). TypeScript-nativo.

| Dimensión | Dato |
|---|---|
| Modelo negocio | Open-core MIT + SaaS usage-based |
| Licencia | **MIT** ✅ |
| Precios (supermemory.ai/pricing) | Free $0 (~$5 uso incl.) · Pro **$19/mes** (~$20 uso) · Max **$100/mes** (~$130 uso) · Scale **$399/mes**: SOC 2, HIPAA, self-host · Uso: $0.005/1k SM tokens; búsqueda $0.005/1k queries |
| Gratis vs pago | Todo el stack usable en Free con techo de uso; self-host solo Scale+ |
| Distribución | API, SDK TS, MCP, plugins Claude Code/OpenClaw/Hermes, connectors (Drive/Notion/Gmail) |
| Tracción | **29,065 ⭐** · **$2.6M** seed (oct-2025, Susa/Browder/SF1) [alta] |

## 6. LangMem / LangGraph Memory (LangChain) — github.com/langchain-ai/langmem
Store semántico + memoria procedural como parte del ecosistema LangSmith/LangGraph Platform (no producto independiente de precios propios).

| Dimensión | Dato |
|---|---|
| Modelo negocio | Librerías OSS + plataforma cloud de pago |
| Licencia | MIT (langmem/langchain; stars no extraídas hoy — laguna) |
| Precios (langchain.com/pricing) | Developer **$0**: 1 seat, 5k traces/mes · Plus **$39/seat/mes**: 10k traces + Deployment · Enterprise custom: self-host/hybrid · Usage: LCU **$1.50**, LSU **$1.00** |
| Gratis vs pago | Dev free amplio; colaboración/deploy/self-host = pago |
| Distribución | pip, LangSmith cloud, hybrid/self-host Enterprise |
| Tracción | Ecosistema enorme (langchain ~100k⭐ histórico) · funding de LangChain **no verificado en esta sesión** |

## 7. Honcho (Plastic Labs) — github.com/plastic-labs/honcho
Memoria razonante "peer-centric": modelo continuo del usuario vía modelos propios (Neuromancer); SOTA en LongMem/LoCoMo. Dreaming asíncrono.

| Dimensión | Dato |
|---|---|
| Modelo negocio | Core OSS + cloud usage-based (sin tiers mensuales) |
| Licencia | **AGPL-3.0** ✅ (único copyleft fuerte del sector) |
| Precios (honcho.dev) | Ingesta **$2.00/M mensajes** (incluye reasoning) · context() **ilimitado** · reasoning .chat(): $0.001–$0.50/query según profundidad · Startups (<$5M raised): $1k créditos · Enterprise custom |
| Gratis vs pago | Sin free tier explícito de plataforma; retrieval gratis; se paga escritura+reasoning |
| Distribución | pip/npm, app.honcho.dev, self-host (AGPL), plugins Claude Code/Codex/OpenClaw/Hermes, MCP |
| Tracción | **6,840 ⭐** · funding Plastic Labs **no verificado** · SOC 2 Type I |

## 8. memU (NevaMind AI) — github.com/NevaMind-AI/memU
Memoria como wiki/skills Markdown destiladas por el propio agente; sidecar para coding-agents (Codex, Claude Code, Cursor, OpenClaw…).

| Dimensión | Dato |
|---|---|
| Modelo negocio | OSS + cloud gratuito (cross-device) — monetización aún difusa |
| Licencia | README declara **Apache-2.0**; GitHub detecta "Other/NOASSERTION" [media] |
| Precios | Cloud **free/unlimited** (memu.so); self-host local gratis; tiers de pago no publicados |
| Distribución | pip `memu-cli`, npx, binarios por host, SQLite/Postgres local |
| Tracción | **14,347 ⭐** en ~13 meses (crecimiento viral) · funding no público |

## 9. Memobase (MemoDB) — github.com/memodb-io/memobase
Perfiles de usuario estructurados para chatbots (companions). ⚠️ **Sitio memobase.io caído** (404 Framer) y repo sin commits desde ene-2026 → probablemente discontinuado/pivotado [confianza media].

| Dimensión | Dato |
|---|---|
| Licencia / Tracción | Apache-2.0 ✅ · 2,859 ⭐ · pricing actual no verificable |

## 10. Memary — github.com/kingjulio8238/Memary
Memory layer KG para agentes autónomos. **Abandonado** (último push oct-2024). MIT ✅ · 2,641 ⭐ · OSS puro, sin SaaS ni precios.

## 11-12. Referencia cerrada: ChatGPT / Claude Memory
- **ChatGPT**: memoria en todos los planes (limitada en Free), "Dreaming" (curación automática) desde jun-2026. Free $0 (con ads US) · Go ~$8 · Plus **$20-23** · Pro **$100/$200** [confianza alta-media, fuentes terceras].
- **Claude**: memoria en TODOS los planes desde mar-2026 (antes Team/Enterprise/Max). Pro **$20** · Max **$100/$200**. Modelo resumido, no perfil automático.
- Lección: la memoria ya es commodity en asistentes; el valor está en capas portables entre apps (ninguno recuerda across-vendors).

## Tabla comparativa consolidada

| Producto | Licencia | ⭐ | Free tier | Entrada | Mid | Enterprise | Cobro por | Self-host |
|---|---|---|---|---|---|---|---|---|
| Mem0 | Apache-2.0 | 64.0k | 10k add/mes | $19 | $249 (grafo+Dream) | custom (on-prem) | requests | ✅ OSS |
| Zep | Apache-2.0 | 30.3k | 10k créditos | $125 | $375 | custom (BYOC) | créditos ingesta | Graphiti ✅ |
| Cognee | Apache-2.0 | 30.3k | 1M tokens | $5/workspace | — | SLA/BYO-cloud | tokens+workspaces | ✅ OSS |
| Supermemory | MIT | 29.1k | ~$5 uso | $19 | $100 / $399 | Scale (SOC2) | SM tokens | Scale+ |
| Letta | Apache-2.0 | 24.4k | 3 agentes | $20 | créditos | custom | créditos compute | ✅ OSS |
| memU | Apache-2.0* | 14.3k | ilimitado cloud | — | — | — | aún no | ✅ |
| Honcho | AGPL-3.0 | 6.8k | retrieval gratis | usage-based | usage-based | custom | ingesta $2/M | ✅ AGPL |
| Memobase | Apache-2.0 | 2.9k | — sitio caído — | — | — | — | — | ⚠️ |
| Memary | MIT | 2.6k | OSS puro | — | — | — | — | ✅ (muerto) |
| ChatGPT | propietario | — | memoria limitada | $8-23 | $100/$200 | custom | suscripción | ❌ |
| Claude | propietario | — | memoria full free | $20 | $100/$200 | custom | suscripción | ❌ |

## Patrones del mercado
1. **Escalera típica: 4 planes.** Free → ~$19-25 → ~$100-400 → Enterprise custom. El punto de entrada clúster-consistente es **$19-25/mes** (Mem0 $19, Supermemory $19, Letta $20 ≈ ChatGPT Plus $20); mid-tier entre $100-400.
2. **Freemium domina 100%:** todos tienen free tier. Lo gratis: retrieval ilimitado y volúmenes chicos de prueba. Se cobra: ingesta (COGS = llamadas LLM al escribir), multi-proyecto, compliance.
3. **Monetizar lo self-hostable:** (a) conveniencia cloud gestionada, (b) medición por créditos/tokens de *escritura*, (c) features de compliance gated (SSO/SLA/audit/SOC2/on-prem) — nunca features de código core, (d) algoritmo premium gated (grafo, consolidación tipo "Dream").
4. **Licencias:** Apache-2.0 es el estándar del sector (6/9); MIT para nativos-TS; un solo AGPL (Honcho). Nadie usa BSL/SSPL entre los líderes.
5. **Correlación estrellas↔funding:** Mem0 (64k⭐/$24M) > {Cognee, Zep, Supermemory} (~30k⭐, $2.3-9M) > Letta (24k⭐/$10M). Los muertos (Memary, Memobase) quedaron <3k⭐ sin SaaS sólido.
6. **Canal 2026 = coding agents:** server MCP + plugins Claude Code/Codex/Cursor es mesa de entrada obligatoria; memU creció 14k⭐ solo con eso.

## Implicaciones para VantaDB (síntesis rápida)
- **Rust-core es diferenciador real**: nadie lo tiene en producción (Cognee lo anunció para edge — ventana corta).
- **WASM/navegador**: territorio virgen; nadie compite ahí.
- Pricing sugerido por patrón: Free (techo de ingesta) → **~$19-24** pro individual → **~$99-250** equipos con sync cloud → Enterprise (compliance). Cobrar ingesta/sync, dar retrieval gratis.

## Fuentes consultadas (verificadas 25 ago 2026)
- api.github.com/repos/{mem0ai/mem0, getzep/graphiti, getzep/zep, letta-ai/letta, topoteretes/cognee, supermemoryai/supermemory, plastic-labs/honcho, memodb-io/memobase, NevaMind-AI/memU, kingjulio8238/Memary} (stars+licencia exactas)
- mem0.ai/pricing · getzep.com/pricing · letta.com/pricing · supermemory.ai/pricing · cognee.ai/pricing · honcho.dev · langchain.com/pricing
- techcrunch.com (Mem0 $24M, 28-oct-2025) · prnewswire.com (Letta $10M, 24-sep-2024) · cognee.ai/blog ($7.5M seed, 19-feb-2026) · supermemory.ai/blog ($2.6-3M) · indexed.vc/companies/zep-ai ($2.3M)
- morphllm.com/chatgpt-vs-claude, memorylake.ai (estado memoria ChatGPT/Claude 2026) [terceros]

## Lagunas de investigación
- Precio flat mensual exacto del tier medio de Cognee (número no renderizado en extracción) [confianza media]
- Pricing actual de Memobase no verificable (sitio caído); estado real de la empresa desconocido
- Funding de Plastic Labs (Honcho), memU/NevaMind y LangChain no verificados en esta sesión
- Cuotas internas exactas de Letta Auto (opacas por diseño)
- Datos ChatGPT/Claude provienen de comparativas terceras, no páginas oficiales de precios
