# RES-09 — Filas P24 del roadmap huérfano (docs)

**Plan:** `docs/plans/2026-09-03-quality-gtm-wave.md` Task 9 · **Ruta:** vanta-docs · **Tipo:** docs
**SDP:** base(campaign-executor,progreso,ponytail)+writing-guidelines+documentation-and-adrs (keywords: backlog, roadmap, FUT, documentación)

## Impacto mapeado (Regla 0)

- **Leídos completos (zonas de edición):** `docs/Backlog.md` resumen P24/P38 (l.41,48), sección P24 (l.331-348), sección P38 (l.513-549); fuente `docs/research/archive/investigacion-equipo-2026-08-09.md` §roadmap (l.183-188) + l.93 (DiskANN in-memory); código verificado hoy: `src/index/diskann.rs:7,13` ("purely in-memory, not disk-backed"), `src/wal.rs:372` ("Default threshold for Periodic… 1 (sync every write)"), `src/config.rs:180` (default Periodic), `src/wal.rs:224-228` (plumbing sync_mode existe, falta group-commit/flush desacoplado).
- **Referencias hacia dentro:** Task 9 del plan; fila P38 `RES-09` (a eliminar). Ninguna otra referencia a FUT-12/13/14 (colisión descartada: `rg "FUT-1[2-9]"` = 0 hits; FUT llega a -11).
- **Referencias salientes:** filas nuevas citan `wal.rs`/`planner.rs`/`diskann.rs` y la investigación archivada — solo texto, sin enlaces rotos (ruta real: `docs/research/archive/...`).
- **Veredicto:** docs-only, 1 archivo backlog + 1 registro avance + plan. Sin impacto en código/contratos. Count P38 "17" es criterio de auditoría 2026-08-25 (IDs definidos, no filas vivas=7): se decrementa a 16 fiel al delete, no se re-audita (fuera de scope).

## Steps

- [x] 1. Verificar colisión IDs (`rg "FUT-1[2-9]" docs/Backlog.md` → 0) y re-leer fuente §roadmap
- [x] 2. Verificar evidencia código: diskann.rs:7, wal.rs:372/config.rs:180 (gap fsync, no async-ingest genérico)
- [x] 3. Agregar FUT-12/13/14 en tabla P24 + nota origen RES-09; actualizar count resumen P24 (10→13)
- [x] 4. Eliminar fila RES-09 de P38; decrementar count resumen P38 (17→16)
- [x] 5. Registro en `docs/avance/investigaciones.md` (Trigger 1)
- [x] 6. Contrato: `rg -n "FUT-1[234]"` ≥3 · `rg -c "RES-09"` ≥1 · markdownlint-cli2 (existe `.markdownlint-cli2.yaml`) · tabla 5 columnas
- [x] 7. Plan Task 9 ✅ + recitation; commit `docs(backlog): FUT-12/13/14 roadmap huérfano (RES-09)` (NO stagear .opencode/completions/stash)

## Notas

- Descripción FUT-13 fieente: research §roadmap:186 dice "Query planner con optimizaciones reales (hoy router + heurística)" (l.37: clasifica Hybrid/TextOnly/VectorOnly) — no inventar join-reordering más allá de la fuente.
