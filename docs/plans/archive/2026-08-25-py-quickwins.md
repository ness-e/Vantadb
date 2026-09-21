# Plan: Quick wins Python SDK (INV-vantadb-python-01)

> Origen: `/research vantadb-python` 2026-08-25 → informe
> `docs/reviews/research-vantadb-python-20260825.md`. Hallazgos APLICAR/MEJORAR 🟢
> aprobados por el usuario (Fase D): H-01, H-02, H-03, H-05, H-07.
> Ejecutar con `/pipeline run docs/plans/2026-08-25-py-quickwins.md`.

| Wave | Tarea | Hallazgo | Contrato verificable | Archivos |
|------|-------|----------|---------------------|----------|
| 1 | PY-QW1: README 100% inglés (residuos ES) | H-01 | `rg -n "[áéíóúñ]" vantadb-python/README.md` vacío; quickstart sigue funcional | `vantadb-python/README.md:37,52,63` |
| 1 | PY-QW2: eliminar dual API de `put_batch` (P2-5) — solo kwargs/dict entries, deprecar tuplas legacy | H-02 | `put_batch` sin branching de tuplas (~53 líneas menos); tests existentes pasan ajustados; entrada P2-5 marcada resuelta en AGENTS.md tabla P2 | `vantadb-python/src/lib.rs` (flat `put_batch`) |
| 1 | PY-QW3: declarar Python 3.14 | H-03 | classifiers incluyen `3.14`; `requires-python` revisado; build abi3 intacto | `vantadb-python/pyproject.toml:22-26` |
| 2 | PY-QW4: higiene de artefactos locales del módulo | H-05 | `.gitignore` (raíz o módulo) cubre `*.pyd`, `*.pdb`, `dist/`, `probe_lock_db/`, `.coverage`; `git status` limpio tras `maturin develop` + pytest local | `.gitignore`, `vantadb-python/pyproject.toml` |
| 2 | PY-QW5: README lidera diferenciación vs chromadb | H-07 | Primeras 10 líneas mencionan híbrido RRF + grafo + TTL/supersede + migradores; sin claims numéricos sin fuente (Regla 11) | `vantadb-python/README.md:1-30` |

**Fuera de alcance de este plan:** H-04 y H-06 (→ Backlog PY-01/PY-02),
H-08 (wontfix), H-09 (decisión estratégica registrada en memoria).
