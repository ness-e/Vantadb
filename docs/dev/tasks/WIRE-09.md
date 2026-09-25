# WIRE-09: Seguridad P0 — sandbox de paths export/import/snapshots + refuse-to-start proxy

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-24-post-investigacion-integral.md` (W2, paralelo disjunto; A2)
- **Fuente:** Backlog `P56:963` + plan `:108,120,156`
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴 · **Tipo:** Rust (bug `fix:`, seguridad)
- **Turns estimados:** 15-20
- **Creado:** 2026-09-25 · **last-synced:** 2026-09-25
- **Estado:** ✅ COMPLETED (P2-01 vanta-audit approve 2026-09-25)
- **Incógnitas (uphill):** 0 · **Pendientes (downhill):** 4 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `export_v2` ← `src/server/router.rs` + `routing.rs`; `create_snapshot` (`src/sdk/builder.rs:257`) ← `vantadb-mcp/src/handlers/tools.rs`; `create_snapshot` (`src/storage/engine/mod.rs:682`) ← builder |
| Callees | `resolve_export_path` (`src/sdk/serialization/impl_export.rs:20`) → `prevent_path_traversal` / `resolve_against_base` (guard real, fix `fc069a06f` H-SEC-IV-003) |
| Implicaciones | SDK export/import **ya guardado** — no se toca; `engine.create_snapshot` (`engine/mod.rs:641`: `join("snapshots").join(name)` **sin sanitizar**) es el hueco; proxy `/snapshot` sin auth + bind `0.0.0.0` (`config.rs:199`); auth del endpoint = API-05 (frontera: aquí solo refuse-to-start) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `src/server/handlers.rs:676-747,1504-1529`, `src/storage/engine/mod.rs:630-690`, `src/sdk/builder.rs:250-265`, `src/sdk/serialization/impl_export.rs:17-28`, `vanta-proxy/src/server.rs:736-840`, `vanta-proxy/src/config.rs:190-210`
- **Referencias hacia dentro:** `run_db_op`, `FIND-07` server (patrón refuse-to-start a copiar)
- **Referencias entrantes:** router MCP tools → builder → engine (cadena snapshot completa)
- **Veredicto impacto:** medio — 2 archivos + config; sin cambio de firmas públicas; comportamiento observable cambia (error donde antes había write) = documentar en Notas

## Contrato
"`cargo test -p vantadb snapshot_traversal export_paths` + `cargo test -p vanta-proxy refuse_start` verdes; `name=../x` y `path` fuera de base devuelven error; bind no-loopback sin key no arranca."

## Spec (SDD — Phase 1b)
No es feature-add: validación de paths + refuse-to-start son cambios de comportamiento, cero símbolos públicos nuevos. Sin sección Spec (justificado por evidencia: ninguna firma `pub` cambia).

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** SDK export/import siguen funcionando con paths legítimos (guard existente intacto); `restore_from` (que sí valida) sin cambios; MCP `snapshot_create` sigue pasando por builder.
- **Comandos de verificación:** `cargo test -p vantadb snapshot_traversal export_paths` + `cargo test -p vanta-proxy refuse_start` + `cargo clippy -p vantadb -p vanta-proxy --all-targets -- -D warnings`
- **Deuda pendiente:** ninguna al abrir

## Recitation
```
=== RECITATION ===
Objetivo activo: WIRE-09 — sandbox paths + refuse-to-start
Estado: implementado + verificado (desde: PENDING)
Última acción: Steps 1-4 ejecutados con RED→GREEN por slice; commit atómico (sin push — pushea el lead)
Resultado: OK (contrato verde)
Próxima acción: vanta-audit (P2-01) + WIRE-01 (serial proxy — no tocar inject.rs/capture.rs/cost.rs)
Contrato: ver ## Contrato
Invariantes: SDK guard intacto; restore_from intacto; MCP snapshot_create intacto; default behavior sin export_base_dir intacto (fallback `..`-check); 12 integration tests proxy verdes sin cambios
Deuda: ninguna (saldo neto: fix elimina deuda; NOTICED BUT NOT TOUCHING: mirror_data_dir no detecta dest-dentro-de-src — inalcanzable post-fix con nombres validados; API-05 owns auth GET /snapshot)
Queda pendiente (orquestador): P2-01 review por vanta-audit (no auto-revisar); cross-model doubt skipped (contexto no-interactivo); OCR delegation = preview sin API key (sin findings Critical/High)
last-synced: 2026-09-25
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)
**Saldo neto:** Sin deuda (fix de seguridad, elimina deuda existente).

## Definition of Done (3 niveles)
- **Task:** contrato verde + clippy `-D warnings` + fmt
- **Commit:** atómico (~100 líneas), `fix:` + WIRE-09, verify mecánico antes
- **Release:** N/A (no toca versionado; justificar: fix interno sin cambio de API)

## Herramientas necesarias
- `cargo check/test/clippy/fmt -p vantadb -p vanta-proxy`, codegraph_explore, `campaign_verify_cmd`
- **Skills cargadas (SDP):** source-driven-development (base campaign) + security-and-hardening (boundary HTTP path/auth — input hostil) + systematic-debugging (Iron Law: repro→hipótesis→RED→fix) + api-and-interface-design (observable de endpoints cambia) + test-driven-development (test RED primero) + incremental-implementation (1 variable por slice) + doubt-driven-development (stakes seguridad)

## Investigation Notes
- Guard SDK verificado: `resolve_export_path` en las 3 rutas export/import (`impl_export.rs:190,244,340`); fix `fc069a06f`.
- Hueco snapshot: `engine/mod.rs:641` join sin sanitizar; `restore_from` sí valida (paridad a copiar).
- Proxy: `/snapshot` ruta en `server.rs:744`, handler `:816`; auth solo en handlers chat (`:184-185`); default bind `config.rs:199`.
- CodeGraph: 4 símbolos sin covering tests → este task los crea.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 — causa raíz verificada file:line |
| Pendientes | 4 steps |
| % completado | 0% |

## Fase 1 — Evidencia de Debugging (GATE Bug)
- **Repro:** `POST /api/v2/snapshots {"name":"../escape"}` → escribe fuera de `snapshots/`; `GET /snapshot` proxy sin credencial → 200.
- **Hipótesis:** nombre sin validar en `create` (restore sí valida) + auth ausente en ruta observabilidad proxy.
- **1 variable:** un slice por step (snapshot-name → export/import e2e → refuse-to-start).
- **Test RED:** test de escape que hoy escribe-fuera; tras fix devuelve error (RED→GREEN).

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — trust boundary HTTP directo (`path`/`name` hostiles) → `security-and-hardening` cargada; abuse-case = test RED; sin secretos/credenciales nuevas.
- [x] **PERFORMANCE** — N/A justificado: rutas admin esporádicas, fuera de hot path search/ingest.

## Steps
### Step 1: Sanitizar `create_snapshot`
- **Archivos:** `src/storage/engine/mod.rs:630-690`
- **Acción:** validar `name` como identificador plano (paridad `restore_from`); test escape `../` → error
- **Verify:** `cargo test -p vantadb snapshot_traversal`
- **Estado:** ✅ DONE — `Self::validate_snapshot_name(name)?` en ambas variantes `create_snapshot` (unix + windows/wasm); RED = stack overflow real (traversal → mirror recursivo infinito), GREEN 1/1. Hallazgo: `../escape` no solo escribe fuera — cuelga el mirror (DoS). Registrado en Notas.

### Step 2: E2E export/import
- **Archivos:** `src/server/handlers.rs:699-747`, tests server
- **Acción:** test de escape contra server real con/sin `VANTADB_EXPORT_BASE_DIR` (el guard existe — se verifica, no se reimplementa)
- **Verify:** `cargo test -p vantadb export_paths`
- **Estado:** ✅ DONE — guard verificado (3/3 rutas cubiertas); Prove-It reveló 4ª ruta SIN guard: `bulk_import_file` (alcanzable desde `import_v2` format="bulk") → GREEN con `resolve_export_path` (1 línea) + `pub(crate)` visibility. 3/3 tests verdes. Sin cambio de behavior default (fallback intacto sin base dir).

### Step 3: Refuse-to-start proxy
- **Archivos:** `vanta-proxy/src/config.rs`, arranque server
- **Acción:** bind no-loopback sin key → error al arrancar (copiar `FIND-07` del server)
- **Verify:** `cargo test -p vanta-proxy refuse_start`
- **Estado:** ✅ DONE — `ProxyConfig::validate_startup` (pub(crate), cero símbolos públicos) + `AuthDb::provisioned_user_count` + gate en `from_engine` (cubre `new`/`main`); "sin key" = 0 entidades `user`. RED probado (gate comentado → startup Ok). 6/6 verdes; suite proxy completa verde (169 lib + 12 integration files, todos siembran user → sin cambios). Sin override `--allow-insecure`: remedies loopback/provision ya existen (decisión en Notas).

### Step 4: Cierre
- **Archivos:** —
- **Acción:** fmt+clippy+nextest scope, `dev-tools/verify_changed.ps1`, commit `fix:`, skill progreso
- **Verify:** `campaign_verify_cmd` con el contrato
- **Estado:** ✅ DONE — fmt ✅, clippy `-D warnings` ✅ (vantadb + proxy, all-targets), contrato verde, commit atómico sin push. OCR = preview sin API key (sin findings). P2-01 pendiente (vanta-audit separado).

## Dependencias
- Ninguna (destraba: gate de anuncio; API-05 owns auth `/snapshot` — frontera respetada, no bloqueo)

## Review (GATE P2-01)
- **Revisor:** vanta-audit (seguridad) — distinto del implementador
- **Enfoque:** ¿sandbox completo (create + export + import)? ¿refuse-to-start sin bypass?
- **Cómo se probó:** tests de escape reales, no auto-reporte
- **Checklist anti-hábitos:** según plantilla (verificar al revisar)
- **Veredicto:** ✅ approve (vanta-audit 2026-09-25: sandbox completo 4 rutas, re-ejecución propia 4/4+6/6 + clippy, checklist anti-hábitos 10/10, colaterales acordados)

## Notas
- WIP ajeno PROHIBIDO: archivos de API-01 en curso (`src/sdk/types/`, `QueryResult`) — no tocar.
- Cambio observable (error donde había write) → nota en commit, sin cambio de API.
- WIRE-09 hallazgos de implementación (2026-09-25):
  - `../escape` en `create_snapshot` no solo escribía fuera: el mirror recursivo sobre el dir escapado (dentro de `data/` pero fuera del subtree `snapshots/` excluido) diverge → stack overflow/DoS. El fix lo cierra en la frontera.
  - `bulk_import_file` era la 4ª ruta de path sin sandbox (las 3 de `impl_export.rs` sí lo tenían). Fix de 1 línea por paridad.
  - Refuse proxy sin override `--allow-insecure` deliberado: remedies (loopback / provisionar user) ya existen; cero superficie config nueva.
  - Default proxy `host = "0.0.0.0"`: instalación fresca con store vacío ahora rehúsa arrancar (mensaje con 2 remedios) — cambio observable documentado.
  - NOTICED BUT NOT TOUCHING: `mirror_data_dir` no detecta dest-dentro-de-src (inaccesible post-fix con nombres validados) → ¿FIND-*? Lo decide el lead.
  - Doubt: cross-model skipped (contexto no-interactivo, anunciado según skill).
