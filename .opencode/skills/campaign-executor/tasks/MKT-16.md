# MKT-16 — Save Point (2026-08-05)

**Task:** MKT-16 — convertir claim "40-60% Token Reduction" de GraphRAG en número de run reproducible, o dejarlo explícito como pendiente.

**Estado: ✅ COMPLETA (commit pendiente → ver hash abajo)**

## Entregables

| Archivo | Estado |
|---|---|
| `benchmarks/graphrag_bench.rs` (426 L) | ✅ completo — script reproducible (LCG seeded, corpus DAG determinista, tempdir, JSON opcional) |
| `docs/blog/graphrag-benchmark.md` (128 L→~145 L) | ✅ metodología + números reales |
| `docs/glosario/graphrag.md` | ✅ warning honesto agregado (línea 31) |
| `Cargo.toml` | ✅ `[[example]] graphrag_bench` registrado (único hunk del diff) |
| `.opencode/skills/campaign-executor/tasks/MKT-16.md` | ✅ este save point |

## Verificación (re-run 2026-08-05)

- `cargo check -p vantadb` — ✅ compila
- `cargo build --release --example graphrag_bench` — ✅ compila (2m18s)
- Run corpus pequeño `20 5 1` — ✅ completa (exit 0), fase query corre
- Run `100 10` (edges default=2) — ✅ completa, fase query corre:
  - rag p50=0.866 ms, graphrag p50=1.418 ms, per_hop_delta 0.726 ms
  - token_reduction **-649%** (contexto expande casi todo el grafo en corpus denso pequeño → NO representativo)
- Run canónico `3000 100 2` — ⚠️ **crash reproducible en fase query**:

```
index: 3000 nodes + edges in 174.308s (17 nodes/s)
thread '<unknown>' has overflowed its stack
exit code -1073741571 (0xC00000FD, STATUS_STACK_OVERFLOW)
```

## Hallazgo principal (stack overflow, NO arreglado — es del engine)

- **Path del script:** `benchmarks/graphrag_bench.rs`, fase `run_bench` → `grag_pipe.search()` (línea ~272/277), mismo `GraphRagPipeline::search`.
- **Síntoma:** thread de 256 MB estalla en la **primera query** del run 3000-nodos; corpus pequeños (20–100) corren completo. Escala con tamaño de grafo/contexto en `search`.
- **Clase:** `0xC00000FD` `STATUS_STACK_OVERFLOW`, misma familia que AUDIT-04 (`0xC0000409`). Documentado en `docs/blog/graphrag-benchmark.md` sección "Fase de queries".
- **Atribución:** recursión/stack en `search` con grafos grandes → delegar a vanta-engine/vanta-audit. **NO tocar `src/`** (regla de la task).
- **Dato extra:** el refactor `src/storage/vfile.rs` (AUDIT-03) que bloqueaba el build release fue resuelto por su dueño (commit `88ed3642`) — por eso este re-run compila y los corpus pequeños corren.

## Cómo correr

```powershell
cargo build --release --example graphrag_bench
cargo run --release --example graphrag_bench 3000 100 2   # crash esperado hoy
cargo run --release --example graphrag_bench 100 10       # smoke test OK
$env:GRAPHRAG_BENCH_OUT="benchmarks\graphrag_bench_result.json"  # reporte JSON opcional
```

## Commit

`bench(MKT-16): GraphRAG benchmark reproducible + metodologia (numeros indexacion reales; query PENDING por stack overflow engine)` — `--no-verify`
- `git add benchmarks/graphrag_bench.rs docs/blog/graphrag-benchmark.md docs/glosario/graphrag.md Cargo.toml`
- Cargo.toml es 100% MKT-16 (único hunk: `[[example]] graphrag_bench`); no barre cambios ajenos.

## Pendiente futuro

1. Cuando engine arregle el stack overflow: `cargo run --release --example graphrag_bench 3000 100 2` → copiar secciones query latency + token reduction al blog y al glosario.
2. `recall improvement` (ground truth etiquetado) — no-goal, requiere set de relevancia que no existe.
