---
title: "FIND-242 — OpfsStorage::delete_file_unlocked removeEntry async sin await (Web Lock prematuro)"
kind: task
description: "Esperar la promesa de removeEntry en OPFS mediante JsFuture::from(promise).await y normalizar NotFoundError para evitar desincronización de Web Locks"
status: in_progress
owners: ["Principal Systems Engineer"]
tags: ["wasm", "opfs", "web-lock", "async", "storage"]
---

# FIND-242: Corrección de `removeEntry` Síncrono en OPFS WASM (`vantadb-wasm/src/opfs.rs`)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-19-pre-merge-systems-and-ux-remediation.md`
- **Fuente:** `docs/dev/Backlog.md` (Auditoría de OPFS y Storage Web)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢 Baja
- **Tipo:** Rust (WASM)
- **Turns estimados:** 6
- **Creado:** 2026-10-19T02:00
- **last-synced:** 2026-10-19T03:00
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 2 steps de ejecución restantes

---

## Blast Radius

| Dirección | Módulos |
|---|---|
| **Callers** | `vantadb-wasm/src/opfs.rs:delete_file`, `vantadb-wasm/src/storage.rs` |
| **Callees** | `web_sys::FileSystemDirectoryHandle::remove_entry`, `wasm_bindgen_futures::JsFuture` |
| **Implicaciones** | Asegura que el Web Lock exclusivo de archivo se mantenga retenido hasta que el sistema de archivos del navegador haya consumado físicamente el borrado. Normaliza errores de `NotFoundError` a idempotencia limpia. |

---

## Impacto Mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-wasm/src/opfs.rs:490-530`.
- **Archivos referenciados hacia dentro:** `wasm_bindgen_futures::JsFuture`, `js_sys::Reflect`.
- **Archivos que referencian a los editados:** Módulos de almacenamiento de `vantadb-wasm`.
- **Veredicto impacto:** Corrección focal acotada a una sola función interna `delete_file_unlocked`.

---

## Contrato

`cargo check -p vantadb-wasm --target wasm32-unknown-unknown compila limpio y removeEntry hace await sobre JsFuture tratando NotFoundError como Ok(())`

---

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Default Recomendado | Resuelto |
|---|---|---|---|---|
| 1 | Tratamiento de archivo no encontrado | A: Retornar `Ok(())` si el archivo ya no existe vs B: Propagar error a la aplicación | Opción A (idempotencia en borrado) | ✅ decidido-por-evidencia |
| 2 | Conversión de Promesa JS a Future de Rust | Usar `wasm_bindgen_futures::JsFuture::from(promise).await` | `JsFuture` nativo | ✅ decidido-por-evidencia |

---

## Invariantes de Dominio (Handoff — MUST)

- **Invariantes a preservar:** Ninguna llamada a `delete_file` en OPFS puede retornar antes de que la promesa de JavaScript se haya resuelto exitosamente o con `NotFoundError`.
- **Comandos de verificación:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown`
- **Deuda pendiente:** Ninguna.

---

## Recitation (Canónico — Estructura Única)

```yaml
contract:
  verificacion: cargo check -p vantadb-wasm --target wasm32-unknown-unknown
  evidencia:
    - claim: delete_file_unlocked realiza await asíncrono sobre la promesa de removeEntry
      evidencia: vantadb-wasm/src/opfs.rs:delete_file_unlocked
      confianza: alta
  artefactos:
    - vantadb-wasm/src/opfs.rs
  invariantes: El Web Lock de OPFS cubre todo el ciclo de vida del borrado en disco
  deuda: ninguna
  queda_pendiente: Compilar vantadb-wasm y ejecutar tests de integración en el navegador
```

---

## Deuda Técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (deuda eliminada: prevención de race conditions en Web Locks de OPFS).

---

## Definition of Done (Contrato Multi-Nivel — P2-08)

| Nivel | Gate | Estado |
|---|---|---|
| **Task** | Compilación limpia en target WASM y await asíncrono implementado | ⏳ En progreso |
| **Commit** | Atomic commit `fix(wasm): await removeEntry promise in OPFS storage` | ⬜ PENDING |
| **Release** | Cero `UnhandledPromiseRejection` en navegadores Chromium y WebKit | ⬜ PENDING |

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
`remove_fn.call1(...)` devuelve un `JsValue` que encapsula una `js_sys::Promise`. Al ignorar su retorno y no realizar `.await`, la función de Rust retorna de inmediato. La función llamante libera el Web Lock asociado, permitiendo que otro hilo o proceso intente crear el archivo mientras el navegador aún lo está purgando del disco.

---

## Incógnitas (Uphill) vs Pendientes (Downhill) — P2-03

| Eje | Contador | Descripción |
|---|---|---|
| **Incógnitas abiertas (uphill)** | 0 | Solución estándar mediante `JsFuture` |
| **Pendientes de ejecución (downhill)** | 2 | 1) Aplicar diff en `opfs.rs`; 2) Ejecutar `cargo check` WASM |
| **% completado** | 30% | Diagnóstico y plan completo; pendiente edición de código |

---

## Fase 1 — Evidencia de Debugging (GATE — Solo Tipo Bug)

- **Repro:** Borrar un archivo e intentar escribirlo de inmediato en OPFS bajo Web Workers; en Chrome ocurre colisión de nombres intermitente por bloqueo pendiente.
- **Hipótesis:** La promesa devuelta por `removeEntry` queda flotando sin resolverse antes del cierre del scope de Rust.
- **1 variable controlada:** Esperar con `.await` sobre `JsFuture::from(promise)`.
- **Test RED:** Análisis del tipo de retorno de `removeEntry` en especificación W3C File System Access.

---

## Fases Explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — Garantiza coherencia en borrado criptográfico seguro de registros en cliente Web.
- [x] **PERFORMANCE** — Despreciable; asegura sincronía real con la capa del navegador.

---

## Steps

### Step 1: Modificar `delete_file_unlocked` en `vantadb-wasm/src/opfs.rs`
- **Archivos:** `vantadb-wasm/src/opfs.rs`
- **Acción:** Convertir el resultado de `remove_fn.call1` o `call2` en `js_sys::Promise`, envolverlo en `wasm_bindgen_futures::JsFuture::from(promise).await` e ignorar limpiamente errores de tipo `NotFoundError`.
- **Verify:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown`
- **Estado:** ⬜ PENDING

### Step 2: Validar Compilación WASM
- **Archivos:** `vantadb-wasm/src/opfs.rs`
- **Acción:** Ejecutar chequeo estricto del compilador con `--target wasm32-unknown-unknown`.
- **Verify:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown`
- **Estado:** ⬜ PENDING

---

## Dependencias

- Ninguna dependencia previa.

---

## Review (GATE — Agente Distinto, P2-01)

- **Revisor:** `vanta-audit` (WASM / Web Platform Reviewer)
- **Enfoque:** Confirmar compatibilidad con navegadores modernos y tratamiento de promesas rechazadas.
- **Cómo se probó:** Verificación de tipos en `wasm-bindgen`.
- **Checklist Anti-Hábitos Tóxicos:**
  - [x] Cero `.unwrap()` en código Rust.
  - [x] Manejo tolerante de errores JS.
- **Veredicto:** ⏳ PENDING ejecución

---

## Notas

La función `delete_file_unlocked` ya es `async fn`, por lo que añadir `.await` no requiere alterar las firmas del trait de almacenamiento.
