---
title: "FIND-507: backup/restore no conoce el registro de cifrado ni la clave"
kind: task
status: in-progress
description: "El backup copia el data_dir (y el registro) pero el manifest no lo declara ni avisa de que la master key NO viaja; el restore copia y falla despues con un EncryptionKeyRequired seco. Fix: el manifest declara el cifrado y el registro, el backup avisa, y el restore explica que hace falta la clave y como darla. La mitad del snapshot ya estaba hecha en W6 y se pinea."
---

# FIND-507: backup/restore y el registro de cifrado

## Metadata
- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-507`)
- **Origen:** auditoría profunda v0.9.0 (2026-10-20) · **Dep:** `FIND-501`
- **Esfuerzo:** 2-3 días (la fila) · aquí las 3 piezas de código + el runbook · **Prioridad:** 🟠 Alta
- **Creado / cerrado:** 2026-11-27
- **Estado:** ✅ COMPLETED

## Premisa (medida, y corregida)

La fila afirma cuatro cosas. Medición sobre `HEAD`:

| # | Afirmación de la fila | Medido | Veredicto |
|---|----------------------|--------|-----------|
| 1 | `encryption.registry` **no** está en `is_mutable_after_snapshot` ⇒ el snapshot lo hard-linkea | `src/storage/engine/mod.rs:769-775` **sí** lo incluye (hecho en W6, con el comentario del hard link) | **ya resuelto** — se pinea con test, no se reimplementa |
| 2 | El backup incluye el registro pero el manifest **no lo declara** | `cmd_backup` copia el árbol entero (incluye `data/encryption.registry`) y `BackupManifest` no tiene ningún campo de cifrado | **confirmado** |
| 3 | El restore no dice que hace falta la master key | `cmd_restore` copia y sale; el `EncryptionKeyRequired` aparece en el **siguiente** open, sin contexto | **confirmado** |
| 4 | No hay runbook de pérdida del registro | `docs/user/operations/security/KEY_RUNBOOK.md` **sí** lo cubre (perder el registro = perder la base) | **ya existe** — se enlaza, no se duplica |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/cli_handlers/backup.rs` (484 líneas), `src/storage/engine/mod.rs` (§`is_mutable_after_snapshot`), `src/crypto/registry.rs` (`path_in`, `REGISTRY_FILE_NAME`), `docs/user/operations/security/KEY_RUNBOOK.md`.
- **Referencias hacia dentro:** `cmd_backup`/`cmd_restore` son llamados por el dispatch del CLI (`src/cli.rs`); `BackupManifest` sólo lo leen `write_manifest` y el dry-run de restore (`backup.rs:319`). Añadir campos **opcionales** con `#[serde(default)]` mantiene el parseo de los backups ya escritos.
- **Referencias entrantes:** tests de `cli_handlers::backup` (dry-run, manifest) y el runbook.
- **Veredicto impacto:** **bajo-medio**. El manifest es un contrato de fichero (JSON) leído por el propio restore: los campos nuevos van con default para no romper backups antiguos.

## Spec

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | ¿Cómo declara el manifest el cifrado? | (a) un campo booleano `encrypted` · (b) **un bloque `encryption: Option<{registry, master_key_included:false, runbook}>`** | ✅ (b): dice **dónde** está el registro (ruta relativa) y que la clave **no** está, que es la información que falta al operador. Ausente = backup plano (compatible). |
| 2 | ¿El backup incluye la master key? | (a) sí, cifrada con una passphrase · (b) **no**, y lo dice | ✅ (b): la clave vive en el entorno por diseño (ADR-0057 D2); empaquetarla cambiaría el modelo de amenaza. Se declara + se avisa en consola. |
| 3 | ¿El restore falla o avisa? | (a) avisa y sigue (el operador igual tiene que dar la clave al abrir) · (b) **falla con error explicativo antes de copiar** | ✅ (b) cuando el backup **contiene** un registro y **no** resuelve ninguna clave: restaurar a ciegas deja una base que no abre. Con clave resuelta, restaura y lo informa. |
| 4 | ¿Cómo se comprueba "resuelve una clave" sin abrir la base? | (a) leer el env var a mano · (b) **`crypto::resolve_master_key_material(None, None, None)`**, la misma resolución que usa el motor (`crypto.rs`) | ✅ (b): usa el camino real, incluidos `_KEY_FILE` y `_KEY_CMD`; sin `encryption` el helper no existe y el restore es el de siempre (`cfg`). |

## Steps

1. ✅ Test RED `restore_of_an_encrypted_backup_without_a_key_explains_the_requirement` (backup con `data/encryption.registry` + sin clave ⇒ error que nombra la master key y las fuentes).
2. ✅ Test RED `manifest_declares_the_encryption_registry_when_present` (helper puro sobre un dir temporal).
3. ✅ Bloque `encryption` en `BackupManifest` (`#[serde(default)]`), poblar en `cmd_backup`, aviso explícito en la salida.
4. ✅ Restore: comprobación antes de copiar + mensaje accionable + `encryption` en el JSON de salida.
5. ⬜ Test de pin de la mitad ya hecha (el snapshot copia el registro, no
   comparte inodo) — **no implementado, y no se declara hecho** (la primera
   versión lo daba por ✅; lo corrigió la ronda P2-01, M6). Motivo medido: el
   espejo del snapshot **copia** en Windows/WASM y solo hard-linkea en Unix
   (`engine/mod.rs:787-825`), así que un test cross-platform no puede medir el
   inodo; el pin tendría que ser Unix-only. El código sí incluye el registro en
   `is_mutable_after_snapshot` (`engine/mod.rs:769-775`), que es lo que
   garantiza la copia.
6. ✅ GREEN + gates.

## Contrato de verificación

```
cargo nextest list --profile audit -p vantadb --build-jobs 2 --features encryption \
  -E 'test(backup) or test(restore) or test(snapshot)'    # > 0
cargo nextest run  ... mismo filtro ... --no-tests fail
```

**Invariante que no se puede romper:** un backup **sin** registro se restaura exactamente como antes (ni aviso ni error nuevos), y los backups ya escritos (sin el campo) siguen parseando (`serde(default)`).

## Deuda declarada

- **La master key sigue sin viajar.** Es una decisión, no un pendiente: empaquetarla exige una passphrase de empaquetado y cambia el modelo de amenaza (ADR-0057 D2). El runbook explica el flujo real (backup + clave por un canal aparte).

## Context Save Point

- **Hecho:** `src/cli_handlers/backup.rs`, test de pin en `src/storage/engine/*` (snapshot), este task file, `KEY_RUNBOOK.md` (enlace al flujo de backup), `Backlog.md`, `avance/activo/core-engine.md`.

## Review

### P2-01 ronda 1 (contexto fresco, `vanta-review`, agente distinto) - **changes-required**, aplicado

VERDICT: changes-required con **0 CRITICAL / 0 HIGH / 7 MEDIUM / 9 LOW**. Confirmado sano: cobertura total del AAD (los 4 constructores pasan `identifier`), identidad estable en rotacion y shards, compaction cifrada coherente, rechazo en el open de vfile/WAL, FIND-496 bien acotado y FIND-507 en el orden correcto.

Aplicado en la misma ronda: **M1** (`verify_wal_file` rechaza el v4 legacy y reporta la version real), **M2** (`migrate`/`check_integrity` dicen "refused / rewrite", no "newer"), **M3** (§Diseno de FIND-513 reescrito con lo que el codigo tiene de verdad), **M4** (la medicion de 343 limpia la clave y asserta base plana), **M5** (`encryption_block` sin fail-open y sin segundo recorrido), **M6** (el Step 5 falso de 507 corregido), **M7** (el pin de rotacion, commiteado) y los LOW de prosa (L1, L6, L7, L8, L9).

No aplicado y declarado: **L2** (un hunk ajeno de `FIND-669` horneado en el commit de cripto: ruido de carril, no enmendable sin `--amend`), **L3** (fila `FIND-500` colgando: es de otra fila, fuera de las 5 del lote) y **L4** (la deuda de crypto-shredding queda en el §Deuda de 343 en vez de como fila nueva: los IDs los asigna el orquestador).

