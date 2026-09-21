# Plan: WASM Quick Wins (H-01/H-05/H-07/H-08/H-16)

> **Origen:** `/research vantadb-wasm` → `docs/reviews/research-vantadb-wasm-20260825.md`
> **Decisiones HITL 2026-08-25:** 5 quick wins APLICAR aprobados (<1 día c/u).
> Listo para `/pipeline run docs/plans/2026-08-25-wasm-quickwins.md`.
> Resto de hallazgos → P41 `WSM-01..14` en `docs/Backlog.md`.

## Wave 1 — Correcciones de datos/persistencia

### QW-1 (H-01): Fix `OpfsFile::append` sobreescribe desde offset 0
- **Contrato verificable:** `append(data)` escribe al final del archivo existente
  (calcular posición con tamaño actual antes de write, como ya hace el bridge JS).
  Test wasm: append ×2 → contenido = concat de ambos.
- **Archivos:** `vantadb-wasm/src/opfs.rs:85-97`
- **Ref:** divergencia con `opfs_bridge.js:53-57` (implementación JS correcta).

### QW-2 (H-08): `next_cursor` u64→string
- **Contrato verificable:** cursor viaja como string decimal (política string-u64 del
  proyecto); roundtrip >2^53 testeado.
- **Archivos:** `vantadb-wasm/src/lib.rs:943`

## Wave 2 — Semántica honesta

### QW-3 (H-05): `flush()` deja de engañar
- **Contrato verificable:** o delega a `save()` (persistencia real) o renombra/documenta
  "no-op durabilidad: llamar save()". Decisión mínima: docstring + warning console si no
  hay backend persistente activo. Test actualiza expectativa.
- **Archivos:** `vantadb-wasm/src/lib.rs` (flush)

### QW-4 (H-07): CRC inválido → error explícito
- **Contrato verificable:** `read_file` con footer CRC corrupto devuelve error
  "storage corrupted" (no datos crudos que explotan en serde_json). Opt-out legacy
  flagueado si hace falta para migración. Test existente de backward-compat se ajusta.
- **Archivos:** `vantadb-wasm/src/opfs.rs:207`

## Wave 3 — Worker proxy

### QW-5 (H-16): MessagePorts cerrados + retry sin matcheo de strings
- **Contrato verificable:** cada request cierra sus ports (`port1.close()`/`port2.close()`);
  retry usa código/tipo estructurado del error, no substring matching. Tests worker
  existentes siguen pasando.
- **Archivos:** `vantadb-wasm/src/worker.rs`

## Verificación por tarea

```
cargo check -p vantadb-wasm --target wasm32-unknown-unknown
wasm-pack test --chrome --headless   # o cargo nextest workspace si aplica
dev-tools/verify_changed.ps1         # pre-commit
```

## Fuera de alcance de este plan

Todo lo demás → P41 (`WSM-01..14`) · wontfix H-23 · estrategias H-21/H-22 (memoria+ADR).

=== RECITATION WASM-QW2 ===
Campaign ID: a1f1d889-1d97-460c-9ca8-cf8ca3876cc9
Objetivo activo: Verificar WASM-QW2 next_cursor string-u64 ya resuelto en 53f080e5
Estado: completed
Última acción: Task verificado en sesion previa: fix ya en commit 53f080e5, cargo check wasm paso, tests >2^53 presentes. Cerrando para desbloquear WASM-QW1 (one-task-at-a-time).
Resultado: OK
Próxima acción: Proceder con WASM-QW1
Contrato: verificacion: cargo check -p vantadb-wasm OK + next_cursor_to_js string + deserialize_cursor accept string/number + test >2^53 presente | evidencia: vantadb-wasm/src/lib.rs:159-164,135-155,1769-1828 confianza:alta | artefactos: vantadb-wasm/src/lib.rs | invariantes: politica string-u64, no hot path | deuda: ninguna | queda_pendiente: ningunax
Próxima tarea si completa: WASM-QW1
=== END RECITATION ===

=== RECITATION WASM-QW1 ===
Campaign ID: 30e7e1dc-6f93-4681-88a7-5bca016dad1a
Objetivo activo: Verificar Fix OpfsFile::append offset 0 → size+position paridad JS, tests append×2=concat, cargo check wasm
Estado: completed
Última acción: Steps 1-3 verificados: Step1 invariantes size+position 4/4 OK + cargo check wasm 2.74s, Step2 5 tests append (4 opfs +1 worker) OK + fmt OK, Step3 verify full sin edicion (ponytail: no tocar lo ya correcto). Task file WASM-QW1.md actualizado a COMPLETED con Context Save Point.
Resultado: OK
Próxima acción: Ninguno — WASM-QW1 COMPLETED, NO commit (lead commitea). Siguiente: WASM-QW3 si el pipeline lo despacha.
Contrato: verificacion: cargo check -p vantadb-wasm --target wasm32-unknown-unknown ✅ (2.74s) + cargo fmt --check ✅ + grep invariantes 4/4 ✅ + 5 tests wasm ✅ | evidencia: opfs.rs:91-102 getFile size + keepExistingData true + position=size (commit 53f080e5), opfs_bridge.js:53-57 idem, wasm_tests.rs:304 test_opfs_append_concatenates_raw assert b"hello world!" confianza:alta | artefactos: vantadb-wasm/src/opfs.rs:90-108, vantadb-wasm/tests/wasm_tests.rs:229-330,1201 | invariantes: paridad JS bridge, no romper CRC footer, no tocar core storage | deuda: ninguna — clippy global falla por debt pre-existente vfile_mmap.rs:140/file.rs:143 fuera de blast radius (no bloquea WASM-QW1) | queda_pendiente: ninguno
Próxima tarea si completa: WASM-QW3
=== END RECITATION ===
