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
