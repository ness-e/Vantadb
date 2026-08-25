# FIND-06 — READMEs SDK en español → inglés

- **Plan:** docs/plans/2026-08-25-batch-core-server-mcp.md (Task 14)
- **Estado:** ✅ COMPLETED (2026-08-25, commiteado f61cd4ae)
- **Esfuerzo:** 🟡 · **Prioridad:** 🟢 · **Appetite:** 3h
- **Contrato:** READMEs SDK técnicos en inglés; `scripts/validate-docs-coverage.ps1` 0 gaps; FIND-06 en backlog marcado DONE
- **Archivos clave:** `vantadb-python/README.md`, `vantadb-ts/README.md`

## Discovery (DISCOVERY)

**Alcance identificado (Regla 0):**
- `vantadb-python/README.md` — **íntegro en español** (intro, instalación, quickstart, caso de uso, desarrollo, licencia) → **traducir**. Sección Cross-SDK Search Parity ya en inglés (preservar verbatim). Code blocks verbatim (NO traducir código/strings).
- `vantadb-ts/README.md` — **ya en inglés** → sin cambios.
- `integrations/*` (openai, ollama, mem0, haystack, llamaindex, dspy, crewai, letta, langchain), `providers/*`, `vantadb-wasm/demo/README.md` — **ya en inglés** (falsos positivos de grep: "del"→delete, "con"→contact, "para"→parameter).
- `vantadb-wasm/pkg/README.md` — generado por wasm-pack, **no editar**.
- `vantadb-node/` — no tiene README en la raíz (solo node_modules).
- `examples/demo/README.md` — en inglés.

→ Único archivo a modificar: `vantadb-python/README.md`.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `vantadb-python/README.md` (149 líneas, leído).
- **Referencias hacia dentro:** README no es importado por código. Links salientes a `../docs/QUICKSTART.md`, `../docs/api/BINDINGS_NAMESPACES.md`, `../docs/api/PYTHON_SDK.md`.
- **Referencias entrantes:** no hay imports. Es documentación standalone.
- **Veredicto:** edición segura — solo traducción de prosa descriptiva; code blocks y la sección Cross-SDK Search Parity se preservan verbatim. No rompe links (no se cambian rutas).

## Steps

1. ✅ Traducir `vantadb-python/README.md` a inglés (prosa descriptiva), preservando code blocks verbatim y sección Cross-SDK Search Parity.
2. ✅ Verificar que no quedan secciones técnicas en español (grep de stopwords/acentos) y que links internos no se rompieron.
3. ✅ `scripts/validate-docs-coverage.ps1` → 0 gaps (tras arreglar 4 gaps MCP.md pre-existentes de MOD-10).
4. ✅ Marcar FIND-06 DONE en `docs/Backlog.md` (fila P33 + sección completadas).
5. ✅ Update task state + recitation + handoff.

## Context Save Point

- Cambios: `vantadb-python/README.md` (traducción a inglés), `docs/api/MCP.md` (+4 tools: memory_versions, memory_supersede, remove_edge, vacuum; Core Tools 36→42).
- `vantadb-ts/README.md` ya estaba en inglés (sin cambios). Integrations/providers/wasm-demo en inglés. `vantadb-wasm/pkg` generado (no editar). `vantadb-node` sin README raíz.
- Colateral: los 4 gaps de MCP.md eran pre-existentes (MOD-10 commiteó tools pero no actualizó docs/api/MCP.md) — arreglados inline.
- El lead commitea (sub-agentes NO commitean): `vantadb-python/README.md`, `docs/api/MCP.md`, `docs/Backlog.md`, task file FIND-06.md.
