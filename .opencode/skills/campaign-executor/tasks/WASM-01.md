# WASM-01: Persistencia browser real OPFS/IndexedDB probada

## Metadata
- **Plan file:** docs/plans/2026-08-19-vanta-studio-fase4.md
- **Creado:** 2026-08-19
- **Estado:** ✅ COMPLETED

## Contrato (cumplido)
1. **Inventario** de `vantadb-wasm` (OpfsStorage/OpfsFile, IdbStorage con bridge inline JS + BroadcastChannel + Web Locks, OpfsWorker/OpfsWorkerProxy con WORKER_TIMEOUT_MS=5000, exports VantaDB, tests browser-only `tests/wasm_tests.rs`) + verify de build (default, `--features opfs`, y `--target no-modules` para E2E).
2. **E2E de persistencia real**: put 10 → `save()`/`save_idb()` → reload real del page (`page.reload()`) → `load()`/`load_idb()` → get 10 → `PASS:10` para **OPFS e IndexedDB** (fallback Safari) en Edge 151 sobre `http://127.0.0.1`. Resultado: `ALL PASS`.
3. **Límites documentados con fuentes oficiales** (MDN + web.dev, fetched 2026-08-19) en `docs/api/WASM_PERSISTENCE.md`.

## Verify corridos (secuenciales, uno por vez)
- `wasm-pack build vantadb-wasm` — exit 0 (pkg default regenerado)
- `wasm-pack build vantadb-wasm --features opfs` — exit 0 (pkg opfs regenerado; d.ts confirma worker exports)
- `wasm-pack build vantadb-wasm --target no-modules --out-dir e2e/pkg-nomodules` — exit 0
- `node vantadb-wasm/e2e/e2e-persistence.mjs` — exit 0, output:
  ```
  [opfs] seed OK: 10 records put + saved
  [opfs] after reload: PASS:10
  [idb]  seed OK: 10 records put + saved
  [idb]  after reload: PASS:10
  ALL PASS
  ```

## Archivos tocados
- `vantadb-wasm/src/opfs.rs` — **fix bug**: `OpfsFile::open(create=false)` ahora mapea el rechazo `NotFoundError` de `getFileHandle` a `Ok(None)` (archivo ausente) en vez de propagar el error → `connect_persistent` ya funciona en directorio OPFS nuevo (primera corrida). Antes: `db.load()` fallaba con NotFoundError y `connect_persistent` no podía abrir un storage vacío.
- `vantadb-wasm/src/idb.rs` — **fix bug**: `storage()` ahora llama `__vanta_ensure_idb_bridge()` (idempotente) para registrar `globalThis.vantaIdbStorage`. Antes la función extern estaba declarada con `#[allow(dead_code)]` y NUNCA se llamaba → el bridge inline nunca se registraba → **toda** operación IDB fallaba en todos los targets.
- `vantadb-wasm/e2e/persist.html` — harness E2E (nuevo): fases seed/verify vía sessionStorage, dir OPFS único por run, cleanup IDB con `delete_idb()`, assert OPFS/IDB presentes, metadata en formato tagged `VantaValue` (`{ String: "1" }`, `{ Int: i }` — un boolean plano o string plano NO deserializa). Incluye shim `window.require` para el target no-modules (ver bug documentado abajo).
- `vantadb-wasm/e2e/e2e-persistence.mjs` — driver Playwright (nuevo): servidor estático propio en 127.0.0.1, `chromium.launch({ channel: "msedge" })`, `page.reload()` real, listeners console/pageerror, dump de log de página en fallo.
- `docs/api/WASM_PERSISTENCE.md` — doc de límites verificado (nuevo).

## Límites documentados (con fuentes)
- **OPFS**: secure context obligatorio; `getDirectory` Chrome/Edge 86+, Firefox 111+, Safari 15.2+; Baseline marzo 2023; no disponible en private browsing Safari (SecurityError/UnknownError). Fuente: MDN getDirectory + WebKit blog.
- **Cuota** (OPFS e IDB comparten la del origin): Chromium 60% del disco total por origin; Firefox 2GB por eTLD+1; Safari ~1GB con incrementos de 200MB; estimar siempre con `navigator.storage.estimate()`. Fuentes: MDN Storage quotas/eviction + web.dev Storage for the web.
- **Web Locks** (serializa escrituras IDB multi-tab): Chrome 69+, Firefox 96+, Safari 15.4+; el bridge degrada sin lock si falta. Fuente: MDN Web Locks API.
- **WASM ES module import**: Chrome/Edge estables 2026 NO soportan `import * as wasm from "...wasm"` (verificado empíricamente con un módulo mínimo de 369 bytes; MIME application/wasm rechazado). Deploy = bundler (target bundler) o `--target no-modules` con `<script>` clásico. Fuente: MDN WebAssembly ESM integration + verificación empírica del E2E.
- **inline_js snippets**: solo targets `web` y `bundler` (docs oficiales wasm-bindgen); en `no-modules` el glue emite `require()` → shim en el harness para E2E. Fuente: wasm-bindgen guide, JS snippets caveats.

## Bugs encontrados (registrados)
1. **connect_persistent fallaba en directorio nuevo** (NotFoundError) — FIX aplicado (opfs.rs). Bloqueaba la E2E.
2. **Bridge IDB nunca se registraba** (`__vanta_ensure_idb_bridge` dead code) → toda operación IDB fallaba en todos los targets — FIX aplicado (idb.rs). Bloqueaba la E2E.
3. **demo/app.js no llama `save()`/`save_idb()`** después de put — el claim de persistencia del demo es falso. NO arreglado (fuera de contrato).
4. **Sin fallback automático OPFS→IDB**: `connect_persistent` no detecta ausencia de OPFS y cae a `connect_idb` — el "fallback IndexedDB en Safari" del contrato es manual, no automático. NO arreglado (gap registrado en el doc).
5. **`docs/architecture/WASM_STORAGE_REVIEW.md` parcialmente stale**: afirma que no hay rename atómico/checksum y que IDB no tiene tests; hoy opfs.rs tiene write atómico tmp+rename + footer CRC-32 y wasm_tests.rs tiene tests IDB. NOTA en el doc, no arreglado.

## Decisiones
- E2E corre sobre `http://127.0.0.1` con **Edge real** (channel msedge): el chromium bundled de Playwright no expone OPFS de forma fiable y los probes sobre about:blank no son secure context. El build `no-modules` evita el import ES de wasm que Chrome/Edge rechazan.
- Fix de opfs.rs usa el mismo patrón NotFoundError que ya existía en `delete_file` — mínimo, consistente, sin refactor del storage.
- La metadata en el harness usa el formato tagged del enum serde `VantaValue` (canónico en `vantadb-ts/src/types.ts`) — valores planos (boolean/string) fallan la deserialización.

## Context Save Point
- **Fecha:** 2026-08-19
- **Branch:** develop
- **Commit:** pendiente (el worker NO commitea — lo verifica y commitea el lead)
- **Prerequisito E2E:** Edge instalado + `wasm-pack build vantadb-wasm --target no-modules --out-dir e2e/pkg-nomodules` antes de correr el script
- **Problemas conocidos:** pkg/ y e2e/pkg-nomodules/ son artefactos regenerables git-ignored (no versionados); worktree tenía cambios pre-existentes ajenos (SKILLS-MANIFEST.md, desktop/src-tauri/src/connections/*, docs/plans/2026-08-19-web-design-audit.md)