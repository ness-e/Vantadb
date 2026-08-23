# Task MCP-29 — Namespaces de memoria como tablas IQL (camino 1 de MCP-27)

**Fuente de verdad:** `docs/Backlog.md` → fase **P25** → fila `MCP-29` + nota de cierre de `MCP-27`.
**Prioridad:** 🟢 (diferido; ejecutar = hay demanda explícita del owner) · **Esfuerzo:** 🔴

## Fase 1 — DISCOVERY
- [ ] Leer fila `MCP-29` + resolución de `MCP-27` en Backlog (root cause: scan filtra `type == <entity>`, records no tienen `type`).
- [ ] codegraph_explore: `memory_record_to_node`, PhysicalScan type filter (`src/executor/`), validación de identificadores del parser.
- [ ] Mapear blast radius: records existentes sin `type`, namespaces con `/`, colisión con tipos de grafo.

## Fase 2 — EJECUCIÓN
- [ ] Decisión ADR-first: registrar trade-off en la fila antes de tocar código (Regla 5 — el autor articula; IA aporta evidencia).
- [ ] Setear `type=<sanitizado>` en `memory_record_to_node`.
- [ ] Migración/backfill de records existentes (o política lazy documentada).
- [ ] Sanitización de namespace → identificador IQL válido (reject o encode para `a/b`).
- [ ] Política de colisión namespace vs tipo de grafo existente.
- [ ] Tests: put → `SELECT * FROM <ns>` visible; namespace con `/`; colisión; migración.

## Fase 3 — VERIFICACIÓN
- [ ] `cargo check -p vantadb && cargo test -p vantadb`
- [ ] `cargo test -p vantadb-mcp`
- [ ] `cargo clippy -p vantadb --all-targets -- -D warnings` (si falla por deuda pre-existente del workspace, limitar a crates tocados)

## Fase 4 — CIERRE
- [ ] SKILL.md ×2 + docs/api (semántica IQL↔memoria) + fila Backlog ✅.

## RESULTADO (obligatorio)
Bloque RESULTADO: ✅ COMPLETO | 🟡 INCOMPLETO | ❌ FALLIDO + evidencia + DECISION_TOMADA.
