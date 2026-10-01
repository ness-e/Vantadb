---
title: "FIND-176 — Wizard setea ORT_DYLIB_PATH al directorio del store (ort carga un archivo)"
kind: task
---

# FIND-176: wizard deja embeddings en dummy — ORT_DYLIB_PATH apuntaba al directorio

## Metadata

- **Fuente:** `docs/dev/Backlog.md:336` (FIND-176, 🟠 Alta) · Origen: slice DEF-08 (2026-09-27)
- **Carril:** pre-release 0.8.0 · **Decisión owner:** 2026-10-01 (fix en este carril)
- **Esfuerzo:** 🟢 (1 script + 1 test) · **Tipo:** bug-fix
- **Branch:** develop · **Commit:** lo hace el LEAD (sub-agente NO commitea — mandato del carril)
- **Estado:** ✅ COMPLETED — fix + test + verificación; revisión del lead (4to sitio cubierto + 8 PASS); commit = lead

## Contexto verificado del bug (mecanismo exacto)

1. **`ort` carga un ARCHIVO, no un directorio** — `ort::init_from(path)` →
   `libloading::Library::new(&absolute_path)` (`ort-2.0.0-rc.13/src/lib.rs:136`)
   → `LoadLibraryExW` en Windows. Con un directorio falla (`Dlopen`).
2. **El probe lee `ORT_DYLIB_PATH` verbatim** — `resolve_ort_dylib_path()`
   (`src/embedding_health.rs:220-249`) y `ort_load_error_at` → `ort::init_from`
   (`:257-264`); un dir → `fallback:true`, `reason:"dylib"`.
3. **`setup-embeddings.ps1` seteaba el DIRECTORIO** en los 3 caminos de
   `Ensure-OrtNative` (`:191/:207/:221` originales — `$env:ORT_DYLIB_PATH = $store`).
   El launcher `vanta-mcp-local.ps1:78-91` ya apuntaba al archivo (correcto).
4. **Evidencia DEF-08:** (A) `ORT_DYLIB_PATH=<store dir>` → fallback=true
   reason=dylib ("LoadLibraryExW failed"); (B) `ORT_DYLIB_PATH=<store>\onnxruntime.dll`
   → fallback=false.

## Decisión de fix

`$env:ORT_DYLIB_PATH = $dll` en los 3 sitios — `$dll` ya es exactamente
`Join-Path $store 'onnxruntime.dll'` en los 3 puntos (definido en `:188` y
retornado por `Install-OrtStore` en `:206/:220`), así que es DRY y sigue el
valor real instalado. La alternativa literal del backlog
(`Join-Path $store 'onnxruntime.dll'`) es equivalente; se descartó por duplicar
la expresión que `$dll` ya tiene.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Consumidores del valor | live test del wizard (`vanta-cli status --json` → probe `embedding_health`) · sesión del usuario que exporte la var · `vanta-mcp-local.ps1` (solo confía en env pre-seteado; si está unset autodetecta el archivo él mismo) |
| Callees | `Get-OrtStore`, `Install-OrtStore`, `Get-DllVersion` (sin cambios) |
| Implicaciones | Sin cambio de API ni formato on-disk. Solo el valor de la variable de sesión. |

**NOTICED BUT NOT TOUCHING:** `Ensure-OrtNative:194-202` acepta un
`ORT_DYLIB_PATH` pre-seteado apuntando a un DIRECTORIO (valida que haya un dll
≥1.27 debajo pero no normaliza la var al archivo) — misma clase de bug, fuera
del alcance de los 3 sitios; candidato a FIND si el owner quiere cubrirlo.

> **Revisión del lead (2026-10-01):** CUBIERTO — `Ensure-OrtNative` ahora normaliza
> el path pre-seteado al archivo (`$env:ORT_DYLIB_PATH = $found`); el test de
> regresión suma el caso (run #2: `preset/normalized-to-dll`) → 8 PASS.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos/secciones):** `setup-embeddings.ps1` (515 L) ·
  `src/embedding_health.rs` (346 L) · `vanta-mcp-local.ps1:60-99` ·
  `ort-2.0.0-rc.13/src/lib.rs` (loader, registry cache) · `docs/dev/tasks/DEF-08.md` (contexto)
- **Referencias hacia dentro:** wizard ← `scripts/install.ps1:105`, `scripts/install.sh:156`
  (chain), `docs/user/QUICKSTART.md:278`, `README.md:275`, `docs/dev/strategy/DISTRIBUTION.md:242`
- **Referencias entrantes a lo editado:** el valor se consume en la misma sesión (live test) y por
  cualquier proceso hijo; los tests Rust `tests/embedding_fallback_visibility.rs` /
  `vantadb-mcp/tests/embedding_visibility.rs` fijan sus propios valores (no dependen del wizard)
- **Veredicto impacto:** **bajo** — 1 script PowerShell + 1 test nuevo; cero Rust; sin superficie de contrato

## Contrato / Verificación

- **RED:** `pwsh -NoProfile -File dev-tools/scripts/test-find176-wizard-ort-dylib.ps1` →
  wizard imprime `ORT_DYLIB_PATH=...\VantaDB\onnxruntime` (dir) → 2 PASS / 4 FAIL.
- **GREEN (fix):** mismo comando → 6 PASS / 0 FAIL; valor resuelto
  `...\VantaDB\onnxruntime\onnxruntime.dll`.
- **Carga real por ort (OS-level, replica el camino DEF-08):**
  `LoadLibraryExW(<store dir>)` → handle=0, win32_error=126 ·
  `LoadLibraryExW(<store>\onnxruntime.dll)` → handle válido, error=0.
- **Wizard:** corre real con `-NonInteractive -SkipLiveTest -NoProxy` contra
  store + DB TEMPORALES (override de `LOCALAPPDATA` en proceso hijo); nunca toca
  el store persistente del usuario ni `~/.vantadb`.

## Steps

- [x] S1 — Fix 3 sitios (`$store` → `$dll`) + comentario de invariante en `Ensure-OrtNative`
- [x] S2 — Test ejecutable RED→GREEN (`dev-tools/scripts/test-find176-wizard-ort-dylib.ps1`)
- [x] S3 — Verificación ort (loader OS-level + fuente del crate) y sintaxis PS

## Evidencia

1. RED: `pwsh -NoProfile -File dev-tools/scripts/test-find176-wizard-ort-dylib.ps1`
   → `FIND-176 RED: 2 PASS / 4 FAIL` — valor: `...\VantaDB\onnxruntime` (directorio).
2. Fix: 3 líneas (`$env:ORT_DYLIB_PATH = $store` → `$dll`) + 2 líneas de comentario.
3. GREEN: mismo comando → `FIND-176 GREEN: 6 PASS / 0 FAIL` — valor:
   `...\VantaDB\onnxruntime\onnxruntime.dll` (`path/is-leaf` + `path/is-store-dll` ✅).
4. Loader: P/Invoke `LoadLibraryExW` — dir: `handle=0 win32_error=126`; archivo:
   `handle≠0 win32_error=0`; fuente: `ort-2.0.0-rc.13/src/lib.rs:136` (`libloading::Library::new`).
5. Sintaxis PS del wizard: parser 0 errores · `git diff --check` exit 0.

## Deuda / notas

- Probe DEF-08 end-to-end con binario `--features embed-local`: el binario
  feature-build quedó eliminado por un `cargo clean`/rebuild concurrente de otra
  sesión sobre `target/` compartido (5 cargo + 2 rustc activos; disco 0 → ~61 GB
  libres). Reintentable: `cargo build -p vantadb --features embed-local --bin vanta-cli`
  + probe A/B; no bloquea el fix (evidencia OS-level + DEF-08 la cubren).
- Test nuevo: no wired a `verify.ps1` (decisión de gate del LEAD).
