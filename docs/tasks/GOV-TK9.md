# GOV-TK9: Verificar URL `vantadb-examples` del checklist

## Metadata
- **Plan file:** `docs/plans/2026-09-03-quality-gtm-wave.md` (Wave0 — Task 3)
- **Creado:** 2026-09-03
- **last-synced:** 2026-09-03
- **Estado:** ⏳ IN PROGRESS → ✅ COMPLETED 2026-09-03 (commit `bd22f387`, campaign state `completed`)
- **Tipo auto-detectado:** `docs` (Documentation) — sin `## Spec` requerida (no es feature-add)
- **SDP:** campaign-executor, progreso, ponytail, coordinated-web-search, documentation-and-adrs, writing-guidelines, writing-plans (keywords: checklist, onboarding, docs, url; manifest grep sin candidatos extra → base + lifecycle)
- **Research Digest:** ruta correcta del checklist es `docs/operations/pilot-onboarding-checklist.md:51` (el Backlog fila 436 cita ruta vieja `pilot-onboarding-checklist.md` — corregir al cerrar). Organización verificada: `ness-e` (FIND-17).

## Gate D (question-gates.md) — evaluado, NO disparado
- Blast radius: 1 archivo docs, 1 línea. Sin hot path, sin API pública, sin símbolos `pub` nuevos, sin spec faltante (docs-fix, no feature-add).
- Veredicto: proceder sin `question` al usuario.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:**
  - `docs/operations/pilot-onboarding-checklist.md` (124 líneas — la línea 51 es `| 2.5 | Clone or download example repo | \`git clone https://github.com/vantadb/vantadb-examples\` | ☐ |`)
  - `docs/plans/2026-09-03-quality-gtm-wave.md` Task 3 (líneas 76-88)
  - `docs/Backlog.md:436` (fila GOV-TK9 — cita ruta vieja sin `docs/operations/`)
- **Referencias hacia fuera (qué referencia el checklist):** ninguna — es hoja (doc enterprise, no importado por código).
- **Referencias entrantes (quién cita `vantadb-examples`):**
  - `rg -n "vantadb-examples" docs/` → solo 3 hits no-clone: Backlog fila 436 (tracking), plan Task 3 (contrato/meta), archivo `docs/plans/archive/2026-08-28-backlog-triage.md:633` (histórico). Ningún otro doc usuario (`docs/README*`, `docs/api/*`, `docs/operations/*`) referencia el repo → el único paso roto de cara al piloto es checklist:51.
- **Veredicto de impacto:** cambio de 1 línea en 1 doc + fila Backlog al cerrar. Sin código, sin BENCHMARKS.md, sin `src/`. Reversible (`git revert`).

## Contrato (ley)
- `webfetch https://github.com/vantadb/vantadb-examples` → 404 (verificado live hoy) → rama 404 del contrato.
- Tras fijar: `rg -n "vantadb/vantadb-examples" docs/operations/ docs/api/ docs/QUICKSTART.md docs/README*` == 0 (cero URLs clone muertas en docs de cara al usuario; plan/backlog/archive son tracking histórico, no pasos del piloto).
- Si `ness-e/vantadb-examples` tampoco existe (verificado live: también 404) → NO crear repo desde el agente; dejar `[TODO humano: crear repo]` explícito en la línea.
- **Stop condition:** si se decide crear el repo → fuera de scope, hallazgo (no alcanzado: nadie pidió crearlo).

## Evidencia live (coordinated-web-search, capa keyless/webfetch)
- `GET https://github.com/vantadb/vantadb-examples` → **404** (org `vantadb` inexistente / repo ausente).
- `GET https://github.com/ness-e/vantadb-examples` → **404** (owner real `ness-e` verificado 200 con 4 repos: Vantadb, ness-e, portfolio, roadmap-retos-programacion — ninguno es `vantadb-examples`).
- Conclusión: ningún repo existe en ninguna org → rama TODO-humano del contrato.

## Steps
### Step 1: Corregir checklist:51 con TODO humano explícito
- **Archivos:** `docs/operations/pilot-onboarding-checklist.md:51`
- **Acción:** reemplazar el paso clone muerto por comentario honesto que no da error al piloto + TODO etiquetado para el humano (crear repo o apuntar al real cuando exista). Inglés (doc técnica = fuente de verdad en inglés).
- **Verify:** `rg -n "vantadb/vantadb-examples" docs/operations/` == 0
- **Estado:** ✅ COMPLETED (2026-09-03 — `rg` en `docs/operations/` + `docs/api/` = 0 hits; TODO-humano explícito en línea 51)

### Step 2: Cierre (plan + backlog + commit + progreso)
- **Archivos:** `docs/plans/2026-09-03-quality-gtm-wave.md` (Task 3 → ✅), `docs/Backlog.md` (fila 436 se ELIMINA vía skill progreso Trigger 1 — con eso muere también la cita a la ruta vieja)
- **Acción:** commit `docs(gov): verificar URL vantadb-examples (GOV-TK9)` solo con archivos de esta tarea; skill progreso (registro en `docs/avance/activo/operaciones.md`)
- **Verify:** `git log --oneline -1` muestra el commit; `rg -n "GOV-TK9" docs/Backlog.md` == 0
- **Estado:** ⬜ PENDING → ✅ COMPLETED (commit `bd22f387`)

## Dependencias
- Ninguna (Wave0 paralelo con RES-07 y GOV-TK1 — disjuntos: config.rs/BENCHMARKS y cli.rs vs este doc).

## Notas
- No tocar `src/` ni `docs/operations/BENCHMARKS.md` (otros waves).
- Idioma: la línea fijada va en inglés (Doc Language Split: docs/operations = English source of truth).

## Context Save Point
- **Fecha:** 2026-09-03
- **Branch:** develop
- **CI pendiente:** no (cambio docs-only de 1 línea)
- **Decisiones:** TODO-humano en vez de apuntar a `ness-e/vantadb-examples` porque ese repo tampoco existe (404 live) — apuntar a una URL 404 distinta sería repetir el bug.
- **Problemas conocidos:** ninguno
- **Próxima tarea:** Wave0 resto (RES-07, GOV-TK1 corren en paralelo por otros agentes)
