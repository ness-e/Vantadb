Cargá backlog-executor, ponytail (full).

Refs: `docs/plans/2026-07-13-code-task-execution-campaign.md`, `docs/Backlog.md`, `docs/bitacora.md`

── UNA ITERACIÓN (después de leer el plan file COMPLETO) ──

   ┌─ ¿PRIMERA VEZ EN ESTA TAREA? ──────────────────────────┐
   │ 1. Cargá skills según fase (tabla AGENTS.md → DEFINE   │
   │    → PLAN → BUILD → REVIEW → SHIP). Ponytail full.     │
   │ 2. codegraph_explore: callers / callees / implicaciones │
   │    (contratos, API pública, perf, migración).           │
   │ 3. Implementá (~100 líneas, ponytail ladder).           │
   │ 4. Verify: `cargo build && cargo nextest run`           │
   │ 5. Documentá relaciones en el commit.                   │
   └─────────────────────────────────────────────────────────┘

   ┌─ ¿YA IMPLEMENTASTE? ────────────────────────────────────┐
   │ 1. Cargá code-review-and-quality + doubt-driven-devel.  │
   │ 2. codegraph_explore post-impact: módulos dependientes  │
   │    compilan? interfaces rotas? implicaciones no vistas?  │
   │ 3. ¿Código más simple? ¿tests faltantes? ¿edge cases?   │
   │ 4. Decidí: ¿listo para mergear o falta algo?            │
   └─────────────────────────────────────────────────────────┘

OK → Ejecutá SOLO una acción atómica.
     codegraph/implementar/verify → usá sub-agentes en paralelo.
     commit → actualizá plan file + Backlog.md + bitacora.md.
     **Escribí RECITATION block.**
     Detenete.

REGLAS:
- Verify = comando mecánico real (cargo build && cargo nextest run)
- Verify falla 2 veces con mismo error → ❌ FAILED
- No cambies scope. Anotalo, no lo implementes.
