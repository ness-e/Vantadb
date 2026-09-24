# Task API-STD-18 — Implementación: waves, comandos, MCP local, release

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (planificación, sin código — PLAN read-only)
> **Fecha:** 2026-09-24

## 1. Waves ordenadas (dependencias → orden)

| Wave | Scope (decisiones 15) | Verify exacto | Rollback |
|---|---|---|---|
| W0 fundación | tipos base + codegen 1-schema; error `code+message+context` + RFC 9457; casing fronteras; `u128→string` (incl. napi FIND-NEW-01) | `dev-tools/verify_changed.ps1` + `cargo test --test sdk_serialization` | revert commit + re-verify |
| W1 bindings | `score` TS; `put_batch` array Py; `getNode/deleteNode` Py; `roots` bigint; `searchMulti` Py; `bulk_import` TS/Node; `method` inline; sparse+`exclude_superseded` Node; `import_records` wasm-fix | `cargo test --test python_sdk_boundary` + `tsc --noEmit` + `npm test` (ts) | por binding, revert + tests |
| W2 HTTP/OpenAPI | YAML owner; recursos plurales; `/api/v2` total; `/messages`; cursor+`has_more`; status=YAML; `RecordInput` opcional; case/tipos únicos | `cargo test --test openapi_yaml_parity` + curl por grupo | revert + parity |
| W3 MCP | 1 nombre/tool; Schemars estricto; `invalid_params`; tipados; prompts/resources/tools; `thread_id` string | MCP smoke: `vanta-cli server --mcp` + `memory_put/get/search` | revert + re-listar tools |
| W4 proxy | auth `/snapshot`; plurales; `space_id`; upstream sin self-loop; `ttl=0` off; 1 rate-limit | `cargo test -p vanta-proxy` + curl auth | revert + config |
| W5 IQL | `IQL_VERSION`; sintaxis+case; `==`/int/quotes; AST JSON | `cargo test` parser (`mod.rs`) + YAML ejemplo válido | revert + tests |
| W6 CLI | flags simétricos; `--json` global completo; TTY-truncate; `count` exit≠0; SDK-path; RO | `--help` × comando + snapshot `--json` | revert |
| W7 memory | API Rust estable (NO exponer, Gate P); pagar D37/D21/MEM-16 con benchmark | `cargo test -p vanta-memory` + `canonical_p99` si toca hot | revert |
| W8 cierre | VERSIONING 11 superficies; docs mismo-PR (Regla 3); MCP local refresh; `/audit quick`; `/ship` | `validate-docs-coverage.ps1` + `dev-tools/verify.ps1` + MCP re-smoke | — |

Cada commit: `feat!:` + task ID (breaking). Deuda Regla 6: pagar P2-5 en W1, P2-8 en W0. Benchmarks Regla 9 donde aplique (`canonical_p99`, entorno+fecha).

## 2. MCP local refresh (tras W3/cualquier cambio tools)

```powershell
pwsh vanta-mcp-local.ps1 -DbPath C:/Users/Eros/.vantadb
# restart opencode; re-listar tools; smoke memory_put/get/search
# si cambia opencode.jsonc → pegar diff en el commit
```

## 3. Release siguiente (Regla 7)

`develop`→PR→`main`; release-plz bump+CHANGELOG+tags (nunca manual); OIDC verificado sin tokens (17); `/audit quick` (`just verify`/`verify.ps1`); `/ship` GO/NO-GO; `/rollback` si falla. `vantadb-pro` intocable (open-core).

## 4. DoD

- [x] Contrato ✅ (9 waves + comandos + rollback + MCP + release) · Task file sync · Recitation: implementación lista para ejecutar

## Context Save Point

API-STD-18 DONE — PLAN COMPLETO 18/18. Next: `/pipeline task API-STD-0X` ejecución o `/audit quick`.
