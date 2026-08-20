# WASM-02: Transporte WASM en el desktop (WasmBackend + mapper vanta_*)

## Metadata
- **Plan file:** docs/plans/2026-08-19-vanta-studio-fase4.md
- **Creado:** 2026-08-19
- **Estado:** ✅ COMPLETED (commit pendiente — lo verifica/commitea el lead)

## Contrato (cumplido)
1. **`WasmBackend`** en `desktop/src/transport.ts` — db lazy (`VantaDB.connect_persistent("vantadb")` con fallback `connect_idb`), persist() → `save()`/`save_idb()`, call() = `getWasmMapping` + run + persist. Factory: `getTransport()` devuelve WasmBackend cuando `VITE_VANTA_MODE === "wasm"` o `MODE === "wasm"` (antes del HttpBackend).
2. **`desktop/src/vanta-wasm-map.ts`** — tabla de 24 comandos (espejo de cobertura del http-map; `vanta_deep_link_take` queda fuera por guard Tauri-only en vanta.ts): 10 mapped (health, ingest, ingest_batch, search, get, delete, put, list, delete_by_filter, metrics), 14 unsupported con razón descriptiva (patrón WEB-04). persist=true en put/ingest/ingest_batch/delete/delete_by_filter.
3. **`desktop/src/vanta-wasm-map.test.ts`** — 14 tests (fake db): adaptación DTO, coerción de wire, persist flags, cobertura 24 comandos, degradaciones descriptivas.
4. **Builds verdes** desktop (tsc + vite) y web (`vite build --mode web`); suite node-compatible completa verde.
5. **Smoke contra el motor WASM real** (build no-modules en Node): put/ingest/get/search/list/delete_by_filter/metrics/health verificados contra shapes reales; hallazgos registrados abajo.

## Verify corridos (secuenciales, uno por vez)
- `node --test src/vanta-wasm-map.test.ts` — 14/14 pass
- `node --test src/vanta-http-map.test.ts src/vanta-wasm-map.test.ts src/vanta-deep-link.test.ts src/components/export/export-jsonl.test.ts src/components/export/status-report.test.ts` — 47/47 pass (33 existentes + 14 nuevos)
- `npm run build` (tsc && vite build) — exit 0 (dist/ sin chunk wasm; warning GraphLens 966 kB pre-existente)
- `npx vite build --mode web` — exit 0 (dist-web/)
- Smoke temp (`C:\Users\Eros\AppData\Local\Temp\opencode\smoke-wasm02.mjs` + debug*) contra `vantadb-wasm/e2e/pkg-nomodules/` — comandos core OK; query IQL devuelve `{Read: []}` (ver open items)

## Archivos tocados
- `desktop/src/vanta-wasm-map.ts` — NUEVO: mappings 24 comandos, `getWasmMapping` (throw descriptivo), `wasmMappedCommands()`, `wasmUnsupportedCommands()`, `wasmRecordFromSdk` (coerción Number de timestamps/version + Float32Array→number[]), `metricsFromWasm` (u64 string→Number).
- `desktop/src/vanta-wasm-map.test.ts` — NUEVO (14 tests).
- `desktop/src/transport.ts` — `WasmBackend` + rama wasm en `getTransport()`.
- `desktop/src/vanta-http-map.ts` — 7 helpers ahora exportados: `fromVantaValue`, `recordFromSdk`, `searchHitFromSdk`, `ingestToInput`, `genId`, `searchToRequest`, `filterToWire` (toVantaValue/mapValues siguen privados).
- `desktop/vite.config.ts` — `build.rollupOptions.external: [/vantadb-wasm\/pkg\/vantadb_wasm\.js/]` (Vite 7 no bundlea el glue ESM del .wasm; el import queda como `import()` runtime que nunca se ejecuta en builds Tauri/web).

## Degradaciones documentadas (unsupported con razón)
- **Conexiones múltiples** (connect/disconnect/list_connections/set_active): Tauri-only; WASM tiene una DB implícita.
- **Versiones** (get_version/versions): el binding no tiene parámetro/método de version history.
- **Export** (export_namespace): escribe a path de filesystem (Tauri/server); el browser no tiene path.
- **Graph** (graph_bfs/graph_dfs/graph_degree): el core devuelve `Vec<u128>` de ids visitados (no el DTO `{nodes, edges}` del desktop); graph_degree es root-based vs namespace-based. Requiere adapter engine-side (fuera de WASM-02).
- **IQL** (vanta_query + iql_autocomplete): ver open items.
- **Namespace stats** (namespace_stats): sin método en el binding; vanta.ts cae a list() count.
- **Audit** (audit_events): server/native-only.

## Hallazgos del wire WASM real (smoke, shapes verificados)
- `memory_record_to_js` serializa u64 timestamps/version como **STRINGS** (`created_at_ms`, `updated_at_ms`, `version`, `expires_at_ms`) → `wasmRecordFromSdk` coerce con `Number()` (el cast de `recordFromSdk` era una mentira en runtime).
- `vector` vuelve como **Float32Array** (JSON.stringify de uno vacío → `{}`) → `Array.from` si `instanceof Float32Array`.
- `delete_by_filter`/`compact_layout` devuelven **bigint** (wasm-bindgen u64) → `Number()`.
- `connect_persistent`/`connect_idb` son **static** en la clase `VantaDB` (no en el módulo): `mod.VantaDB.connect_persistent(...)`.
- El pkg **bundler** (`pkg/vantadb_wasm.js`) no instancia en Node (`LinkError: Import #3 ./snippets/.../inline0.js __vanta_ensure_idb_bridge: function import requires a callable`); el smoke usa el build **no-modules** (`e2e/pkg-nomodules/`) con shim `globalThis.require` + parche `let wasm_bindgen =` → `globalThis.wasm_bindgen =` + `initSync({ module: new WebAssembly.Module(bytes) })`.

## Open items (registrados, fuera de scope de WASM-02)
1. **Metadata read-back `{}`**: `put` con metadata `{kind: doc}` → get/list devuelven `metadata: {}` en el open in-memory del binding; PERO `delete_by_filter(kind=doc)` matchea 1 record (la metadata SÍ está en ShreddedRowStore). Gap de round-trip metadata en el camino wasm (el E2E de WASM-01 probó records SIN metadata). Requiere revisión de Engine/WASM-01.
2. **`vanta_query` degradado a unsupported**: `db.query("SELECT * FROM ns")` → `{Read: []}` silencioso con records presentes (el executor IQL resuelve reads contra el graph store, no el memory record store). Un read vacío silencioso es peor que degradación honesta. Re-enable cuando el engine exponga records de memoria vía query().
3. **WASM standalone browser**: el external de vite deja el glue wasm fuera de los bundles; WASM-03 debe cablear la carga real (vite-plugin-wasm o target no-modules con `<script>`).

## Decisiones
- `vanta_query` → unsupported: el IQL panel del desktop haría reads vacíos silenciosos; la degradación descriptiva es más honesta y el mensaje explica el gap de engine.
- Adaptador record WASM separado (`wasmRecordFromSdk`) en vez de tocar `recordFromSdk` (compartido con HTTP): el wire WASM tiene shape propio (strings/Float32Array) que no debe contaminar el adaptador REST.
- External de vite en vez de vite-plugin-wasm: Tauri embebe el bundle y nunca ejecuta el import wasm; el plugin agrega complejidad de runtime sin beneficiar los builds actuales (WASM-03 lo resuelve).
- Smoke en build no-modules: única variante que instancia fuera de un bundler; verificado en Node (el E2E browser real de persistencia es dominio de WASM-01).

## Context Save Point
- **Fecha:** 2026-08-19
- **Branch:** develop
- **Commit:** pendiente (el worker NO commitea — lo verifica y commitea el lead)
- **Problemas conocidos:** `pkg/` y `e2e/pkg-nomodules/` son artefactos regenerables git-ignored; worktree tenía cambios pre-existentes ajenos (SKILLS-MANIFEST.md, desktop/src-tauri/src/connections/*, docs/plans/2026-08-19-web-design-audit.md); scripts temp del smoke en `%TEMP%\opencode\` (smoke-wasm02.mjs, debug*-wasm02.mjs) — no versionados