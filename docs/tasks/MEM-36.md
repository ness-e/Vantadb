# MEM-36 (META-TAREA): Crear el plan de campaña Bindings SDK — sub-clientes por dominio

## Metadata
- **Fuente:** `docs/Backlog.md` fila MEM-36 + decisiones del usuario 2026-08-21 ("campaña separada, con esta meta-tarea que produce el plan")
- **Tipo:** Planning/spec (NO código) — **Ruta: vanta-lead** (pipeline)
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡
- **Creado:** 2026-08-22
- **Estado:** ⬜ PENDING
- **Entregable ÚNICO:** `docs/plans/<FECHA>-vantadb-bindings-sdk.md` — plan completo de la campaña de bindings, con sus tareas listas para `/pipeline run`

## Objetivo
A partir de esta spec, producir el plan ejecutable de la campaña que exponga **sub-clientes por dominio** en los SDKs TS y Python, manteniendo 100% backward-compat.

## Contexto para quien ejecute esta meta-tarea

### Qué es un sub-cliente por dominio
Hoy los SDKs exponen una API **plana**: `db.put()`, `db.search()`, `db.get()`, etc. — todos los métodos al nivel superior. La superficie creció con F4-F7 (memoria con scopes, skills, escenas, wiki, conversación, context engine) y seguirá creciendo como lista plana. Un sub-cliente agrupa por dominio:

```ts
// ANTES (plano, sigue funcionando — backward-compat 100%)
db.put({ namespace, key, payload });
// DESPUÉS (agrupado, azúcar organizativa encima)
db.memory.put({ namespace, key, payload });
db.conversation.add(...);
db.graph.bfs(...);
```

### Superficies a agrupar (mapear contra la API real en DISCOVERY)
| Dominio propuesto | Métodos candidatos (verificar nombres reales) |
|---|---|
| `memory` | put/get/list/search/search_multi/supersede/versions/similar_to_key |
| `conversation` | capture L0 / record_turn / read_messages (MEM-09), conversation/add pipeline |
| `scene` | upsert/get/list/current_scene + tools read/write/edit |
| `persona` | get_persona/generate_persona/triggers |
| `recall` | perform_auto_recall / assemble_with_recall |
| `skills` | SkillStore CRUD + versions + cleanup |
| `wiki` | store state machine + search/read/list/graph |
| `graph` | bfs/dfs/topological + node ops |

### Referencias obligatorias
1. **TDAM SDK estructura:** clon `C:\Users\Eros\AppData\Local\Temp\opencode\tdam` @ `97f9465` — `sdk/memory-core/typescript/src/v3/{client,memory-prompt-client,skill-client,metadata-client}.ts` (clientes por dominio reales de TDAM)
2. **Contrato WASM:** learning VS-CORE-05 (`.opencode/AGENTS.md`): tras agregar método público a `vantadb-wasm/src/lib.rs`, `tsc` del TS falla hasta regenerar `vantadb-wasm/pkg` con `wasm-pack build --dev` — el `.d.ts` del pkg es el contrato
3. **Reglas:** `.opencode/rules/js-ecosystem.md` + `python-bindings.md` (cargar antes de diseñar)
4. **Estado actual:** suites vanta-proxy 52 · vantadb-mcp 29 · vanta-memory 453 · core wiki 24; total workspace 2568+

### Decisiones ya tomadas (heredadas)
- **Backward-compat 100%** — la API plana NO se toca; sub-clientes son aditivos
- Campaña SEPARADA del roadmap Rust (decisión usuario 2026-08-21)
- Sin release durante la campaña

### Decisiones que el plan DEBE dejar cerradas (no abrir en las tareas)
1. Alcance v1: ¿todos los dominios o solo memory/conversation/graph?
2. Orden: ¿WASM primero (contrato .d.ts) y luego TS/Python? (recomendado — el pkg es la fuente)
3. Testing: ¿tests de contrato compartidos entre TS y Python?
4. Versionado: ¿los sub-clientes disparan minor bump?

### Riesgos conocidos
- Regeneración wasm pkg con fricción (VS-CORE-05) → tarea dedicada de build antes de tocar SDKs
- Drift entre .d.ts generado y tipos escritos a mano en vantadb-ts → generar, nunca editar a mano
- Python: numpy getter PERF-31 ya existe — no romper

## Steps
1. Verificar superficie real: listar métodos públicos expuestos por vantadb-wasm/vantadb-ts/vantadb-python hoy
2. Mapear dominios ↔ métodos (tabla completa)
3. Escribir el plan file con tasks atómicas, contratos mecánicos (`cargo check -p vantadb-wasm` + `wasm-pack build` + `tsc` + pytest) y risk registers
4. Cerrar: commit del plan + recitation apuntando a `/pipeline run <plan>`

## Contrato
"Existe `docs/plans/<FECHA>-vantadb-bindings-sdk.md` con ≥4 tareas ✅ DO, cada una con archivos clave, contrato mecánico verificable y task file referenciado; commit hecho"

## Definition of Done
- [ ] Plan creado y commiteado
- [ ] Decisiones cerradas documentadas en el header del plan
- [ ] Recitation apuntando a la ejecución
