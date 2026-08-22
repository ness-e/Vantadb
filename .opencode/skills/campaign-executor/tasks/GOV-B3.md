# GOV-B3: Fix de snippets + guard anti-regresión

## Metadata
- **Plan file:** docs/plans/2026-08-22-doc-governance-plan.md (NO editar)
- **Creado:** 2026-08-22
- **Estado:** ✅ COMPLETED

## Contrato
`.venv/Scripts/python.exe dev-tools/validate_doc_snippets.py` → 0 FAILs de los ítems 1-4 del hallazgo;
grep "ef_search" en docs/glosario/ = 0; FAQ sin claim "fsync cada 5s"; URL GitHub canónica única.

## Resultado
**Antes:** 21 PASS / 31 FAIL / 6 SKIP · **Después:** 34 PASS / **0 FAIL** / 24 SKIP (×2 determinístico)

| Fix | Evidencia |
|-----|-----------|
| graph_bfs ×2 (03:~174, lancedb:~276): IDs int vía `put().node_id` + `graph_bfs([root], depth)` | ambos PASS |
| glosario ef_search → 0 hits (sweep 6 archivos, token→`ef`) | rg = 0 |
| FAQ fsync: default Periodic threshold=1 = fsync por escritura (wal.rs:338, config.rs:151) | corregido |
| URLs GitHub → ness-e/Vantadb (FAQ ×2, master-index ×2) | rg vantadb/vantadb = 0 |
| Chroma blocks autocontenidos → skip (ONNX download cuelga >30s) | SKIP |
| 05:133 IndentationError → dedent + self-contained + metadata str (probe: put_batch exige HashMap<String,String>) | PASS |
| 04 ×5 NameError embed → helper determinístico inline | PASS |
| lancedb ellipsis pyarrow → valores reales (:77 PASS) | PASS |
| lancedb :349 `$gte` → Python SDK NO soporta rangos (TypeError verificado); reescrito equality+client-side | PASS |
| skips justificados: OpenAI key (01×5, 05×1), Ollama down (02×3, 05×1), fixtures vectara (×4), continuations LanceDB (×2) | 24 SKIP total |

## Tickets generados (NO ejecutados aquí)
1. **Harness leak**: `validate_doc_snippets.py` header usa `mkdtemp()` sin cleanup → 224 dirs / 68 GB acumulados (llenó C:). Fix: TemporaryDirectory o finally-cleanup.
2. **API inconsistency**: `put_batch(metadatas=...)` solo acepta valores `str`; `put(metadata=...)` acepta int/float/bool. Unificar o documentar (cercano a P2-5).
3. **Range filters Python**: `$gte` etc. lanzan TypeError genérico — error message mejorable o soportar operadores.
4. `docs/operations/pilot-onboarding-checklist.md:51` referencia `github.com/vantadb/vantadb-examples` (repo/org distintos del canónico ness-e/Vantadb) — decisión owner.
5. master-index.md tenía la URL vieja también (territorio GOV-C4) — corregido aquí de paso.

## Context Save Point
- **Fecha:** 2026-08-22
- **Verificación:** harness ×2 = 34/0/24; markdownlint-cli2 (10 archivos editados + glosario) = 0 issues
- **Problemas conocidos:** disco C: se llenó durante la tarea por el leak del harness — limpiado (68 GB libres); otra sesión editaba web/ en paralelo (sin conflicto de archivos)
