---
title: "Backlog — Correcciones urgentes y medias (2026-09-10)"
type: plan
status: stable
tags: [vantadb, docs, backlog, correcciones]
last_reviewed: 2026-09-15
aliases: []
related: []
---

# Backlog — Correcciones urgentes y medias (2026-09-10)

> **INGERIDO 2026-09-14:** las 26 filas (FIND-63..88) viven en `docs/dev/Backlog.md` (verificado por conteo). Este archivo es el recibo original — no agregar filas aquí, solo allá.
>
> Fuente: revisión 19 módulos (`docs/dev/reviews/archive/review-full-20260910-modulos.md` + apéndice).
> Alcance: 8 High + 18 Medium. Generado para `/pipeline plan`.

## Urgentes (High)

| ID | Descripción | Archivos | Esfuerzo |
|---|---|---|---|
| `FIND-63` | Vitest desktop 73/86: 13 fails `localStorage` bajo Node — fijar environment jsdom o mock storage en `vitest.config.ts` | `desktop/src/store/undo.test.ts`, `desktop/vitest.config.ts` | 🟢 |
| `FIND-64` | CI no dispara en cambios solo-`vanta-memory/` — agregar `'vanta-memory/**'` a paths de `ci-rust-10.yml` | `.github/workflows/ci-rust-10.yml` | 🟢 |
| `FIND-65` | Test `cache::tests::ttl_expiry_predicate` paniquea (Instant - 10_000s overflow en hosts con uptime <2.7h) — usar `checked_sub` o duraciones pequeñas | `vanta-proxy/src/cache.rs:760-763` | 🟢 |
| `FIND-66` | `Formula/README.md` desincronizado ×3 (mcp listado no instalado, ARM64 Planned ya servido, --head sin stanza) | `Formula/README.md`, `Formula/vantadb.rb` | 🟢 |
| `FIND-67` | `docs/user/QUICKSTART.md` stale (v0.4.x + wheel 0.1.1 vs 0.5.0) — actualizar boundary + paths + revalidar | `docs/user/QUICKSTART.md` | 🟡 |
| `FIND-68` | Sin doc dedicada del proxy (8 features opt-in + endpoints) + `config.toml` ejemplo mínimo — crear `docs/api/PROXY.md` | `docs/api/PROXY.md` (nuevo), `vanta-proxy/config.toml` | 🟡 |
| `FIND-82` | Skills drift 79 documentadas vs 56 servidas + `test-mcp.py` no aserta conteo — buildear desde fuente y asertar por perfil | `skills/vantadb-mcp/scripts/test-mcp.py` | 🟡 |
| `FIND-83` | Skills copias divergentes + contradicción MCP-27/29 + `api-reference.md` omite 8 tools — unificar y re-copiar con hash gate | `skills/`, `.opencode/skills/` | 🟡 |

## Medias

| ID | Descripción | Archivos | Esfuerzo |
|---|---|---|---|
| `FIND-69` | Fallback dspy roto sin framework (TypeError) — tolerar o no llamar super en fallback | `integrations/dspy/vantadb_dspy/vectorstore.py` | 🟢 |
| `FIND-70` | `ingestion_concurrent` se saltea en silencio sin `--features async-ingestion` — pasar flag o documentar skip | `benches/`, `heavy-bench-nightly-51.yml` | 🟢 |
| `FIND-71` | Embeddings: recortar `ALLOW_PATTERNS` (3-4× peso) + `verify`→smoke + sizes README | `embeddings/download.py`, `embeddings/README.md` | 🟡 |
| `FIND-72` | Benchmarks Python: CLI en `batch_vs_sequential_bench.py` + pins en requirements + fix Chroma WinError32 | `benchmarks/` | 🟡 |
| `FIND-73` | Providers `.pyi` omiten `key` en `store()` + `ollama/README.md` stale; endurecer `verify_pyi.py` a firmas | `providers/` | 🟢 |
| `FIND-74` | `examples/README.md` índice + requirements 0.5.0 + enlazar QUICKSTART + decidir TS | `examples/`, `docs/user/QUICKSTART.md` | 🟢 |
| `FIND-75` | `vantadb-wasm/README.md` bundle stale 1.35 vs 1.58MB + input zero-copy pendiente | `vantadb-wasm/` | 🟢 |
| `FIND-76` | Documentar `jwt_secret` en CONFIGURATION.md + corregir link roto `HTTP_API.md:600` | `docs/user/operations/CONFIGURATION.md`, `docs/api/HTTP_API.md` | 🟢 |
| `FIND-77` | Comentarios stale conteo tools MCP (76→79) | `vantadb-mcp/src/handlers/tools.rs` | 🟢 |
| `FIND-78` | `vantadb-node/README.md` link roto + nota engines node>=18 vs ts>=22.19 | `vantadb-node/README.md` | 🟢 |
| `FIND-79` | TS SDK: tests export/import/reindex + endurecer `importRecords` | `vantadb-ts/src/` | 🟡 |
| `FIND-80` | Fuzz: commitear seed corpus + upload crashes + actualizar `docs/dev/workflow/fuzz-40.md` | `fuzz/`, `docs/dev/workflow/fuzz-40.md` | 🟡 |
| `FIND-81` | Server higiene: `vanta_certification.json` legacy + `vantadb_data/` 335MB + mini README | `vantadb-server/` | 🟢 |
| `FIND-84` | Integrations: pineos superiores + fixtures robustas + decidir `dist/` y PyPI | `integrations/` | 🟡 |
| `FIND-85` | Python: matriz CI 3.12/3.14 + limpiar `probe_lock_db/` + unificar `put_batch_raw` | `vantadb-python/`, `release-wheels-60.yml` | 🟡 |
| `FIND-86` | Memory diferidos: MEM-69 wiring + tool 77 + MEM-70 números reales | `vanta-memory/` | 🟡 |
| `FIND-87` | TS SDK: exponer `./native` en exports + nota wiki en TS_SDK.md | `vantadb-ts/` | 🟢 |
| `FIND-88` | Proxy: output-side cost (SSE drain) + translate simétrico si el roadmap lo exige | `vanta-proxy/src/server.rs` | 🟠 |
