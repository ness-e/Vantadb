# FND-24 — ICP + JTBD definidos con evidencia de usuarios reales

> **Estado:** ✅ COMPLETO (2026-08-16) — steps 1-4 ✅
> **Fuente:** `docs/Backlog.md:518` (P20d, prio 🟢)
> **Contrato:** "ICP + JTBD definidos con evidencia de usuarios reales"
> **Rol:** vanta-docs (leaf — no commit, no git add)

## Contexto

VantaDB es una base de datos vectorial Rust embebida (WAL + HNSW + BM25 + RRF) con bindings Python/TS + WASM. La tarea pregunta: ¿para quién es? (¿dev de chatbot local o de edge computing?) y ¿qué job cumple? (¿por qué VantaDB sobre SQLite+vector extension?).

**Hallazgo de DISCOVERY (crítico):** el repo NO tiene evidencia de usuarios reales externos:
- GitHub `@ness-e/Vantadb`: ⭐ 2 stars, 0 forks (fuente: `docs/Investigaciones/investigacion-equipo-2026-08-09.md:119`)
- crates.io `vantadb`: 32 descargas, **0 dependents** (fuente: `investigacion-equipo-2026-08-09.md:120`)
- Manual estratégico: "NADIE ha encontrado tu proyecto orgánicamente" (`VantaDB_Manual_Estrategico_Unificado.md:1043`)
- El Show HN aún no ocurrió (planificado sept 2026) — no hay thread de HN que citar.
- No hay issues de usuarios con contexto real (los templates existen pero el volumen es ~cero).

**Consecuencia (regla de la tarea):** TODO claim de ICP/JTBD se marca como **hipótesis** derivada de fuentes internas de posicionamiento, con plan de validación. NO inventar usuarios.

## Fuentes de evidencia (todas locales, verificadas por lectura)

| Fuente | Qué aporta | Tipo |
|---|---|---|
| `README.md:34` | Claim de posicionamiento: "AI agents, local RAG pipelines, and edge applications" | Hipótesis (creator) |
| `docs/vision/VISION.md:50-108` | ICP primario/secundario/terciario + pains citados | Hipótesis (creator) |
| `docs/strategy/GO_TO_MARKET.md:140-191` | 3 verticales GTM (Local LLM Stack, Agentic Frameworks, AI-IDE) | Hipótesis (creator) |
| `docs/strategy/SHOW_HN_PREP.md:23-34` | Audiencia objetivo + pains de alternativas (SQLite+vec, cloud VDB, in-memory) | Hipótesis (creator) |
| `docs/operations/PILOT_PROGRAM.md:34-61` | Early adopter profile (durability, compilation, latency, local-first) | Hipótesis (creator) |
| `docs/Investigaciones/investigacion-equipo-2026-08-09.md:119-124` | 2 stars, 0 forks, 32 descargas, 0 dependents | **Evidencia dura de NO adopción** |
| `VantaDB_Manual_Estrategico_Unificado.md:1043` | "NADIE ha encontrado tu proyecto orgánicamente" | Evidencia dura de NO adopción |
| `.github/ISSUE_TEMPLATE/*.yml` | Perfil implícito del reportador (Python/CLI/server, OS) | Hipótesis (estructura) |

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** README.md (403L), docs/vision/VISION.md (242L), docs/strategy/GO_TO_MARKET.md (463L), docs/strategy/SHOW_HN_PREP.md (185L), docs/strategy/REDDIT_POSTS.md (105L), docs/operations/PILOT_PROGRAM.md (275L), docs/Investigaciones/investigacion-equipo-2026-08-09.md, VantaDB_Manual_Estrategico_Unificado.md (grep), CONTRIBUTING.md (286L), .github/ISSUE_TEMPLATE/*.yml, docs/plans/2026-08-16-wave-p20-tsys.md, docs/Backlog.md:500-518.
- **Referencias hacia dentro (del nuevo doc):** ninguno — `docs/Investigaciones/FND-24-icp-jtbd.md` es hoja de investigación (patrón FND-02/13/16 existentes).
- **Referencias entrantes:** solo `docs/Backlog.md:518` (NO se toca, lo actualiza el lead) y este task file.
- **Veredicto:** crear 1 doc nuevo en `docs/Investigaciones/` + task file. Sin riesgo de impacto en código.

## Steps

1. ✅ DISCOVERY — leer fuentes de posicionamiento y buscar evidencia de usuarios reales (DONE arriba)
2. ✅ ANALISIS — extraer señales → ICP (rol, stack, dolor, alternativa) + JTBD (funcional/emocional/social) con etiqueta evidencia/hipótesis
3. ✅ IMPLEMENTACION — crear `docs/Investigaciones/FND-24-icp-jtbd.md` con tabla de evidencia + sección de hipótesis + plan de validación
4. ✅ VERIFY — contrato mecánico: doc existe, ICP definido, JTBD ≥3, tabla evidencia→fuente (verificado 2026-08-16: doc_exists=True, icp_profiles=5, jobs=10, evidencia_table=True, hipotesis=True, plan=True)

## Contract (verificación mecánica)

```powershell
Test-Path docs/Investigaciones/FND-24-icp-jtbd.md   # → True
# ICP: sección "ICP" con rol/stack/dolor/alternativa
# JTBD: 3+ jobs (funcionales + emocionales + sociales)
# Tabla: claim → fuente (todas marcadas hipótesis o evidencia dura de no-adopción)
# Sección: "Hipótesis sin validar" + "Plan de validación"
```

## Protectos

- NO git add/commit (lead commitea)
- NO tocar: `docs/Backlog.md`, `AUD-024.md`, `verify-log.jsonl`, `completions/_vanta-cli.ps1`, `docs/plans/2026-08-16-wave-p20-tsys.md`
- NO campaign_update_task_state (instrucción del usuario)
- NO inventar evidencia; sin web search (no hay URL Show HN/comunidad con contenido real citable — Show HN no ocurrió)