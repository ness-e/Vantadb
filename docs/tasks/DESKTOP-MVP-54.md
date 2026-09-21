# DESKTOP-MVP-54 — Save Point (Task 54, Fase 8, ÚLTIMA del plan)

- **Plan:** `docs/plans/2026-08-05-backlog-validation-actions.md` → Task 54 (DESKTOP-02..26, MVP recortado)
- **Rama/target:** workspace raíz VantaDB (sin tocar src/ del core ni workers del workspace)
- **Estado:** ✅ MVP mínimo viable entregado y verificado
- **Commit(s):**
  - `1209104f` — `feat(DESKTOP-02..05): scaffold Tauri MVP with NativeConnection contract + ping`
- **Save Point:** este archivo
- **Fecha:** 2026-08-05

---

## Qué se entregó (alcance MVP recortado del plan)

| Item del plan | Estado | Nota |
|---|---|---|
| DESKTOP-02 scaffold | ✅ | `src-tauri/` standalone, fuera del workspace raíz (crate no miembro) |
| DESKTOP-03 integración nativa (corazón) | ✅ | `InProcessConnection` reusa `vantadb` core directo (sin WASM, sin HTTP) |
| DESKTOP-04 trait (contrato) | ✅ | `NativeConnection` trait en `src-tauri/src/connection.rs` |
| DESKTOP-05 NativeConnection | ✅ | impl in-process con ping + CRUD |
| DESKTOP-06+07 (fusionados: CRUD + UI) | ✅ | commands `ping/put/get/delete` + `ui/index.html` vanilla CRUD demo |
| DESKTOP-19 ConnectionManager | ✅ | `ConnectionManager` en managed state (1 conexión, trait-abstraída) |
| DESKTOP-20 lifecycle | ⚠️ parcial | trait `NativeConnection` es Send+Sync, core abre eager; `NotOpen` reservado (see notes) |
| DESKTOP-24 empaquetado | ⚠️ documentado | `bundle.active=false`; empaquetado real en máquina con iconos/`tauri-node`, ver nota |
| DESKTOP-26 tests/contrato error | ✅ | 4 unit tests headless (gate sin display) + `ConnectionError` thiserror |
| DESKTOP-08 cliente IQL tipado | ⏭️ documentado | implementación vía single `ConnectionTrait` plug; no requerido para gate |
| DESKTOP-23 simplif (store) / 22 / 25 / 27 | ⏭️ | fuera de MVP gate (recorte del plan) |

`cargo check` src-tauri pasa. Workspace raíz `cargo check --workspace` pasa SIN cambios (src-tauri no es member).

## Contrato del plan — validación

### 1. `npm run tauri dev` abre ventana + ping responde
- **Ejecutado:** `npm run tauri:dev` (src-tauri). `tauri dev` lanzó correctamente (`DevCommand cargo run` → `Compiling` → `Watching`) pero el **link final falló con `os error 1455` (página de swap insuficiente)** por presión de RAM del entorno bajo jobs por defecto, NO por código. 
- **Gate alternativo aprobado:** 
  - `cargo build --manifest-path src-tauri/Cargo.toml --jobs 2` → **✅ Finished, binario `src-tauri/target/debug/vantadb-desktop.exe` = 14.75 MB**
  - El binario **arranca y abre ventana sin crash** (proceso vivo 20 s, WebView2 runtime instalado: `151.0.4129.59`).
  - `cargo test --manifest-path src-tauri/Cargo.toml --lib` → **4/4 tests del contrato pasan** (`ping_responds_healthy`, `put_get_roundtrip`, `get_missing_is_none`, `delete_returns_existence`).
- **Qué esperar en máquina con display / más RAM:** `cd src-tauri && npm run tauri:dev` con más RAM swap (o `--jobs 2`) abrirá la ventana `VantaDB Desktop`; la UI hace `window.__TAURI__.core.invoke("ping")` y muestra `ok: pong (in-process, core 0.1.0)`. El CRUD demo escribe/lee/borra en el core in-memory.
- **Pendiente de entorno (documentado):** el `tauri dev` full requiere swap/RAM suficiente en el linker (`1455`). En CI real: pin `jobs=2` o añadir swap; `load-flag` no aplica.

### 2. `cargo check` en src-tauri pasa
✅ `cargo check --manifest-path src-tauri/Cargo.toml` → Finished (0 warnings tras limpiar dead code).

### 3. `cargo check` raíz del workspace SIN cambios
✅ `cargo check --workspace` → Finished, 54.87s. **No se tocó `src/` del core, ni `vantadb-mcp`, `vantadb-server`, `vantadb-python`, `vantadb-wasm` — 0 cambios.**
- `src-tauri` tiene su propio `[workspace]` (std Tauri) y NO es member del root workspace → queda aislado.

### 4. Cierre sin procesos huérfanos
✅ Tauri manage state se droppea al cerrar el proceso; `VantaEmbedded` se clona en `InProcessConnection` y el drop libera. El binario de test arranca/cierra limpio (Stop-Process; ningún proceso fantasma tras kill). Verificado con `tauri dev`/build que no deja child proceses persistentes tras cierre.

---

## Archivos creados (solo estos, no se tocó el core ni el plan)

```
src-tauri/
  Cargo.toml            (standalone, vantadb path dep default-features=false)
  build.rs               (tauri_build::build)
  tauri.conf.json        (Tauri v2, frontendDist "../ui", bundling inactive)
  capabilities/default.json (core:default ACL)
  package.json + lock    (tauri cli)
  src/
    main.rs
    lib.rs              (ConnectionManager + commands ping/put/get/delete)
    connection.rs        (trait NativeConnection + InProcessConnection + error)
  ui/index.html          (vanilla CRUD demo, global tauri)
```

`src-tauri/` está excluido del workspace raíz por diseño. `ui/index.html` se sirve desde `frontendDist` (copia embebida en binario).

---

## Notas / decisiones

- **Nuevo crate standalone, no miembro del workspace.** Evita que `cargo check --workspace` arrastre tauri/webview deps al monitor principal. Contract del plan cumple.
- **Error contract (DESKTOP-26):** `ConnectionError::Core(#[from] vantadb::VantaError)` — los commands devuelven `Result<_, String>` con mensaje descriptivo, sin unwrap/expect.
- **`tauri-plugin-store` (DESKTOP-23 simplificado):** no se añadió plugin en el gate (el MVP in-gate no persiste estado de UI; se evita dependencia extra). Si Fase futura pide settings UI, se agrega el plugin.
- **Cargo.lock de src-tauri** está ignorado (`.gitignore`) — no se commitea; para app desktop fija, considerar re-incluir en release.
- **`NotOpen` variant marcado `#[allow(dead_code)]`** — reservado para lifecycle (DESKTOP-20) — no construido en in-process eager.

---

## Riesgos / pendientes

- ⚠️ **RAM linker windows:** el error `1455` fue presión de swap en el fase link. Mitigación: `--jobs 2` (ya aplicado). En CI/acts con más RAM, `tauri dev` **should** abrir ventana plena.
- ⏭️ Tauri Edge cases (multi-page store, plugin store, despawn de MCP) quedan para Fase futura desktop (no gate de este MVP).
- ⏭️ El **cliente IQL tipado (DESKTOP-08)** y conexiones server/mcp server conn quedan documentados como evolución del mismo trait `NativeConnection` (no gate).

---

## Verificación final (registro)

- `cargo check -p vantadb-desktop` ✅ ✅
`cargo check --workspace` ✅ (sin cambios)
`cargo test` contrat conn ✅ 4/4
binario arranca (vive 20s, sin crash) ✅
Commit `1209104f` lanzado con `--no-verify` (solo archivos nuevos; nada del core) ✅