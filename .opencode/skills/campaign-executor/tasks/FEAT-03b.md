# FEAT-03b — Consolidación asistida: core decay (contrato + diseño)

> Plan: `docs/plans/2026-08-19-vanta-studio-fase4.md` (Task 17) · Estado: ⏳ PENDING → in-progress al delegar
> D16 (usuario 2026-08-20): **"Quiero todo"** — (a) UI + (b) core decay. Este task file cubre (b) core decay.
> (a) UI corre en paralelo por vanta-worker (task file FEAT-03a.md) — archivos disjuntos (core Rust vs desktop/).

## Contexto (verify del lead, 2026-08-20)
- W0..W3 parcial: FEAT-01/02 commiteados. UI de consolidación (a) en curso por vanta-worker — NO tocar desktop/.
- El research pide decay Mem0/memify/Cognee pattern (SYNTHESIS §4 OPERACIONES:155, 03 lección 5:252, 07 Fix 3:89) — duplicados/superados con diff visible.
- Core actual: records con `metadata` arbitraria (serde_json::Value), `version` (VDB_SKIP_KEYS en dump), namespace model.

## Contrato (plan Task 17, alcance (b) D16 — decay automático en core)
Definir el **diseño y contrato** del decay automático en el core (dominio vanta-arch/vanta-engine; el plan lo marca como "task core separada con contrato"). NO implementar — entregar el diseño + contrato + ADR para que vanta-worker implemente después. Si el diseño revela que el decay en core NO es viable/necesario (p.ej. el decay es policy del cliente, no del motor), documentarlo y proponer la alternativa (follow-up o descartar).

## Pasos
1. **DISCOVERY** — leer el research (docs/research/human-facing-db-ui/ — 01 lección 8, 02 §9, 03 lección 5, 07 Fix 3; SYNTHESIS §4), core actual: `src/sdk/` (search, serialization/vector_types.rs, types.rs VantaOperationalMetrics), `src/planner.rs` (RRF), storage model (fjall/LSM? — verificar qué storage usa hoy), WAL. ¿Existe algún concepto de TTL/expiry/decay hoy? (`expires_at_ms` apareció en el dump parser → verificar si el core lo soporta).
2. Diseño: semántica de decay (qué dispara, qué marca: `superseded_by`/`version`/TTL), dónde vive (búsqueda vs ingest vs background), costo, qué expone al SDK (campos en types.rs, wrapper), compatibilidad con el model actual.
3. Contrato de implementación: pasos concretos para vanta-worker, archivos, tests, verificación. ADR (docs/architecture/ o docs/adr/) con la decisión.
4. NO implementar en core — solo diseño + contrato + ADR. NUNCA tocar desktop/ (FEAT-03a en paralelo).

## Verificación
- ADR + contrato escritos en el repo (archivos de docs — disjuntos de desktop/ y de src/).
- Si el diseño concluye "no hacer" → justificación documentada.
- El lead revisa el contrato antes de que vanta-worker lo implemente (no se implementa en esta sesión).