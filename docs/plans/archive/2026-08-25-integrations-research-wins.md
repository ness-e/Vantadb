# Plan: Integrations Quick Wins (H-01..H-05, H-08..H-11)

> **Origen:** `/research integrations` → `docs/reviews/research-integrations-20260825.md` (score 6.3/10)
> **Decisiones HITL 2026-08-25:** 9 hallazgos APLICAR aprobados. Estrategias H-06/H-07 →
> P44 `INTG-01/02` en `docs/Backlog.md`. Ningún descartado.
> **Reconciliación:** MOD-46..50 (agregadas commit c7b7e559, removidas del Backlog sin
> completarse ni archivarse — huérfanas) quedan **absorbidas por este plan**:
> MOD-46→QW-1 · MOD-47→QW-2 · MOD-48→QW-3 · MOD-49→QW-4 · MOD-50→QW-5.
> Listo para `/pipeline run docs/plans/2026-08-25-integrations-research-wins.md`.

## Wave 1 — Fixes de bugs de contrato

### QW-1 (H-02 =MOD-46): crewai from_dict + cursor
- **Contrato verificable:** roundtrip to_dict→from_dict→_run(query) no lanza TypeError;
  `from_dict` reconstruye embedding callable desde config (no pasa el string crudo).
  `list(cursor=...)` convierte str→int antes de llamar a `list_memory` (patrón dspy).
  Tests crewai cubren ambos casos.
- **Archivos:** `integrations/crewai/vantadb_crewai/vectorstore.py:164-176,217-218`

### QW-2 (H-03 =MOD-47): langchain ids parciales
- **Contrato verificable:** `add_documents` con mezcla de docs con/sin id no lanza
  ValueError engañoso — generar UUIDs para los faltantes ANTES de filtrar (o validar
  longitudes y fallar con mensaje accionable). Test del caso mixto.
- **Archivos:** `integrations/langchain/vantadb_langchain/vectorstore.py:470`

### QW-3 (H-04 =MOD-48): llamaindex attrs privados + import
- **Contrato verificable:** `_namespace`/`_client` declarados como `PrivateAttr`;
  anotación de tipo resuelta bajo `get_type_hints()` (import completo). Test que
  ejercita serialización pydantic del store.
- **Archivos:** `integrations/llamaindex/vantadb_llamaindex/vectorstore.py:42-50`

## Wave 2 — Limpieza / dedup

### QW-4 (H-05 =MOD-49): dedup gemelos ollama/openai
- **Contrato verificable:** un módulo interno compartido (`integrations/_shared/` o
  paquete común interno) para `Document`, `add_texts`, `delete`, async helpers;
  ollama/openai quedan como thin subclasses (~200 líneas menos). `asimilarity_search`
  consistente entre ambos (mismo mecanismo executor). Suites existentes pasan sin
  cambios de API pública.
- **Archivos:** `integrations/{ollama,openai}/vantadb_*/vectorstore.py`

### QW-5 (H-10 =MOD-50): nits agrupados
- **Contrato verificable:** `categorize()` eliminada (~65 líneas, ya DEPRECATED);
  heurística `_normalize_score` mem0 documentada con su semántica exacta (o reemplazada
  por regla explícita distancia→score); haystack `count_documents()` cuenta por páginas
  con cursor (sin materializar hasta 1M records). Tests correspondientes.
- **Archivos:** `crewai/:229-293` · `mem0/vantadb_mem0/vectorstore.py:45-55` · `haystack/vantadb_haystack/vectorstore.py:370-382`

### QW-6 (H-08): decisión letta
- **Contrato verificable:** README de `integrations/letta/` declara estado experimental
  y por qué (Letta es plataforma stateful con memoria propia — sin contrato de
  vector-store público), O el adapter se retira si la decisión lo indica. Decisión
  mínima lazy: documentar experimental (borrar solo si Letta confirma incompatibilidad).
- **Archivos:** `integrations/letta/README.md`

## Wave 3 — Publicación y visibilidad

### QW-7 (H-01 =MKT-18f ampliada): publicar 9 paquetes en PyPI
- **Contrato verificable:** `pypi.org/pypi/vantadb-<fw>/json` responde 200 para los 9
  (`langchain, llamaindex, dspy, haystack, crewai, letta, mem0, ollama, openai`).
  Paquetes Python puros → build sdist/wheel + twine (no requieren maturin). Agregar
  job al release pipeline o publicar manual una vez; decidir versión inicial 0.5.0
  alineada a vantadb-py. Actualizar MKT-18f al cerrarse.
- **Archivos:** `integrations/*/pyproject.toml`, `.github/workflows/` (si CI), PyPI

### QW-8 (H-11): posicionamiento en READMEs
- **Contrato verificable:** cada README de adapter tiene sección "Why VantaDB" honesta:
  engine embebido Rust local-first vs zep (requiere servidor) / cognee (KG runtime
  propio) / memoria nativa del framework (cuándo basta la nativa). Sin claims de
  performance sin benchmark (Regla 11).
- **Archivos:** `integrations/*/README.md` ×9

## Wave 4 — Calidad continua

### QW-9 (H-09): matriz CI de compatibilidad
- **Contrato verificable:** workflow (scheduled/manual para no cargar Fast Gate) que
  instala cada framework en su versión actual + pin mínimo declarado y corre la suite
  del adapter contra ambos. Falla visible si un release del framework rompe el adapter.
  Pins corregidos según resultado.
- **Archivos:** `.github/workflows/` (nuevo scheduled), `integrations/*/pyproject.toml`

## Verificación por tarea

```
python -m pytest integrations/<fw>/tests -q        # suite del adapter tocado
dev-tools/verify_changed.ps1                        # pre-commit
twine check integrations/<fw>/dist/*                # solo QW-7
```

## Fuera de alcance de este plan

Estrategias H-06 (LangGraph checkpointer/BaseStore) y H-07 (CrewAI Memory backend) →
P44 `INTG-01/02` en Backlog.

=== RECITATION QW-1 ===
Campaign ID: b5d6239d-6143-469e-87ac-63439ef798b2
Objetivo activo: QW-1 (H-02=MOD-46) crewai from_dict + cursor — roundtrip to_dict→from_dict→_run no TypeError; from_dict reconstruye embedding callable; list cursor str→int
Estado: completed
Última acción: Harden vectorstore.py: list empty→None+ValueError + next_cursor fix; from_dict embedding callable + k/top_k; verify 8 passed + edge chain
Resultado: ✅
Próxima acción: Lead commitea: git add integrations/crewai/vantadb_crewai/vectorstore.py .opencode/skills/campaign-executor/tasks/QW-1.md
Contrato: verificacion: python -m pytest integrations/crewai/tests -q → 8 passed (9.56s); edge: list cursor ''→None, '0'→0, str→int, invalid→ValueError accionable, next_cursor fix (is not None); from_dict embedding callable via 'embedding' + k/top_k compat
 evidencia: claim: roundtrip no TypeError | evidencia: python -m pytest test_from_dict_roundtrip_no_typeerror PASSED | confianza: alta
 evidencia: claim: list(cursor=str)→int | evidencia: test_list_cursor_string PASSED + inline p1 cursor 2 → p2 4 chain | confianza: alta
 artefactos: integrations/crewai/vantadb_crewai/vectorstore.py
 invariantes: thin wrapper, no tocar vantadb/src/*, blast radius <10 archivos
 deuda: ninguna
 queda_pendiente: QW-2
Próxima tarea si completa: QW-2
=== END RECITATION ===

=== RECITATION QW-6 ===
Campaign ID: b5d6239d-6143-469e-87ac-63439ef798b2
Objetivo activo: QW-6 (H-08) decisión letta — README declara estado experimental y por qué (Letta stateful sin contrato vector-store público)
Estado: completed
Última acción: Verify-only: grep experimental línea 34 + stateful plataforma línea 36 + no public vector-store contract — 3/3 grep ✅; diff vacío fix ya en 96e143ec (19 ++); task file 3/3 DONE; no commit por instrucción
Resultado: ✅
Próxima acción: Lead commitea (cuando aplique): git add .opencode/skills/campaign-executor/tasks/QW-6.md docs/plans/2026-08-25-integrations-research-wins.md — no commit ahora por regla usuario
Contrato: verificacion: Select-String -Path integrations/letta/README.md -Pattern "experimental" → 34: ## Status: experimental ✅; Select-String "stateful platform" → 36: Letta is a stateful platform with its own memory layer and no public vector-store contract ✅; git diff HEAD -- integrations/letta/README.md → vacío (fix 96e143ec) ✅
 evidencia: claim: README declara experimental | evidencia: integrations/letta/README.md:34 `## Status: experimental` (Select-String match) | confianza: alta
 evidencia: claim: README explica por qué letta stateful sin contrato vector-store | evidencia: integrations/letta/README.md:36-40 `Letta is a stateful platform with its own memory layer and no public vector-store contract; this adapter is a community convenience, not an officially supported integration.` (Select-String stateful + no public) | confianza: alta
 artefactos: integrations/letta/README.md, .opencode/skills/campaign-executor/tasks/QW-6.md
 invariantes: doc-only, no tocar vantadb/src/*, blast radius 1 archivo, inglés source of truth, no performance claims sin benchmark (Regla 11 OK)
 deuda: ninguna — ADR formal solo si Letta confirma incompatibilidad y se retira adapter ( Spec decide lazy doc-only); retirement teardown trivial
 queda_pendiente: QW-7
Próxima tarea si completa: QW-7
=== END RECITATION ===
