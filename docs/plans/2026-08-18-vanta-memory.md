# Plan de Ejecución: Vanta Memory Engine — port de TDAM (F1–F7)

> **Campaign ID:** (generar al ejecutar — P27 backlog)
> **Inicio:** 2026-08-18
> **Estado:** draft (a ejecutar)
> **Fuente:** `docs/research/tdam/` (PLAN + 01..09 verificados + SYNTHESIS) + análisis multi-agente 2026-08-18 (3× vanta-research)
> **Modo:** secuencial por fases — core LLM-free primero (F1–F3), crate LLM-driven después (F4–F5), opcionales (F6–F7) en segunda iteración.

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| F1 search profile, F2 entidades+ACL, F3 skills, F4 vanta-memory (L1/L2/L3/offload/recall), F5 Context Engine | F6 vanta-proxy + F7 wiki (segunda iteración) · billing/quota (server mode) · SDK sub-clientes (MEM-36) | split 4 servicios TDAM, Redis, SQLite/dual-write, store Mongo, @colbymchenry/codegraph, agent-adapters, 3 imágenes Docker, prompts Kenty chino | MEM-31 (callback destino), MEM-29 (dep red en WASM) |

## Decisiones del usuario (2026-08-18, resueltas)

| # | Decisión | Valor |
|---|----------|-------|
| D1 | Trait LLMRunner | **Ambos (sync + async)** — trait sync base + wrapper async opcional en server |
| D2 | Nodo escena MEM-12 | **Entra en F4** (S, barato — ancla LLM-free de L2) |
| D3 | Tokenizador offload | **tiktoken o200k_base** (⚠️ añade dependencia — validar tiktoken-rs en MEM-23; 3 chars/token como fallback) |
| D4 | Persistencia entity_* | **Nodos en partición InternalMetadata** (patrón thread.rs) |
| D5 | vanta-proxy | **Crate aparte** (`vanta-proxy/`, fuera de default-members) |
| D6 | Puertos propios | Fijar al implementar MEM-25 sin colisionar |
| D7 | Permission-checker | **Versión completa (cadena 7 eslabones, 96 líneas)** |
| D8 | Skill extracción | **Síncrona en v1** (cola local solo si latencia lo exige) |
| D9 | MMD formato | **Mermaid literal v1** (05); contrato META como mejora |
| D10 | Callback S2S | **Hook síncrono local + estado en store** (MEM-28); Vanta Studio lee el estado; S2S real diferido |
| D11 | Alcance F6/F7 | **Segunda iteración** de P27 (F1–F5 primero) |
| D12 | Publicación vanta-memory | **Interno del workspace**; publicar como `vantadb-memory` cuando F4-F5 estables |

## Orden de ejecución (dependencias verificadas)

1. **F1 (MEM-01→02):** parametrizar planner (core) → exponer en MCP. Sin dependencias previas. LLM-free.
2. **F2 (MEM-03→04→05):** entidades → checker → auth server. Core LLM-free. `src/rbac.rs` dead code evaluado en MEM-04.
3. **F3 (MEM-06→07):** skills multi-versión core → tools MCP.
4. **F4 (MEM-08a→08b→09→10→11→12→13→14→15→16→17→18→19→20→21):** fundación crate → contratos+trait → L0 → L1 → L2 → L3 → triggers → skill extract → recall → cursor → MCP scenes. **Checkpoint tras F4.**
5. **F5 (MEM-22→23→24):** Context Engine cascade → emergency/tokens → MMD. **Checkpoint tras F5 (release candidate).**
6. **Segunda iteración (F6 MEM-25..27, F7 MEM-28..33):** proxy → wiki. Opcional.
7. **Transversales:** MEM-34 (telemetría) en paralelo con F4; MEM-35 (data plane) tras F3; MEM-36 (SDK) tras F3; MEM-37 (integración) tras F4/F5; MEM-38 (ADR+docs) gate pre-release.

## Checkpoints

- **Checkpoint 1 (tras F1+F2):** `cargo test -p vantadb` verde; search profile y entidades/checker con tests; review con humano antes de F3.
- **Checkpoint 2 (tras F4):** `cargo test -p vanta-memory` verde con LLM mock; pipeline L0→L3 end-to-end con mock; `cargo check -p vantadb` sin regresiones; review.
- **Checkpoint 3 (tras F5):** offload assemble/mild/aggressive/emergency verde; report correcto; decide D3 definitivamente.
- **Checkpoint 4 (release):** unified-review certify (Pre-Launch Gate, 8 capas) + semver-checks + ADR.

## Riesgos

| Riesgo | Impacto | Mitigación |
|--------|---------|------------|
| Coste LLM por flush (3 llamadas L1/L1.5/L2) | Alto | Modo LLM-free + control triggers (SYNTHESIS §4); defaults configurables |
| Compresión pierde detalle (refs solo a demanda) | Medio | Documentar trade-off en report; cursor idempotente (MEM-20) |
| Heat lo mantiene el LLM (no contador real) | Medio | Documentar; MCP scene_* depende de confiabilidad (MEM-21) |
| `src/rbac.rs` dead code ↔ checker nuevo | Bajo | Decisión explícita en MEM-04 (reemplazo vs coexistencia) |
| CreditCalculator ÷1000 vs ÷10000 TDAM | Bajo (diferido) | Elegir UNA al portar billing (post-F7) |
| Prompts Kenty en chino | Medio | Reescribir principios, no traducir (MEM-10) |

## Relación con P26 (Vanta Studio)

Integración **por contratos, no por ejecución** — campañas independientes (velocidades distintas: Studio Fase 0 en curso; este plan draft). Ningún contrato es bloqueante; la integración real se toca cuando F4/F5 existan (2ª iteración, D11). D10 ya decide el punto de unión principal: *"Hook síncrono local + estado en store (MEM-28); Vanta Studio lee el estado"*.

| # | Contrato | Lado Studio (P26) | Lado Memory (P27) | Estado |
|---|----------|-------------------|-------------------|--------|
| 1 | `explain_memory_search` (VS-CORE-03, ya existe en core) | Lente RETRIEVAL (Fase 1) muestra por qué | Recall (F4) usa el mismo search | Un contrato, dos consumidores — ya resuelto en core |
| 2 | Nodos escena + META `{created,updated,summary,heat}` | Grafo/IQL (Fase 2) + Inspector renderizan escenas/skills/entities | F4 añade nodo escena al grafo core (L2, MEM-12) | Inspector KV genérico ya los cubre — sin código ahora |
| 3 | Audit log JSONL compartido | ACTIVITY + Timeline (Fase 1) | Telemetría por capa (MEM-34): eventos L1/L2/L3/offload | Memory escribe en el MISMO audit log que Studio lee — disciplina, no código |
| 4 | DTO estado (MEM-28) | Studio lee estado vía bridge Tauri | State store (pending→ready, run_id) | Mismo patrón que VS-11 (DTO enriquecido); definir cuando exista F7 |

**Punto de diseño compartido (no bloqueante):** VS-CORE-07 (retención de versiones) lo necesitan ambos — Studio para Historial+Diff, memory para offload/skills versionadas. Acordar el diseño una sola vez cuando VS-CORE-07 se ejecute (task file con cláusula de doble consumidor). Ver también: MEM-01 debe exponer el search profile en las mismas estructuras que `explain` (consumible por la lente RETRIEVAL).

## Open Questions

1. ✅ Orden F1–F7 y decisiones D1–D12 **confirmadas por el usuario 2026-08-18** (ver tabla arriba).
2. ✅ F6/F7 → **segunda iteración** de P27 (D11).
3. ✅ Publicación → **interno del workspace** (D12).
4. ⚠️ D3 (tiktoken): validar que `tiktoken-rs` compile en WASM antes de fijar MEM-23; si no, fallback 3 chars/token documentado.