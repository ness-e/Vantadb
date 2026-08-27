# TASK QW-2: langchain ids parciales — generar UUIDs para docs sin id ANTES de filtrar

## Metadata
- **Plan file:** `docs/plans/2026-08-25-integrations-research-wins.md`
- **Creado:** 2026-08-27T12:00
- **last-synced:** 2026-08-27T12:00
- **Estado:** ✅ COMPLETED
- **Ruta:** vanta-worker
- **Prioridad:** Wave 1 — Fixes de bugs de contrato (H-03 =MOD-47)

## Blast Radius

Callers | Callees | Implicaciones

- `VantaDBVectorStore.add_documents` (langchain/vectorstore.py:450-472) — caller: usuarios LangChain `VectorStore.add_documents(docs)` con mezcla ids parciales; callee: `add_texts` → `_db.put` (VantaDB core). Bug filtraba ids (`[doc.id for doc if doc.id is not None] or None`) → lengths mismatch ValueError engañoso en `add_texts`. Fix genera UUIDs ANTES de filtrar (`[doc.id if doc.id else str(uuid.uuid4()) for doc ...]`).
- `VantaDBVectorStore.add_texts` (langchain/vectorstore.py:387-448) — valida `ids length == texts length`, genera `_build_key` si ids es None/empty. Con fix ids nunca None y length siempre match.
- `VantaDBVectorStore._build_key` — fallback deterministico uuid5(text:index) — ya no alcanzado para docs sin id (ahora uuid4 random) pero mantenido para `add_texts` sin ids.
- Tests: `integrations/langchain/tests/test_vectorstore.py::test_add_documents_partial_ids`, `test_add_documents_all_with_ids_preserved`, + suite 25 tests langchain.
- **Implicaciones:** thin wrapper, sin tocar `vantadb/src/*` (protegido). Blast radius pequeño (<5 archivos), solo `integrations/langchain/`. No toca paths multi-índice/dashmap/parking_lot/Tokio → no requiere auditoría concurrencia (Regla 8). No hot path vectorial → no requiere perf bench (Regla 9). No trust boundary nuevo → no security hardening.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos (antes de editar):**
  - `integrations/langchain/vantadb_langchain/vectorstore.py` (573 líneas completas)
  - `integrations/langchain/tests/test_vectorstore.py` (280 líneas completas)
  - `integrations/langchain/pyproject.toml` (completo)
  - `integrations/langchain/vantadb_langchain/__init__.py` (completo)
  - `docs/plans/2026-08-25-integrations-research-wins.md` (Wave 1 QW-2)
  - `integrations/langchain/README.md` (completo, Why VantaDB)
- **Referencias hacia dentro (qué importa este archivo):**
  - `import uuid` → stdlib `uuid.uuid4()` para ids faltantes (ponytail: stdlib existente)
  - `from langchain_core.documents import Document` → `Document.id` optional str | None
  - `from langchain_core.vectorstores import VectorStore` → abstract base, contrato `add_documents`/`add_texts`
  - `import vantadb_py as vanta` → `VantaDB.put(namespace,key,payload,metadata,vector)` PyO3
- **Referencias entrantes (qué depende de lo que cambio):**
  - `tests/test_vectorstore.py` — `test_add_documents_partial_ids` y `test_add_documents_all_with_ids_preserved` ejercitan fix directamente
  - `integrations/langchain/README.md` → ejemplos `add_documents` (no ids)
  - `packages/langchain-vantadb` (adapter package espejo) — debe mantenerse en sync si existe (verificar)
  - Ningún módulo Rust core depende de este adapter (aislado)
- **Veredicto:** impacto localizado, seguro para edit. Fix ya en HEAD (96e143ec), 1 línea; tarea pipeline-full re-verifica contrato y documenta edges. No toca paths multi-índice/dashmap/parking_lot/Tokio → no requiere auditoría concurrencia. No hot path → no perf bench.

## Spec

N/A — bug-fix con contrato mecánico (Wave 1 QW-2). No agrega símbolos públicos nuevos; solo corrige asignación de ids parciales.

Problema: `add_documents` con mezcla de docs con/sin `id` filtraba `ids = [doc.id for doc in documents if doc.id is not None] or None` → lista filtrada más corta que `texts` → `add_texts` lanzaba `ValueError: ids length (1) must match texts length (2)` engañoso, o silenciosamente mapeaba ids incorrectos si se pasaba ids filtrados.
Criterio: `add_documents([Document(id="custom-id-1"), Document(page_content="without id")])` → `len(ids)==2`, `ids[0]=="custom-id-1"`, `ids[1]` es UUID válido, sin ValueError; `get_by_ids` recupera ambos; test `test_add_documents_partial_ids` y `test_add_documents_all_with_ids_preserved` pasan.
Alcance: `integrations/langchain/vantadb_langchain/vectorstore.py:470-472` (línea `ids = [doc.id if doc.id else str(uuid.uuid4()) ...]` + comentario "Generate UUIDs ... BEFORE filtering").
Decisiones: usar `str(uuid.uuid4())` stdlib (no deterministico, no colisión); `doc.id if doc.id else` cubre `None` y `""` (empty falsy); no validar longitudes extra (ponytail: mínimo código); no tocar `add_texts` (ya valida lengths).

## Contrato

```
python -m pytest integrations/langchain/tests -q  # 25 passed (incl. test_add_documents_partial_ids, test_add_documents_all_with_ids_preserved)
python -m pytest integrations/langchain/tests/test_vectorstore.py::test_add_documents_partial_ids -q  # pasa
add_documents con mezcla con/sin id genera UUIDs para faltantes ANTES de filtrar — no ValueError engañoso
```

Verificación mecánica:
1. `python -m pytest integrations/langchain/tests/test_vectorstore.py::test_add_documents_partial_ids -q` ✅
2. `python -m pytest integrations/langchain/tests/test_vectorstore.py::test_add_documents_all_with_ids_preserved -q` ✅
3. `python -m pytest integrations/langchain/tests -q` → 25 passed ✅

## Herramientas

Skills cargadas: `source-driven-development`, `ponytail`, `systematic-debugging`, `test-driven-development` (≤4 por SDP, resto sin candidatos)

- **source-driven-development:** verificar APIs externas (LangChain Document.id semantics, uuid stdlib, VantaDB add_texts contract)
- **ponytail (full):** ladder mínimo — stdlib `uuid.uuid4()` (rung 3), 1 línea, sin deps nuevas, sin abstracciones
- **systematic-debugging:** root cause: filtrado `if doc.id is not None` vs generación `if doc.id else uuid` — un solo fix en shared function cubre todos los callers
- **test-driven-development:** contrato ya en tests existentes (test_add_documents_partial_ids), verify rojo→verde (antes ValueError, después UUID)

**SKILLS_CARGADAS (SDP):** source-driven-development, ponytail, systematic-debugging, test-driven-development + SDP sin candidatos adicionales (keywords: langchain, vectorstore, add_documents, uuid, ids, Document)
Lifecycle mapping BUILD (source-driven, ponytail, TDD) + VERIFY (systematic-debugging)
Grep SKILLS-MANIFEST.md por `langchain|vector|embedding|uuid|ids` → sin skill directa; `vantadb` genérica no añade valor específico al fix (ya cubierta por source-driven).
Discovery ≤8 skills → 4 cargadas, justificadas arriba.

## Steps

### Step 1: DISCOVERY — verificar fix ya aplicado y blast radius
- **Archivos:** `integrations/langchain/vantadb_langchain/vectorstore.py:450-472`, `integrations/langchain/tests/test_vectorstore.py:260-280`
- **Acción:** Confirmar que fix QW-2 (96e143ec) ya está en disco: `ids = [doc.id if doc.id else str(uuid.uuid4()) for doc in documents]` con comentario BEFORE filtering. Mapear Regla 0 completa arriba. Validar que 27 tests existentes cubren contrato, incluyendo `test_add_documents_partial_ids`. Verificar `packages/langchain-vantadb` espejo si existe.
- **Verify:** `git show 96e143ec -- integrations/langchain/vantadb_langchain/vectorstore.py` diff 1 línea; `python -m pytest integrations/langchain/tests/test_vectorstore.py::test_add_documents_partial_ids -q` → 1 passed (3.75s) ✅
- **Estado:** ✅ DONE (2026-08-27 — fix ya en HEAD; Regla 0 mapeada; no package espejo)

### Step 2: VERIFY — suite contract + edge mixtos exhaustivo
- **Archivos:** `integrations/langchain/tests/test_vectorstore.py`
- **Acción:** Ejecutar full suite langchain (27 tests) + casos borde: mezcla ids parciales, todos con id preservados, doc.id="" (empty) → UUID, doc.id=None → UUID, ids unicidad, get_by_ids recupera. Inline python script para 6 casos mixtos.
- **Verify:** `python -m pytest integrations/langchain/tests -q` → 27 passed (14.50s) ✅; `python -m pytest ...::test_add_documents_partial_ids ...::test_add_documents_all_with_ids_preserved -v` → 2 passed; `test_qw2_edges.py` 6 edge checks ALL PASS (T1..T6) ✅
- **Estado:** ✅ DONE (2026-08-27)

### Step 3: CIERRE — verify full + no-commit + recitation + progreso
- **Archivos:** `.opencode/skills/campaign-executor/tasks/QW-2.md`, `docs/plans/2026-08-25-integrations-research-wins.md`
- **Acción:** `cargo fmt --check` skip (py only); pytest gate ya verde; NO commit (Reglas usuario: no commit); actualizar task file con steps ✅ y Context Save Point; handoff.
- **Verify:** task file steps ✅ 3/3, `git diff HEAD -- integrations/langchain/` vacío (fix ya en 96e143ec), no commit por instrucción
- **Estado:** ✅ DONE (2026-08-27)

## Dependencias
- Ninguna (Wave 1 QW-2 independiente; QW-1/QW-3 disjuntos; QW-1 ya verificado 2026-08-27)

## Notas
- Fix ya merged en 96e143ec (1 línea 470-471 + comentario). Esta tarea pipeline-full re-verifica contrato y documenta edges; si no hace falta código adicional, cierra como verify-only con evidencia (patrón QW-1).
- Ponytail: stdlib uuid, 1 línea, no factory, no config futura.ladder: existe (uuid stdlib) → usarla. Skipped: validación longitudes extra / mensaje accionable custom — add_texts ya lanza ValueError accionable; duplicaría lógica.
- `doc.id if doc.id else` vs `if doc.id is not None`: cubre `None` y `""` vacío (LangChain permite id=""). Si id=0 (no válido) también genera UUID — correcto.
- `packages/langchain-vantadb` no existe como package separado (solo `integrations/langchain`); no requiere sync.

## Context Save Point
- **Fecha:** 2026-08-27T12:00
- **Branch:** develop
- **CI pendiente:** no — local pytest 25 passed esperado (verificar en Step 2)
- **Decisiones:** mantener fix actual (uuid4 para faltantes BEFORE filtering), no añadir validación extra; minimal diff wins
- **Problemas conocidos:** ninguno
- **Próxima tarea:** QW-3
