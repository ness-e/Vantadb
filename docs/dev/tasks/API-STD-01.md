# Task API-STD-01 — Inventario funcional 11 superficies + mapa conjunto vs individual

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (ejecución inline, sin subagentes — free tier no permite `subagent`)
> **Fecha:** 2026-09-24

## 1. Objetivo + contrato

Fijar las 11 superficies API y el mapa conjunto vs individual antes de investigar.

**Contrato (verificado mecánicamente 2026-09-24):**

| Condición plan | Real | Veredicto |
|---|---|---|
| `ls docs/api/*.md = 19` | **18 `.md`** + `openapi.yaml` = **19 ficheros** | 🟡 CORREGIDO — el plan contaba el `.yaml`; contrato real: 18 md + 1 yaml |
| `cargo metadata` ≥4 bindings | 8 dirs existen: python, ts, node, wasm, server, mcp, vanta-memory, vanta-proxy (todos `True`) + workspace 7 members `Cargo.toml:744-752` | ✅ |
| Tabla 11 filas | Abajo | ✅ |

Evidencia: `Get-ChildItem docs/api/*.md → 18`; `Get-ChildItem docs/api/* → 19`; `Test-Path` ×8 → `True`; `src/lib.rs:19` (`Embedded`), `:125` (`sdk`), `:130` (`server`), `:118` (`parser`), `:73` (`cli`).

## 2. Tabla 11 superficies

| # | Superficie | Entry-point código | Doc canónica | Funcionamiento (1 línea) | Conjunto vs individual |
|---|---|---|---|---|---|
| 1 | Rust core SDK | `src/lib.rs:19` `Embedded`, `src/sdk/` | `docs/api/EMBEDDED_SDK.md` | Dueño del estado: open/CRUD/search/graph/IQL/índices/export in-process | **Conjunto: single-owner** — todo vive aquí |
| 2 | Python SDK | `vantadb-python/src/lib.rs` (PyO3, 44 fns + `connect`) | `docs/api/PYTHON_SDK.md` | Vista PyO3 (`*.abi3.so`): nodos por `id:u128`, memoria híbrida, solo-Python batch/hw/wiki | Individual: vista con semántica propia (`get/delete` = nodo) |
| 3 | TS SDK | `vantadb-ts/src/vantadb.ts` (43 métodos) + `native.ts` | `docs/api/TS_SDK.md` | Vista WASM (`serde_wasm_bindgen`) + backend nativo alternativo; `score→distance` renombrado | Individual: doble backend wasm/native que pueden divergir |
| 4 | Node nativo | `vantadb-node/src/lib.rs` (NAPI-RS), `index.d.ts:243-447` | `docs/api/NODE_SDK.md` | Vista NAPI mínima (sin export/import/bulk/audit) + extras `versions/vacuum` | Individual: mínimo server-side, más rápido que WASM |
| 5 | WASM | `vantadb-wasm/src/lib.rs` (47 fns) + `pkg/` + OPFS | `docs/api/WASM_API.md` (+PERSISTENCE, +STANDALONE) | Vista browser: memoria/graph/system + persistencia OPFS/IDB | Individual: fija el wire JSON (`u128`→string decimal) |
| 6 | HTTP + OpenAPI | `src/server/router.rs:149-351` → `handlers.rs`, `docs/api/openapi.yaml` | `docs/api/HTTP_API.md` | Servidor Axum `/api/v2/*` (TCP localhost) + YAML que deriva de la impl | Conjunto: expone el core por red; **YAML no es fuente** (deriva) |
| 7 | MCP | `vantadb-mcp/src/handlers/tools.rs` (stdio, vía `vanta-cli server --mcp` → spawn `vantadb-server --mcp`) | `docs/api/MCP.md` | Cara LLM: tools memory/graph/system con alias duplicados y validación parcial | Conjunto: consume server vía hijo; Individual: nombres propios |
| 8 | IQL | `src/parser/grammar.rs` + `lexer.rs` (nom) | `docs/api/IQL.md` | Lenguaje query 7 statements reales (docs dicen 6), sin versión, solo UPPERCASE | Conjunto: lo usan SDK/HTTP/MCP/CLI; Individual: gramática propia |
| 9 | CLI | `src/cli.rs` + `src/cli_handlers/`, bin `vanta-cli` | (sin doc api dedicada — gap) | API humana/terminal: CRUD/search/IQL/server; mezcla concerns (spawn binario, RW en lecturas) | Individual: flags/salida propios, sin `--json` global |
| 10 | vanta-proxy | `vanta-proxy/src/server.rs:741-772` (8 endpoints), `config.toml` | `docs/api/PROXY.md` | Proxy LLM transparente (forward bytes + opt-ins), 1 auth faltante (`/snapshot`) | Individual: endpoints/config propios, rate-limit duplicado con server |
| 11 | vanta-memory | `vanta-memory/src/` (L0–L3), LLM opcional | `docs/api/VANTA_MEMORY.md` | Pipeline memoria agentes core-only, degradado sin LLM, NO expuesto en bindings (D42/D43) | Individual: crate aparte, fuera del contrato VERSIONING |

**NO son superficies** (descartados con evidencia): GraphRAG/embeddings (features del core, `docs/api/GRAPH_RAG.md`, `EMBEDDINGS.md`), `providers/` (excluidos del workspace por crash MSVC, `Cargo.toml:756-758`), `integrations/`, `scores.md`/`ERROR_HANDLING.md` (docs transversales).

## 3. Mapa conjunto vs individual

```
                 ┌──────────────┐
                 │  Rust core   │  single-owner: ficheros+WAL+VFile
                 │ src/lib.rs   │  Embedded/InMemoryEngine
                 └──────┬───────┘
        in-process │    │    │    │  (PyO3 .so / NAPI .node / wasm pkg)
        ┌──────────┘    │    └──────────┐
   Python(TS/Node/WASM │           HTTP TCP (server/proxy)
    vistas sin         │                │  + MCP stdio (hijo)
    lifecycle propio   │           IQL (todos) / CLI directo
                       │
              vanta-memory (core-only L0-L3, sin binding)
```

**Verdad orquestador (verificada en código, alimenta API-STD-13):** no hay centro único; cada runtime carga el core in-process; CLI→SDK directo salvo `--mcp` que hace `Command::new("vantadb-server").arg("--mcp").spawn()` (`src/cli_handlers/server.rs:262-324`); sin Unix sockets/SHM/WebSockets (grep vacío salvo `.gitignore:42` y nota MSVC `Cargo.toml:757`).

## 4. Blast radius / implicaciones

Read-only: ninguno. Corrección contrato (18+1) anotada para API-STD-17 (dueño `docs/api/`).

## 5. DoD task

- [x] Contrato mecánico ✅ (comandos arriba, resultados pegados)
- [x] Task file sync (este archivo)
- [x] Recitation: 11 superficies fijadas; corrección 18md+1yaml; mapa + orquestador verificados

## Context Save Point

API-STD-01 DONE. Next: API-STD-02 (Rust core ficha individual). Deuda: ninguna. WIP ajeno: ninguno.
