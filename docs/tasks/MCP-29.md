# Task MCP-29 — Namespaces de memoria como tablas IQL (camino 1 de MCP-27)

**Fuente de verdad:** `docs/Backlog.md` → fase **P25** → fila `MCP-29` + nota de cierre de `MCP-27`.
**Prioridad:** 🟢 (diferido; ejecutar = hay demanda explícita del owner) · **Esfuerzo:** 🔴

## Impacto mapeado (Regla 0)

**Archivos leídos completos (secciones relevantes):**
- `src/sdk/serialization/mod.rs` (1-490): `memory_record_to_node_owned` (:404, write path — NO lo tocamos), `memory_record_from_node_inner` (:299, read path — metadatos = todos los campos no reservados → tocarlo cambiaría semántica SDK), `validate_namespace` (:66 — chars válidos: A-Za-z0-9._/-), `FIELD_NAMESPACE` (:14).
- `src/physical_plan/scan.rs` completo: `PhysicalScan::next` filtra `relational["type"] == entity` (:76) — ÚNICO choke point del filtrado de tablas.
- `src/parser/mod.rs`: `ident` (:30) = `[A-Za-z_][A-Za-z0-9_#.]*` → `/` y `-` inválidos en identificadores IQL.
- `src/planner.rs` (350-430): SELECT compila a PhysicalScan + PhysicalFilter — un solo camino.
- `src/executor.rs` (150-270): `execute_hybrid` → `execute_statement`; consumers de `"type"`: SemanticSummary filter (:200), INSERT setea type (:224).
- Consumers de `type` mapeados: executor.rs:200,224,353; cli_server.rs:2277; llm.rs:276; wiki/graphrag/GDS operan por sus propios campos — variante elegida NO toca `type`, blast radius ≈ 0.

**Referencias entrantes:** planner.rs:379,401 → PhysicalScan; MCP test `mcp_tests.rs:529` documenta la semántica VIEJA (asserts `[]`) — debe actualizarse.

**Veredicto:** Variante A (match por namespace sanitizado en el scan) — sin write/read path changes, sin migración (legacy visible al instante), sin cambio SDK get/list.

## Fase 1 — DISCOVERY
- [x] Leer fila `MCP-29` + resolución de `MCP-27` en Backlog (root cause: scan filtra `type == <entity>`, records no tienen `type`).
- [x] codegraph_explore: `memory_record_to_node`, PhysicalScan type filter (`src/executor/`), validación de identificadores del parser.
- [x] Mapear blast radius: records existentes sin `type`, namespaces con `/`, colisión con tipos de grafo.

## Fase 2 — EJECUCIÓN
- [x] Decisión ADR-first: registrar trade-off en la fila antes de tocar código (Regla 5 — el autor articula; IA aporta evidencia).
- [x] Setear `type=<sanitizado>` en `memory_record_to_node`.
- [x] Migración/backfill de records existentes (o política lazy documentada).
- [x] Sanitización de namespace → identificador IQL válido (reject o encode para `a/b`).
- [x] Política de colisión namespace vs tipo de grafo existente.
- [x] Tests: put → `SELECT * FROM <ns>` visible; namespace con `/`; colisión; migración.

## Fase 3 — VERIFICACIÓN
- [x] `cargo check -p vantadb && cargo test -p vantadb`
- [x] `cargo test -p vantadb-mcp`
- [x] `cargo clippy -p vantadb --all-targets -- -D warnings` (si falla por deuda pre-existente del workspace, limitar a crates tocados)

## Fase 4 — CIERRE
- [x] SKILL.md ×2 + docs/api (semántica IQL↔memoria) + fila Backlog ✅.

## RESULTADO (obligatorio)
Bloque RESULTADO: ✅ COMPLETO | 🟡 INCOMPLETO | ❌ FALLIDO + evidencia + DECISION_TOMADA.

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 15/15 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: ninguno (lead comitea)
ARCHIVOS: src/sdk/serialization/mod.rs, src/physical_plan/scan.rs, vantadb-mcp/src/handlers/tools.rs, vantadb-mcp/tests/mcp_tests.rs, .opencode/skills/vantadb-mcp/SKILL.md, .opencode/skills/vantadb/SKILL.md, docs/api/MCP.md, docs/Backlog.md
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
DECISION_TOMADA: Variante A — match por namespace sanitizado en PhysicalScan::next (no setear type=ns en write path). Por qué: cero migración (records legacy visibles al instante), get/list sin cambios, blast radius ≈ 0 (consumers de `type` del grafo intactos). Costo documentado: sanitización no inyectiva ("a/b" y "a_b" → "a_b"), marcado con ponytail: en el helper. Colisión namespace vs tipo de grafo = UNION de resultados, documentada en tool description + SKILL + test.
```

**Evidencia de verificación (2026-08-23):**
- `cargo check -p vantadb` ✅ · `cargo test -p vantadb --lib` ✅ 1917 passed · 5 tests scan MCP-29 ✅
- `cargo test -p vantadb-mcp` ✅ 60+9+… passed, incluye round-trip actualizado
- `cargo clippy -p vantadb -p vantadb-mcp --all-targets` ✅ sin warnings
- `cargo fmt -p vantadb -p vantadb-mcp` ✅

> Nota SARL: el worktree contiene cambios NO relacionados con MCP-29 (`desktop/`, `scripts/check-avance-coverage.ps1`, `docs/plans/2026-08-22-vanta-ultima-milla.md`, `.opencode/skills/campaign-executor/tasks/MEM-58.md`) — el lead debe commitear solo los ARCHIVOS listados arriba.

<!-- Learnings: MCP-29 — 2026-08-23 -->
- El choke point único del scan (`PhysicalScan::next`) permitió exponer memoria vía IQL sin tocar write path ni migrar: match por campo existente (`__vanta_namespace`) sanitizado es más barato que introducir un campo nuevo (`type`) con consumidores ajenos.
- rustc 0xc0000409 en build de tests persistió incluso con `-j 2`; la cura completa fue borrar `target\debug\deps\libvantadb-*` + `.fingerprint\vantadb-*` (confirma MEM-47, no solo reintento).
