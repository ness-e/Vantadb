# FIND-587 — Lock `.vanta.lock`, fsync del `.bak` y retiro de `--new-key <hex>` en `encryption rotate` (absorbe FIND-589)

- **Plan:** docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md (Task 10)
- **Prioridad:** 🔴 P0 · **Esfuerzo:** 🟡 4h · **Wave:** 1 (tras FIND-585 `a59e7927e`, convive con FIND-508)
- **Estado:** ⏳ IN PROGRESS
- **SDP:** security-and-hardening, test-driven-development, systematic-debugging, documentation-skill

## Spec

Tres defectos en `vanta-cli encryption rotate` (`src/cli_handlers/encryption.rs`):

1. **Carrera destructiva:** rota `encryption.registry` sin adquirir `.vanta.lock` — un proceso vivo usando la base puede reescribir/leer el registry durante la rotación.
2. **Durabilidad:** el `.bak` que escribe `KeyRegistry::rotate` (vía `std::fs::copy`) no recibe `sync_all()` ni fsync del directorio padre.
3. **Fuga de KEK:** `--new-key <hex>` expone la clave maestra nueva en `ps`, `/proc/<pid>/cmdline` y el historial del shell.

## Decisiones de diseño (cerradas por el owner)

- **`--new-key` se retira por completo** (sin deprecación): `encryption rotate` no existe en `v0.9.0` (`git show v0.9.0:src/cli.rs` no contiene `new_key`) — no hay usuario publicado al que romper.
- **Fuentes de la nueva clave:** `--new-key-file <path>` (existente) y `--new-key-stdin` (nuevo: lee una línea de stdin y le quita el salto de línea final). Exactamente una obligatoria: `ArgGroup` clap `required = true`, `multiple = false`. Sin variable de entorno nueva.
- **Zeroize:** todo buffer con material de clave (texto leído de fichero/stdin, hex decodificado, clave actual) vive en `crate::crypto::Zeroizing` (tipo existente, sin dependencias nuevas). El buffer hex se lee como `Vec<u8>` (`std::fs::read` / `read_until(b'\n')`) para poder envolverlo — el trait `Zeroize` local no tiene impl para `String`.
- **Lock:** `.vanta.lock` en exclusivo al inicio del handler (tras el check de existencia del registry, que no toca disco), misma semántica del motor (`src/storage/engine/init.rs:261-356`): `fs2::FileExt::try_lock_exclusive` bajo `#[cfg(feature = "fs2")]` (no-op sin fs2, igual que el motor), `Error::DatabaseBusy` con `holder_detail` de `read_lock_metadata`, y `write_lock_metadata` al adquirir. Lock tomado → fallo inmediato sin tocar registry ni `.bak`. El File vive hasta después del último fsync (drop al salir). **Sin auto-deadlock:** el handler no llama a `StorageEngine::open` — solo `KeyRegistry::open_or_create` (ruta de escritura, NO `open_existing` de FIND-508).
- **Durabilidad del `.bak`:** tras `registry.rotate()`, abrir `encryption.registry.bak`, `file.sync_all()`, después `crate::utils::fs::sync_parent_dir(&bak_path)` (helper existente, no-op deliberado en Windows — resuelve el riesgo 1 del pre-mortem). El registry nuevo ya es tmp→fsync→rename→dir-fsync (`registry.rs:777-826`, ADR-0057 R-8) — no se toca (`registry.rs` PROHIBIDO).

## Contrato

```
cargo nextest run --profile audit -p vantadb --features encryption -E "test(rotate) or test(encryption)" --locked --build-jobs 2
```

La línea base negativa son los tests nuevos: fallan contra el código actual (RED), pasan tras el cambio (GREEN).

## DoD

- [ ] Test (a) concurrencia: `.vanta.lock` tomado en exclusivo (engine abierto) → `rotate` falla con `DatabaseBusy`, `encryption.registry` byte a byte idéntico, sin `.bak` nuevo.
- [ ] Test (b) CLI: `--new-key <hex>` rechazado por clap; `--new-key-file` y `--new-key-stdin` aceptados; ambos o ninguno → error.
- [ ] Test (c) rotación feliz con `--new-key-file`: la base abre con la clave nueva, no con la vieja, y existe el `.bak`.
- [ ] Contrato verde + `cargo clippy -p vantadb --features encryption --all-targets --locked -- -D warnings` + `cargo fmt --check`.
- [ ] Docs: `CONFIGURATION.md:503` (comando inexistente `vanta-cli key rotate` → real) y `KEY_RUNBOOK.md:76-77` (`--new-key` → `--new-key-file`/`--new-key-stdin`).
- [ ] Review vanta-audit (zeroize, sin auto-deadlock, registry nunca a medio rotar) + commit local.

## Impacto mapeado (Regla 0)

- `src/cli_handlers/encryption.rs` (leído completo): `cmd_encryption_rotate` `:27` (feature) y `:125` (sin feature). No llama a `StorageEngine::open` → sin auto-deadlock. Firma cambia a `(db_path, new_key_file: Option<&str>, new_key_stdin: bool, json_output)`.
- `src/cli.rs` (leído): `EncryptionCommand::Rotate` `:545-554`; tests `:624-664` migran a los flags nuevos.
- `src/bin/vanta-cli.rs:326-336`: match arm del subcomando — actualiza args.
- `src/storage/engine/init.rs:261-356` (leído, NO se toca): semántica de lock a reutilizar — `OpenOptions read+write+create`, `try_lock_exclusive`, `DatabaseBusy` con holder_detail, `write_lock_metadata`.
- `src/crypto/registry.rs` (leído, PROHIBIDO): `rotate` `:706` copia `.bak` con `std::fs::copy` (sin fsync → el handler fsynca después); `persist` `:777` ya es tmp→fsync→rename→dir-fsync.
- `src/utils/fs.rs` (leído): `sync_parent_dir` `pub(crate)`, no-op en Windows.
- `src/crypto.rs` (leído): `Zeroizing<T: Zeroize>` `:190` (solo Deref, sin DerefMut → leer en buffer plano y envolver); `Zeroize` impls: `[u8]`, `[u8;32]`, `Vec<u8>` (no `String`); `decode_hex` `:1126` (`pub(crate)`, hace `trim` interno).
- `src/storage/lock.rs` (leído): `LockMetadata::current`, `read_lock_metadata`, `write_lock_metadata` — reutilizables.
- Entrantes a `cmd_encryption_rotate`: único caller `src/bin/vanta-cli.rs:330`. Tests: `src/storage/engine/tests/encryption.rs` (helpers `key_hex`, `keyed_config` en `mod tests` de engine — los tests nuevos del handler viven inline en `cli_handlers/encryption.rs` con helpers propios).
- Docs: `CONFIGURATION.md:503`, `KEY_RUNBOOK.md:76-77`. NOTICED pero fuera de scope: `ADR-0057:221` menciona `--new-key` (ADR histórico, no se reescribe) → candidato FIND.

## Steps

- [x] DISCOVERY + task file
- [x] RED: tests (a)(b)(c) fallan contra código actual (compile-error contra firma vieja + `--new-key-stdin` no existe — log `targetfind587-red`)
- [x] GREEN: `cli.rs` (ArgGroup + flags), `encryption.rs` (lock + stdin + zeroize + fsync `.bak`), `vanta-cli.rs` (match arm) — 76/76 ✅
- [ ] VERIFY: contrato ✅ (76/76) + clippy (en curso) + fmt ✅
- [x] Docs: `CONFIGURATION.md`, `KEY_RUNBOOK.md` + `check-links.mjs` ✅
- [ ] Review vanta-audit + commit local
