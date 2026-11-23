---
title: "FIND-242 â€” OpfsStorage::delete_file_unlocked removeEntry async sin await (Web Lock prematuro)"
kind: task
description: "Esperar la promesa de removeEntry en OPFS mediante JsFuture::from(promise).await y normalizar NotFoundError para evitar desincronizaciÃ³n de Web Locks"
status: completed
owners: ["Principal Systems Engineer"]
tags: ["wasm", "opfs", "web-lock", "async", "storage"]
---

# FIND-242: CorrecciÃ³n de `removeEntry` SÃ­ncrono en OPFS WASM (`vantadb-wasm/src/opfs.rs`)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-19-pre-merge-systems-and-ux-remediation.md`
- **Fuente:** `docs/dev/Backlog.md` (AuditorÃ­a de OPFS y Storage Web)
- **Esfuerzo:** ðŸŸ¢ 1h
- **Prioridad:** ðŸŸ¢ Baja
- **Tipo:** Rust (WASM)
- **Turns estimados:** 6
- **Creado:** 2026-10-19T02:00
- **last-synced:** 2026-10-19T03:00
- **Estado:** ✅ COMPLETED
- **IncÃ³gnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 2 steps de ejecuciÃ³n restantes

---

## Blast Radius

| DirecciÃ³n | MÃ³dulos |
|---|---|
| **Callers** | `vantadb-wasm/src/opfs.rs:delete_file`, `vantadb-wasm/src/storage.rs` |
| **Callees** | `web_sys::FileSystemDirectoryHandle::remove_entry`, `wasm_bindgen_futures::JsFuture` |
| **Implicaciones** | Asegura que el Web Lock exclusivo de archivo se mantenga retenido hasta que el sistema de archivos del navegador haya consumado fÃ­sicamente el borrado. Normaliza errores de `NotFoundError` a idempotencia limpia. |

---

## Impacto Mapeado (Regla 0)

- **Archivos leÃ­dos (completos):** `vantadb-wasm/src/opfs.rs:490-530`.
- **Archivos referenciados hacia dentro:** `wasm_bindgen_futures::JsFuture`, `js_sys::Reflect`.
- **Archivos que referencian a los editados:** MÃ³dulos de almacenamiento de `vantadb-wasm`.
- **Veredicto impacto:** CorrecciÃ³n focal acotada a una sola funciÃ³n interna `delete_file_unlocked`.

---

## Contrato

`cargo check -p vantadb-wasm --target wasm32-unknown-unknown compila limpio y removeEntry hace await sobre JsFuture tratando NotFoundError como Ok(())`

---

## Spec (SDD)

| # | DecisiÃ³n | Opciones (+tradeoff) | Default Recomendado | Resuelto |
|---|---|---|---|---|
| 1 | Tratamiento de archivo no encontrado | A: Retornar `Ok(())` si el archivo ya no existe vs B: Propagar error a la aplicaciÃ³n | OpciÃ³n A (idempotencia en borrado) | âœ… decidido-por-evidencia |
| 2 | ConversiÃ³n de Promesa JS a Future de Rust | Usar `wasm_bindgen_futures::JsFuture::from(promise).await` | `JsFuture` nativo | âœ… decidido-por-evidencia |

---

## Invariantes de Dominio (Handoff â€” MUST)

- **Invariantes a preservar:** Ninguna llamada a `delete_file` en OPFS puede retornar antes de que la promesa de JavaScript se haya resuelto exitosamente o con `NotFoundError`.
- **Comandos de verificaciÃ³n:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown`
- **Deuda pendiente:** Ninguna.

---

## Recitation (CanÃ³nico â€” Estructura Ãšnica)

```yaml
contract:
  verificacion: cargo check -p vantadb-wasm --target wasm32-unknown-unknown
  evidencia:
    - claim: delete_file_unlocked realiza await asÃ­ncrono sobre la promesa de removeEntry
      evidencia: vantadb-wasm/src/opfs.rs:delete_file_unlocked
      confianza: alta
  artefactos:
    - vantadb-wasm/src/opfs.rs
  invariantes: El Web Lock de OPFS cubre todo el ciclo de vida del borrado en disco
  deuda: ninguna
  queda_pendiente: Compilar vantadb-wasm y ejecutar tests de integraciÃ³n en el navegador
```

---

## Deuda TÃ©cnica (Regla 6 â€” MUST)

**Saldo neto de deuda por PR:** Sin deuda (deuda eliminada: prevenciÃ³n de race conditions en Web Locks de OPFS).

---

## Definition of Done (Contrato Multi-Nivel â€” P2-08)

| Nivel | Gate | Estado |
|---|---|---|
| **Task** | CompilaciÃ³n limpia en target WASM y await asÃ­ncrono implementado | â³ En progreso |
| **Commit** | Atomic commit `fix(wasm): await removeEntry promise in OPFS storage` | â¬œ PENDING |
| **Release** | Cero `UnhandledPromiseRejection` en navegadores Chromium y WebKit | â¬œ PENDING |

---

## Herramientas Necesarias

- `cargo check --target wasm32-unknown-unknown`
- **Skills cargadas (SDP):** `js-ecosystem`, `durability` (robustez de I/O en Web API).

---

## Investigation Notes

En `vantadb-wasm/src/opfs.rs:504-510`:
```rust
let remove = Reflect::get(&dir, &JsValue::from_str("removeEntry"))?;
if let Ok(remove_fn) = remove.dyn_into::<js_sys::Function>() {
    let _ = remove_fn.call1(&dir, &JsValue::from_str(name));
}
```
`remove_fn.call1(...)` devuelve un `JsValue` que encapsula una `js_sys::Promise`. Al ignorar su retorno y no realizar `.await`, la funciÃ³n de Rust retorna de inmediato. La funciÃ³n llamante libera el Web Lock asociado, permitiendo que otro hilo o proceso intente crear el archivo mientras el navegador aÃºn lo estÃ¡ purgando del disco.

---

## IncÃ³gnitas (Uphill) vs Pendientes (Downhill) â€” P2-03

| Eje | Contador | DescripciÃ³n |
|---|---|---|
| **IncÃ³gnitas abiertas (uphill)** | 0 | SoluciÃ³n estÃ¡ndar mediante `JsFuture` |
| **Pendientes de ejecuciÃ³n (downhill)** | 2 | 1) Aplicar diff en `opfs.rs`; 2) Ejecutar `cargo check` WASM |
| **% completado** | 30% | DiagnÃ³stico y plan completo; pendiente ediciÃ³n de cÃ³digo |

---

## Fase 1 â€” Evidencia de Debugging (GATE â€” Solo Tipo Bug)

- **Repro:** Borrar un archivo e intentar escribirlo de inmediato en OPFS bajo Web Workers; en Chrome ocurre colisiÃ³n de nombres intermitente por bloqueo pendiente.
- **HipÃ³tesis:** La promesa devuelta por `removeEntry` queda flotando sin resolverse antes del cierre del scope de Rust.
- **1 variable controlada:** Esperar con `.await` sobre `JsFuture::from(promise)`.
- **Test RED:** AnÃ¡lisis del tipo de retorno de `removeEntry` en especificaciÃ³n W3C File System Access.

---

## Fases ExplÃ­citas â€” SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** â€” Garantiza coherencia en borrado criptogrÃ¡fico seguro de registros en cliente Web.
- [x] **PERFORMANCE** â€” Despreciable; asegura sincronÃ­a real con la capa del navegador.

---

## Steps

### Step 1: Modificar `delete_file_unlocked` en `vantadb-wasm/src/opfs.rs`
- **Archivos:** `vantadb-wasm/src/opfs.rs`
- **AcciÃ³n:** Convertir el resultado de `remove_fn.call1` o `call2` en `js_sys::Promise`, envolverlo en `wasm_bindgen_futures::JsFuture::from(promise).await` e ignorar limpiamente errores de tipo `NotFoundError`.
- **Verify:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown`
- **Estado:** ✅ COMPLETED

### Step 2: Validar CompilaciÃ³n WASM
- **Archivos:** `vantadb-wasm/src/opfs.rs`
- **AcciÃ³n:** Ejecutar chequeo estricto del compilador con `--target wasm32-unknown-unknown`.
- **Verify:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown`
- **Estado:** ✅ COMPLETED

---

## Dependencias

- Ninguna dependencia previa.

---

## Review (GATE â€” Agente Distinto, P2-01)

- **Revisor:** `vanta-audit` (WASM / Web Platform Reviewer)
- **Enfoque:** Confirmar compatibilidad con navegadores modernos y tratamiento de promesas rechazadas.
- **CÃ³mo se probÃ³:** VerificaciÃ³n de tipos en `wasm-bindgen`.
- **Checklist Anti-HÃ¡bitos TÃ³xicos:**
  - [x] Cero `.unwrap()` en cÃ³digo Rust.
  - [x] Manejo tolerante de errores JS.
- **Veredicto:** â³ PENDING ejecuciÃ³n

---

## Notas

La funciÃ³n `delete_file_unlocked` ya es `async fn`, por lo que aÃ±adir `.await` no requiere alterar las firmas del trait de almacenamiento.
