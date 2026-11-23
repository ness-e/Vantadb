---
id: FIND-246
title: "Forzar file.sync_all() en persistencia HNSW antes de avanzar checkpoint_seq en Fjall (DUR-01)"
status: completed
type: fix
plan: docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md (Task 16)
---

# FIND-246: Forzar `file.sync_all()` en persistencia HNSW antes de avanzar `checkpoint_seq` en Fjall

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md`
- **Fuente:** `docs/dev/Backlog.md` (L242 `DUR-01` G2) / Plan Task 16
- **Esfuerzo:** 🟡 4h
- **Prioridad:** 🔴
- **Tipo:** Rust (`src/utils/fs.rs`, `src/index/serialize/file.rs`, `src/index/port_impl.rs`, `src/storage/engine/maintenance.rs`, `src/storage/engine/init.rs`)
- **Turns estimados:** 20
- **Creado:** 2026-11-16T10:00
- **last-synced:** 2026-11-18
- **Estado:** ✅ COMPLETED
- **Commits:** `aa261e836` (`fix(durability): fsync HNSW snapshot atomically before advancing checkpoint_seq`), `0b082c35f` (guarda `!data_dir.is_empty()` en `rebuild_vector_index`)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `StorageEngine::flush`, `StorageEngine::compact_wal`, `StorageEngine::rebuild_vector_index`, `CPIndex::save_to_path` |
| Callees | `src/utils/fs.rs` (`write_atomic`), `std::fs::File::sync_all`, `std::fs::rename` |
| Implicaciones | Cierra ventana de corrupción/pérdida de vectores ante corte eléctrico entre el guardado de `vector_index.bin` y el avance de `checkpoint_seq` en Fjall. |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `src/utils/fs.rs`, `src/index/serialize/file.rs`, `src/index/port_impl.rs`, `src/storage/engine/maintenance.rs`, `src/storage/engine/init.rs`, `tests/durability_recovery.rs`.
- **Archivos referenciados hacia dentro:** `crate::utils::fs::write_atomic`, `Backend::put` (`checkpoint_seq`).
- **Archivos que referencian a los editados:** `src/storage/engine/mod.rs`, `tests/durability_recovery.rs`, `tests/index_reconstruction.rs`.
- **Veredicto impacto:** medio (modifica ruta de checkpoint/persistencia del índice vectorial; sin cambio en formato binario).

## Contrato
`cargo nextest run --profile audit -p vantadb --features failpoints -E "test(checkpoint) or test(persist) or test(index_reconstruction)" --locked --build-jobs 2`

## Fase 1 — Evidencia de Debugging (GATE — tipo Bug)
- **Repro:** `src/index/serialize/file.rs:51-65` truncaba el archivo destino en sitio y solo llamaba `writer.flush()` (page cache del SO) sin `sync_all()` ni escritura atómica `.tmp` → `rename`, mientras `maintenance.rs:88` persistía `checkpoint_seq` inmediatamente.
- **Hipótesis:** Unificar las 3 rutas de escritura de `vector_index.bin` (plaintext, mmap, cifrado) mediante `utils::fs::write_atomic` (`.tmp` → `flush` → `sync_all` → `drop` handle → `rename` → `#[cfg(unix)] sync_parent_dir`) y exigir `save_vector_index()?` antes de grabar `checkpoint_seq` elimina tanto imágenes desgarradas como el adelanto de `checkpoint_seq`.
- **1 variable controlada:** Reemplazo de escritura directa por `write_atomic` + orden estricto antes de `checkpoint_seq`.
- **Test RED:** Con failpoint o error de E/S en la persistencia del índice, `checkpoint_seq` no debe avanzar y el archivo `.tmp` huérfano se limpia en aperturas writable (`!read_only`).

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** `checkpoint_seq` jamás avanza en el backend KV si `save_vector_index()` devuelve `Err`; en Windows el handle del `.tmp` se cierra (`drop`) antes de `std::fs::rename`; en aperturas `read_only` (`init.rs:480-485`) nunca se borra `vector_index.bin.tmp` (cumple `ADR-0001`); en motores in-memory (`data_dir.is_empty()`, `maintenance.rs:543`) `rebuild_vector_index` no escribe `vector_index.bin` en el CWD (`0b082c35f`).
- **Comandos de verificación:** `cargo nextest run --profile audit -p vantadb --features failpoints --test durability_recovery --locked --build-jobs 2`
- **Deuda pendiente:** Nit preexistente no bloqueante anotado en auditoría (#20): `save_vector_index` (`maintenance.rs:206`) comprueba `self.wal.is_none()` en lugar de `data_dir.is_empty()`.

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** ≤ 0 (centraliza escritura atómica en `src/utils/fs.rs` eliminando duplicación en 3 rutas).

## Definition of Done (contrato multi-nivel — P2-08)
| Nivel | Gate |
|-------|------|
| **Task** | Contrato mecánico + tests de integración plaintext/mmap/cifrado y unitario `checkpoint_seq` en `tests/durability_recovery.rs:450-560` |
| **Commit** | `aa261e836` + fix guarda efímera `0b082c35f` |
| **Release** | Garantía `DUR-01` verificada para `v0.10.0` |

## Herramientas necesarias
- cargo-mcp / nextest

**Skills cargadas (SDP):** `security-and-hardening`, `systematic-debugging`, `test-driven-development`.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — Previene corrupción de índice y pérdida silenciosa de entradas WAL tras crash/apagón; respeta barrera `read_only` al limpiar `.tmp`.
- [x] **PERFORMANCE** — `sync_all()` ocurre solo en checkpoints/flushes explícitos, no en el hot path por operación.

## Steps
- [x] Step 1: Implementar `write_atomic` en `src/utils/fs.rs:22-81,97-115` (`.tmp` → `flush` → `sync_all` → `drop` → `rename` → `sync_parent_dir` en Unix).
- [x] Step 2: Cablear `write_atomic` en `src/index/serialize/file.rs:51,60,290` y `src/index/port_impl.rs:173`.
- [x] Step 3: Asegurar orden `save_vector_index()?` antes de `checkpoint_seq` en `src/storage/engine/maintenance.rs:56-100` y guarda `!data_dir.is_empty()` en `:543` (`0b082c35f`).
- [x] Step 4: Limpiar `vector_index.bin.tmp` residual en `src/storage/engine/init.rs:480-485` solo cuando `!read_only`.
- [x] Step 5: Añadir tests de regresión en `tests/durability_recovery.rs:450-560`.

## Dependencias
- Wave 0 completa.

## Review (GATE — agente distinto, P2-01)
- **Revisor:** `vanta-review` (adversarial tier: `src/storage/**`, `src/index/serialize/**`) + re-auditoría `audit-w1-durability` (#20).
- **Enfoque:** Correcto. `write_atomic` cierra la ventana de truncamiento in-place y garantiza durabilidad física en disco antes de persistir `checkpoint_seq`.
- **Cómo se probó:** Tests unitarios y de integración en `tests/durability_recovery.rs:450-560` (plaintext, mmap, cifrado y no-avance de `checkpoint_seq` ante error) + gate pre-push ALL 10 PASS en `0b082c35f`.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria.
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado.
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas.
  - [x] Verificar cobertura SDP (v3).
- **Veredicto:** ✅ approve

## Notas
- El perfil `audit` excluye `durability_recovery` en su `default-filter`, por lo que su verificación debe invocarse con `--test durability_recovery`.
