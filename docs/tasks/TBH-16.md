# TBH-16: Evaluate `divan` 0.1.21 vs criterion 0.8 (DOC-ONLY)

## Metadata
- **Plan file:** `docs/plans/2026-08-30-testing-bench-harden.md`
- **Creado:** 2026-08-31T00:00
- **last-synced:** 2026-08-31T00:00
- **Estado:** ⬜ PENDING
- **Agente:** vanta-lead (Wave final, paralelo)
- **Tipo:** docs-only decision (no code change)

## Blast Radius
Callers | Callees | Implicaciones
- `Cargo.toml` → no touched (decision: no install)
- `benches/**/*.rs` (22 bench files) → no touched (decision: no port)
- `docs/research/bench-framework-evaluation-2026-08-30.md` → NEW (decision record)

## Contrato
- `git grep "divan" Cargo.toml` → 0 hits
- `Test-Path docs/research/bench-framework-evaluation-2026-08-30.md` → True
- Decisión documentada: **NO portar a divan** (D1 conservadora + D5 dep justification)

## Herramientas
- codegraph_explore, grep, glob
- (No cargo, no edit of Cargo.toml — ponytail reflex)

## Steps

### Step 1: Discovery (✅ ya hecho)
- **Acción:** verificar `git grep "divan" Cargo.toml` → 0 hits; listar benches existentes
- **Result:** criterion 0.8 (`features = ["html_reports", "async_tokio"]`) cubre 22 bench files;
  `tokenizer_bench.rs` identificado como candidato natural para port (no se porta).
- **Estado:** ✅ DONE

### Step 2: Escribir decisión
- **Archivo:** `docs/research/bench-framework-evaluation-2026-08-30.md` (NEW)
- **Acción:** Documento tipo mini-ADR (mismo formato que `docs/research/concurrency-testing-2026-08-30.md`)
  con: TL;DR, contexto, criterion hoy, qué ofrece divan, costos, decisión, trigger conditions
- **Verify:** `Test-Path docs/research/bench-framework-evaluation-2026-08-30.md` → True
- **Estado:** ⬜ PENDING

### Step 3: Commit + cierre
- **Acción:** `git add docs/research/bench-framework-evaluation-2026-08-30.md .opencode/skills/campaign-executor/tasks/TBH-16.md`
  + `git commit -m "docs(TBH-16): record divan evaluation decision (not introduced; criterion sufficient per D1+D5)"`
- **Verify:** `git log -1 --oneline` muestra el commit
- **Estado:** ⬜ PENDING

### Step 4: Update state
- **Acción:** `campaign_update_task_state TBH-16 completed`
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna (es la tarea final del plan).
- TBH-11 (nightly bench ya cubre 8 benches con criterion) → precedente exitoso que justifica la decisión.

## Notas
- **Ponytail reflex:** NO instalar. NO portar. Solo doc. (D5: "justificar cada dep antes de añadir")
- **D1 conservadora:** plan explícitamente dice "no `divan` alongside criterion".
- Owner decision ya tomada en plan; esta tarea solo formaliza el razonamiento para auditoría futura.

## Context Save Point
- **Fecha:** 2026-08-31
- **Branch:** main
- **CI pendiente:** no
- **Decisiones:** NO portar a divan porque criterion 0.8 cubre el caso de uso completo
  (html_reports, async_tokio, 22 benches ya migrados, baseline JSON, nightly CI pipeline).
- **Problemas conocidos:** ninguno
- **Próxima tarea:** TBH-17 (loom eval) — ya completado por vanta-lead (documento `concurrency-testing-2026-08-30.md` existe).