# Plan: Post-investigación integral — ejecución P52–P56 (2026-09-24)

> **Inicio:** 2026-09-24
> **Estado:** 📘 READY-TO-EXECUTE — plan creado; la ejecución de P52–P56 arranca al cerrar la estabilización EST (plan `2026-09-24-estabilizacion-pendiente.md`) y la Estandarización 11 APIs (Phase 51), que tienen precedencia.
> **Fuente:** investigación integral 2026-09-24 (2 informes module-by-module + arquitectura/producto/competencia + investigación web de 17 sistemas de memoria y 10 BDs multi-modelo) + investigación de definición de producto (`docs/dev/research/product-definition-gap-2026-09-24.md`) + estado verificado del repo (`develop` @ 2026-09-24, 0.7.0 en workspace).
> **Backlog:** `docs/dev/Backlog.md` P52 (VER-01..09), P53 (SCH-01..08), P54 (ICP-01..03), P55 (DEF-01..08), P56 (WIRE-01..10) + `Backlog-negocio.md` BIZ-10..13 + `backlog-notion.md` N-12..N-17.
> **Reglas:** Regla 11 (nada afirmado sin fuente reproducible) · P2-01 review antes de cerrar cada wave · `/progreso` al cerrar cada fila · breaking changes ilimitados pre-lanzamiento (`feat!:`) con todo funcional al final del release · MCP local (`vanta-mcp-local.ps1`/`opencode.jsonc`) se refresca ante cualquier cambio de tools.

---

## 1. Decisiones del owner registradas (2026-09-24)

| # | Decisión | Efecto |
|---|----------|--------|
| D1 | **3 tracks ICP a profundidad** (AI-IDEs vía MCP / devs local-LLM y privacidad / frameworks), sin foco único; cada track con one-pager + superficie mínima + demos CI + métrica | BIZ-08 resuelta; desbloquea BIZ-05; materializa en P54 (ICP-01..03) |
| D2 | **Migración única de esquema en 0.7.0**: MGR-10 (bitemporalidad) + MGR-12 (scores asserted/derived) + MGR-13 (cuarentena) en un solo breaking change | Pre-requisito: cerrar research-docs MGR-10/12/13; materializa en P53 (SCH-01..08) |
| D3 | **Harness completo + head-to-head**: canonical_p99 + LoCoMo + LongMemEval-S + BEAM-subset (dataset commiteado) + write-quality/abstención/tokens/p99-CI + contra Mem0/Zep/Letta con protocolo publicado | Materializa en P52 (VER-08/VER-09); consume MGR-19/EXE-02 |
| D4 | **Cobro con stack real**: profundizar carriles PayPal + Binance + Payoneer (próxima cuenta) | Materializa en `Backlog-negocio.md` BIZ-10..12; BIZ-04 (ToS) bloquea cualquier cobro |

---

## 2. Contexto de ejecución (estado al 2026-09-24)

| Frente | Estado | Documento |
|---|---|---|
| Estabilización pre-0.7.0 (EST-01..12) | FASE 1 commiteada (`bcd62115`); falta EST-03 (propuesta owner), EST-05 verificación verde, FASE 2 (CodeQL EST-06/07/08), FASE 3 (EST-10/11), EST-12 (merge PR #222 + tag + publish — owner) | `docs/dev/plans/2026-09-24-estabilizacion-pendiente.md` |
| Estandarización 11 APIs (API-01..09) | Phase 51 del Backlog; waves W0–W8; precedencia sobre superficies nuevas | `docs/dev/plans/2026-09-24-api-estandarizacion.md` + `docs/dev/tasks/API-STD-15.md` |
| Harness `.opencode/` (FIND-148..151) | ✅ cerrado 2026-09-24 (residual FIND-152) | `docs/dev/plans/2026-09-24-harness-gaps.md` |
| P49 (MGR-01..25 research) | Pendiente (puerta a v0.7/v1.0) | `docs/dev/Backlog.md` §Phase 49 |
| P50 (EXE-01..10) | Pendiente (EXE-02 se ejecuta vía VER-09; EXE-03 = gate de anuncio) | `docs/dev/Backlog.md` §Phase 50 |
| Backlog técnico | 79 abiertas de 109 filas + 38 nuevas de este plan = 117 de 147 | `docs/dev/Backlog.md` |
| Backlog negocio | 20 activas + BIZ-10..13 | `docs/dev/Backlog-negocio.md` |
| Notion | Hub "VantaDB Docs" + "Propuesta" + subpáginas; sync planificado en `docs/dev/strategy/NOTION-SYNC-2026-09-24.md` (N-17) | Notion MCP (verificado accesible 2026-09-24) |

**Renumeración documentada:** el análisis post-investigación propuso "Fase P51 — Verificabilidad" y "Fase P52 — Tracks ICP"; por colisión con **Phase 51 (Estandarización 11 APIs)** creada el mismo día, quedan como **P52 (VER)** y **P54 (ICP)**. La fila "P51 IMPL-MGR" del Exec Summary del Backlog se renumeró a **P53** (es la migración única SCH).

---

## 3. Fases y filas (mapa completo)

### P52 — 🔐 Verificabilidad (la categoría "memoria verificable y gobernable")

> Hueco competitivo exclusivo verificado contra 17 sistemas: ninguno ofrece verificación criptográfica, borrado certificado, redacción persistida ni governance de inyección. VantaDB tiene las piezas (WAL, shred, proxy con redacción, MCP, desktop) — falta cablearlas.

| ID | Qué | Dep | Esf. | Prio |
|---|---|---|---|---|
| VER-01 | Tamper-evident: hash-chain sobre operaciones de memoria en WAL + `vanta-cli verify` | — | 🔴 3-5d | 🔴 |
| VER-02 | Borrado certificado: cablear delete-path shred→GC→WAL + attestation de purga | SCH-02 (versión de registro) | 🟡 2-3d | 🔴 |
| VER-03 | Redacción-on-write persistida + namespaces cifrados (envelope por namespace) | WIRE-01 | 🟡 2-3d | 🔴 |
| VER-04 | Governance de inyección: presupuesto de contexto/request + ACLs por tool/namespace + audit log de inyección | WIRE-01 | 🟡 2-3d | 🟠 |
| VER-05 | Importadores Mem0/Zep/Letta→VantaDB + formato de intercambio documentado | — | 🟢 1-2d | 🟠 |
| VER-06 | Export file-native Markdown + `rebuild_index` (git-friendly) | — | 🟢 1-2d | 🟡 |
| VER-07 | Dreams: dry-run + diff report + `promote_dream_run` real (consolidación ADD/UPDATE/DELETE/NOOP) | — | 🟡 2-3d | 🔴 |
| VER-08 | Harness propio: canonical_p99 + LoCoMo/LongMemEval-S/BEAM-subset + write-quality/abstención/tokens/p99-CI | — | 🟡 3-5d | 🔴 |
| VER-09 | Head-to-head Mem0/Zep/Letta con protocolo publicado (absorbe EXE-02) | VER-08 | 🔴 1-2sem | 🟠 |

### P53 — 🧬 Esquema v0.7.0: migración única (IMPL-MGR, consume P49)

> Decisión D2. Un solo breaking change de schema: bitemporalidad (MGR-10) + confianza (MGR-12) + cuarentena (MGR-13). Pre-requisito: research-docs MGR-10/12/13 cerrados (Cierre MGR = research-doc + preguntas owner + plan de implementación).

| ID | Qué | Dep | Esf. | Prio |
|---|---|---|---|---|
| SCH-01 | Plan único + ADR de migración (diseño conjunto dims 5-6; alcance 0.7.0 vs v1.0) | MGR-10/12/13 research-doc | 🟡 2-3d | 🔴 |
| SCH-02 | Schema v2: `valid_at_ms`/`invalid_at_ms` + `asserted/derived` + `confidence` + `last_validated` + estado `quarantined` + migración + backfill | SCH-01 | 🔴 3-5d | 🔴 |
| SCH-03 | Queries `AS OF`/point-in-time + filtros `valid_at` + `exclude_superseded` extendido | SCH-02 | 🟡 2-3d | 🔴 |
| SCH-04 | Scores asserted/derived consumibles (ranking/UI) — slice 0.7.0; derivación completa queda v1.0 | SCH-02 | 🟡 2-3d | 🟠 |
| SCH-05 | Cuarentena + abstención + trust-aware retrieval (threat model write-time: AgentPoison/MINJA) | SCH-02 | 🟡 2-3d | 🟠 |
| SCH-06 | Tests: migración determinista, time-travel, roundtrip export/import, chaos | SCH-02..05 | 🟡 2-3d | 🔴 |
| SCH-07 | Superficies: bindings/server/MCP/IQL + docs/api mismo-PR | SCH-02..06 | 🟡 2-3d | 🟠 |
| SCH-08 | Corte 0.7.0: migration guide + CHANGELOG + release notes | SCH-07 | 🟢 1d | 🔴 |

### P54 — 🎯 Tracks ICP (decisión D1)

| ID | Track | Superficie mínima | Métrica | Dep | Esf. | Prio |
|---|---|---|---|---|---|---|
| ICP-01 | AI-IDEs vía MCP (Cursor/Claude Code/OpenCode) | One-pager + repo-map/watcher (MGR-22) + viewer + hooks robustos + demo CI | Sesiones con put+search la misma semana (7d) | MGR-22, FIND-106/120, WIRE-01 | 🟡 1-2sem | 🔴 |
| ICP-02 | Devs local-LLM / privacidad | One-pager + redacción persistida + cifrado + forget certificado + auditoría PII + demo CI | 0 PII en store en auditoría | VER-02/03 | 🟡 1-2sem | 🔴 |
| ICP-03 | Frameworks (LangChain/LlamaIndex/…) | One-pager + publicar adapters PyPI (MKT-18f) + importadores (VER-05) + demo CI | Installs/semana de adapters | MKT-18f, VER-05 | 🟡 1sem | 🟠 |

Cada track incluye: one-pager comercial (consume BIZ-05), entrada en `COMPARISON.md` (capa memory-as-a-service), 3 demos que alimentan EXE-01 y su métrica en la North Star.

### P55 — 🧭 Frontera de producto (de `product-definition-gap-2026-09-24.md`)

| ID | Qué | Esf. | Prio |
|---|---|---|---|
| DEF-01 | Decisión única de producto + alineación SPEC/README/VISION (3 tracks sobre un núcleo; jerarquía explícita) | 🟢 1d | 🔴 |
| DEF-02 | Regeneración de `EXPERIMENTAL_FEATURES.md` a 0.7.0 + categorías labs (proxy/desktop/web/memory); fixes inmediatos ya aplicados 2026-09-24 | 🟡 2-3d | 🟠 |
| DEF-03 | Frontera verificable en CI (`validate-frontier` contra features Cargo + rutas reales) | 🟡 2d | 🟠 |
| DEF-04 | Naming freeze 0.7.0→1.0 (ADR; 9 artefactos; política de alias/deprecación; coordina con API-01..09) | 🟢 1d | 🔴 |
| DEF-05 | North-star + success criteria de producto en SPEC/VISION (agentes con recall exitoso en 7d + guardrails) | 🟢 1d | 🔴 |
| DEF-06 | README↔BENCHMARKS reconciliados (claim RocksDB "fallback"; §2 regeneración; citar §1/§8 mientras) | 🟡 2d | 🔴 |
| DEF-07 | Presupuesto de alcance: core-promise vs labs (proxy/desktop/web/memory) + guía de inversión | 🟢 1d | 🟠 |
| DEF-08 | Install SLO + telemetría opt-in de éxito + fallback visible (zero-config) | 🟢 1d | 🟠 |

### P56 — 🔌 Cableado post-investigación (hacer real lo prometido)

| ID | Qué | Dep | Esf. | Prio |
|---|---|---|---|---|
| WIRE-01 | Loop proxy completo: verificar capture L0 (D47/MEM-50) → L1 → search/injection; cablear output-cost (`record_response_usage`; FIND-88) + presupuesto de tokens de inyección | — | 🟡 1-2d | 🔴 |
| WIRE-02 | MCP: enforce de perfil en `tools/call` + default `agent` (~45) + fusión 87→~65 + fix doc `MCP.md:254` | API-04 | 🟡 2-3d | 🟠 |
| WIRE-03 | `query_sparse` + text-only en 3 bindings + filtros avanzados (`$and`/`$or`, range/datetime) | API-02 | 🟡 2-3d | 🔴 |
| WIRE-04 | TTL superficie completa: server HTTP + default por colección + sweeper al índice | MGR-09 | 🟢 1-2d | 🟠 |
| WIRE-05 | Entity linking determinista + boost multi-señal en RRF (Fellegi-Sunter + embeddings; LLM-juez solo candidatos) | MGR-05 | 🟡 3-4d | 🔴 |
| WIRE-06 | Batching productizado (9.1× prototipado; `insert_lock` → segmentos appendables) | FUT-12-spec | 🔴 1sem | 🔴 |
| WIRE-07 | Refactors: crate `ffi-core` (OpGate×3), trait-split storage↔index, desacoplar `server→cli` | — | 🔴 1-2sem | 🟠 |
| WIRE-08 | Range/group_by + cursor con resume en search + RRF en CBO + rewriting + MMR | API-06 | 🟡 3-5d | 🟠 |
| WIRE-09 | Seguridad P0: sandbox de paths export/import/snapshots + refuse-to-start proxy + guards | — | 🟡 2-3d | 🔴 |
| WIRE-10 | Distribución P0: `install.sh` macOS portable, Colab a API `Client`, hooks sin `pwsh` | — | 🟡 2-3d | 🔴 |

---

## 4. Mapeo del análisis → filas (trazabilidad completa)

### Acciones A1–A7

| Acción del análisis | Filas | Nota |
|---|---|---|
| A1 Estabilización pre-0.7.0 | EST-01..12 (plan vigente) | En curso |
| A2 P0 seguridad+GTO | WIRE-09, WIRE-10 (+ API-05 `/snapshot`) | Re-verificar HEAD al ejecutar |
| A3 3 ICPs | ICP-01..03 (P54) | Decisión D1 |
| A4 Migración única | SCH-01..08 (P53) | Decisión D2 |
| A5 Harness + head-to-head | VER-08, VER-09 (P52) | Decisión D3; absorbe EXE-02 |
| A6 Cobro | BIZ-10..12 (+ BIZ-04) | Decisión D4; negocio |
| A7 Paridad dreams | VER-07 (P52) | Mem0 Dream / ChatGPT Dreaming / Claude Auto Dream = paridad mainstream |

### Modificaciones M1–M14

| M | Modificación | Fila |
|---|---|---|
| M1 | Campos schema + backfill | SCH-02 |
| M2 | Queries AS OF | SCH-03 |
| M3 | Loop proxy + cost + presupuesto | WIRE-01 |
| M4 | Perfil MCP enforce + default agent | WIRE-02 |
| M5 | query_sparse + text-only + filtros | WIRE-03 |
| M6 | Entity linking determinista | WIRE-05 |
| M7 | Consolidación explícita + dry-run dreams | VER-07 |
| M8 | TTL server + sweeper | WIRE-04 |
| M9 | Redacción persistida + cifrado + forget | VER-03, VER-02 |
| M10 | Export Markdown + importadores | VER-06, VER-05 |
| M11 | WAL como CDC (futuro) | `backlog-futuro.md` FUT-16 + PRO-02/03 (negocio) |
| M12 | Batching productizado | WIRE-06 |
| M13 | ffi-core + trait-split + server→cli | WIRE-07 |
| M14 | Range/group/cursor + RRF-CBO/MMR | WIRE-08 |

---

## 5. Orden de ejecución (waves con gates)

> Cada wave cierra con: tests verdes + P2-01 review + `/progreso` (fila a `docs/dev/avance/`) + doc sync mismo-PR (Regla 3).

| Wave | Contenido | Gate de salida |
|---|---|---|
| **W0** (en curso) | EST-01..12 → PR #222 verde → merge + tag v0.7.0 | CI verde + publish (owner) |
| **W1** | Phase 51 API-01..09 (W0–W8 internos; precedencia de superficies) | `verify.ps1` + parity + MCP smoke |
| **W2** (paralelo disjunto) | P0: WIRE-09 + WIRE-10; DEF-01/04/05 (decisiones rápidas); DEF-02 (fixes ya aplicados → cierre) | 0 críticos seguridad en export/import/snapshots; claims honestos |
| **W3** | P56 técnico: WIRE-01 → WIRE-02/03/04/05/06/07/08 (orden por dep) | Loop proxy verificable end-to-end; paridad bindings |
| **W4** | P53: SCH-01 (spec+ADR) → SCH-02..08 (migración única) — solo con research-docs MGR-10/12/13 cerrados | Migración + time-travel + surfaces verdes |
| **W5** | P52 verificabilidad: VER-01, VER-05, VER-06, VER-07 + VER-02/03/04 (post-SCH/WIRE-01) | Features de categoría "verificable" con tests |
| **W6** | P54 tracks: ICP-01/02/03 (demos + one-pagers + métricas) + VER-08 | Harness publicado (dataset commiteado) |
| **W7** | VER-09 head-to-head + EXE-03 (gate Fase A, owner) → anuncio | Reporte win/loss + GO firmado |

**Notas de secuencia:**
- W2 y W3 tocan `vanta-proxy` (WIRE-09/10 y WIRE-01) pero en archivos/funciones disjuntos; serializar si colisiona el mismo día.
- SCH (W4) antes que VER-02/03/04 para no tocar el esquema dos veces.
- VER-08 puede arrancar en paralelo a W4 (solo usa superficies estables).

---

## 6. Verificación por wave (contratos mecánicos)

- **W2:** `grep` de validación de paths en `src/server/handlers.rs` (canonicalize+starts_with) + test de escape; `install.sh` sin `sha256sum` (usa `command -v shasum` fallback); Colab ejecuta contra `Client`; hooks funcionan sin `pwsh`.
- **W3:** sesión proxy real: capture→search recupera el turno capturado; `/snapshot` con 401 sin credencial (API-05); `record_response_usage` con caller (grep ≠ 0); `tools/list` default ≤45; `query_sparse` en los 3 SDKs con test; TTL via HTTP; batch insert benchmark ≥5× baseline.
- **W4:** `cargo test` + migración sobre DB v1 real + query `AS OF` devuelve estado histórico; roundtrip export/import con campos nuevos.
- **W5:** `vanta-cli verify` sobre WAL con 1 registro alterado → falla; certificado de purga emitido; dreams dry-run produce diff sin mutar; importador Mem0→VantaDB con fixture.
- **W6:** reporte de harness con dataset commiteado + p99 en CI; 3 demos ICP verdes.
- **W7:** reporte head-to-head con protocolo (modelo/juez/stack/tokens) + EXE-03 firmado.

---

## 7. Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| Scope: 38 filas nuevas + P49/P50 + EST + API es mucho para bus factor 1 | Waves cerradas y priorizadas; W2/W3 son las de mayor ROI (P0 + loop proxy); lo demás puede deslizarse sin bloquear el anuncio |
| Migración única rompe compat | Breaking ilimitado autorizado pre-lanzamiento (nadie usa los paquetes); migration guide + tests de roundtrip (SCH-06/08) |
| Disputa de números en head-to-head | Protocolo publicado + pares accuracy+tokens + dataset commiteado (lección Mem0/Zep; auditoría LoCoMo ~6.4%/63%) |
| Claims prematuros (REAL vs PROPUESTA) | Regla de marcas de `ROADMAP-v0.7.md` aplicada a docs y Notion |
| Colisión de nombres/IDs | Renumeración documentada (§2); DEF-04 congela naming |

---

## 8. Sincronía documental (checklist del plan)

| Documento | Estado |
|---|---|
| `docs/dev/Backlog.md` (P52–P56 + Exec Summary + nota) | ✅ hecho 2026-09-24 |
| `docs/dev/Backlog.md` P57 (residual EST/C catalogado para el pipeline) | ✅ hecho 2026-09-24 |
| `Backlog-negocio.md` (BIZ-08 resuelta, BIZ-05 desbloqueada, BIZ-10..13) | ✅ hecho 2026-09-24 |
| `backlog-notion.md` (N-12..N-17 + links N-07..N-10) | ✅ hecho 2026-09-24 |
| `backlog-futuro.md` (FUT-09/10/11 trackeados, FUT-16 nuevo) | ✅ hecho 2026-09-24 |
| `ROADMAP-v0.7.md` (migración única, VER, ICP, marca) | ✅ hecho 2026-09-24 |
| `VISION.md` (3 tracks, capa memory-as-a-service, métricas corregidas, north-star) | ✅ hecho 2026-09-24 |
| `SPEC.md` (adenda decisiones) | ✅ hecho 2026-09-24 |
| `GO_TO_MARKET.md` (nota de sync) | ✅ hecho 2026-09-24 |
| `BENCHMARKS.md` (sección planificado) + `COMPARISON.md` (capa pendiente) | ✅ hecho 2026-09-24 |
| `EXPERIMENTAL_FEATURES.md` (claims falsos corregidos + labs) | ✅ hecho 2026-09-24 |
| Notion (10 páginas) | 📋 borradores en `docs/dev/strategy/NOTION-SYNC-2026-09-24.md` → ejecutar con N-17 |
| `docs/api/` (TTL server, query_sparse, perfil agent, AS OF, forget/attestation) | ⏳ al estabilizar cada fila (mismo-PR) |

---

## 9. Referencias

- Informes (movidos a `docs/dev/strategy/` el 2026-09-24): `docs/dev/strategy/VantaDB-Informe-Analisis-Completo.md`, `docs/dev/strategy/VantaDB-Analisis-Arquitectura-Producto-Competencia.md`.
- Definición: `docs/dev/research/product-definition-gap-2026-09-24.md`; `docs/dev/strategy/NOTION-SYNC-2026-09-24.md`.
- Mercado (resumen): Mem0 (mem0.ai/research; Dream) · Zep/Graphiti (arXiv 2501.13956) · Letta (letta.com/blog/benchmarking-ai-agent-memory) · LangMem · Cognee (COGX) · MemOS (arXiv 2505.22101) · MIRIX (arXiv 2507.07957) · ReMe · A-MEM (arXiv 2502.12110) · MemMachine (arXiv 2604.04853) · Qdrant Edge (qdrant.tech/edge).
- Benchmarks: LoCoMo (arXiv 2402.17753; auditoría Penfield Labs) · LongMemEval (arXiv 2410.10813) · BEAM (ICLR 2026) · DolphinBench.
- Seguridad: AgentPoison (NeurIPS 2024) · MINJA (arXiv 2503.03704) · unlearning surveys (arXiv 2405.07406).
- Planes vigentes: `2026-09-24-estabilizacion-pendiente.md`, `2026-09-24-api-estandarizacion.md`, `2026-09-24-harness-gaps.md`.
