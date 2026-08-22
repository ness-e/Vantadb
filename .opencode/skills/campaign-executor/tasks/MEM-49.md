# MEM-49 - ADR-029 articulación humana + gate D24-D37

**Plan:** `docs/plans/2026-08-22-vanta-final-cierre.md` → Task 7 → Estado inicial ⬜ PENDING
**Crate:** ninguno (docs-only) · **Ruta:** vanta-docs prepara → AUTOR HUMANO edita → lead commitea · Human-in-loop (D41, Regla 5)

## Objetivo
La IA prepara el material de revisión (documento-guía socrático con contexto, evidencia file:line, consecuencias y preguntas socráticas por decisión D21-D36). **NO redacta decisiones en primera persona.** El AUTOR HUMANO edita ADR-029 con sus palabras, firma y commitea. La tarea IA queda COMPLETED cuando el material está preparado; la articulación humana queda AGENDADA explícitamente para el usuario.

## Impacto mapeado (Regla 0)
- **Leídos completos:** `docs/architecture/adr/ADR-029-vanta-memory-context-engine.md` (155L), `docs/plans/2026-08-22-vanta-final-cierre.md` (Task 7 + encabezado D38-D41), `docs/plans/archive/2026-08-21-vanta-proxy-knowledge.md` (294L — definiciones D24-D37).
- **Fuentes de evidencia consultadas:** `docs/research/tdam/07-proxy.md`, `docs/research/tdam/08-knowledge-panel-sdk.md`; código: `vanta-proxy/src/{rate_limit,auth,session,inject,mem_command,config}.rs`, `src/wiki/{sources,chunker,state,store}.rs`, `src/graph.rs`, `vanta-memory/src/ingest/callback.rs`.
- **Referencias entrantes:** ninguna en código; ADR-029 recibirá un puntero al documento-guía.
- **Veredicto:** cambio aditivo docs-only (2 archivos nuevos + 1 bloque en ADR-029). Cero blast radius en Rust/bindings. Sin deps.

## Steps
- [x] S1 - Task file creado + Regla 0 mapeada.
- [x] S2 - `docs/architecture/adr/guia-revision-ADR-029-y-D24-D37.md`: una entrada por decisión (D21, D22, D23, D24, D25+D34, D26, D27, D28, D29, D30, D31, D32, D33, D35, D36) con: trade-off/alternativas, evidencia técnica (file:line), consecuencias asumidas, preguntas socráticas (en segunda persona, para el autor). Sección final: checklist del autor (editar ADR-029 en primera persona, firmar, commit propio). D37 (riesgos aceptados 2026-08-21) referenciada como marco transversal.
- [x] S3 - ADR-029: bloque `> ⏳ BORRADOR — pendiente articulación humana...` insertado al inicio (no existía el puntero a la guía).
- [x] S4 - Verify: ambos archivos existen; rutas citadas verificadas contra filesystem (sin rutas rotas); inglés técnico.
- [x] S5 - `campaign_update_task_state` taskId=7 completed + recitation §3 aclarando: trabajo IA hecho, articulación humana AGENDADA para el usuario.

## Contrato de verificación
Docs-only: no aplica cargo verify. Verificación mecánica = existencia de archivos + grep de referencias internas resuelve (paths existen) + ADR-029 contiene el bloque BORRADOR apuntando a la guía. Sin commit (regla explícita del orquestador: NO commitear).

## Context Save Point
- REGLA 5 CRÍTICA: la guía NO redacta decisiones en primera persona. Las preguntas socráticas verifican comprensión del autor; las respuestas las escribe él en el ADR.
- Stop condition del plan: si el autor no dispone tiempo → tarea queda abierta honestamente. Resolución adoptada: estado IA=completed (material listo), deuda humana registrada en recitation (`deuda` + `queda_pendiente`).
- Evidencia clave ya compilada en la guía: TDAM refs (guard.ts:40-51, session-key.ts:9-19, store.ts:31,116, wiki-service.ts:1026, manager.ts:110, README:28, git-fetcher.ts:59-63 / KNOWLEDGE_SSRF_CHECK=off l.32-37) + código VantaDB real verificado esta sesión (rate_limit.rs:1,19,32; auth.rs:3,78,86-90; session.rs:20-28,90-94; inject.rs:1-5,157; mem_command.rs:7-22; config.rs:1,9,17; sources.rs:3-33; chunker.rs:13; graph.rs:61,234,258; callback.rs:6-21).
