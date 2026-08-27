# TASK QW-6: decisión letta — README declara estado experimental

## Metadata
- **Plan file:** `docs/plans/2026-08-25-integrations-research-wins.md`
- **Fuente:** Wave 2 QW-6 (H-08)
- **Esfuerzo:** 🟢 0.5h
- **Prioridad:** Wave 2 — Limpieza / dedup
- **Tipo:** docs
- **Turns estimados:** 2-3
- **Creado:** 2026-08-27T16:45
- **last-synced:** 2026-08-27T16:45
- **Estado:** ✅ COMPLETED
- **Ruta:** vanta-docs
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `integrations/letta/README.md` (consumido por PyPI, docs site, `integrations/*/README.md` Why VantaDB pattern), `docs/plans/2026-08-25-integrations-research-wins.md` Wave 2 reference |
| Callees | Ninguno — archivo markdown standalone, sin imports, sin código ejecutable |
| Implicaciones | contrato no cambia API pública; README declara experimental + por qué (Letta stateful, sin contrato vector-store público). Decisión mínima lazy: documentar experimental (borrar solo si Letta confirma incompatibilidad). Sin impacto en `vantadb/src/*`, sin paths multi-índice/dashmap/parking_lot/Tokio → no auditoría concurrencia (Regla 8). No hot path → no perf bench (Regla 9). No trust boundary → no security hardening. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `integrations/letta/README.md` (57 líneas completas)
  - `integrations/letta/vantadb_letta/vectorstore.py` (205 líneas completas)
  - `integrations/letta/vantadb_letta/__init__.py` (completo)
  - `integrations/letta/pyproject.toml` (completo)
  - `docs/plans/2026-08-25-integrations-research-wins.md` (Wave 2 QW-6, líneas 49-54)
  - `.opencode/task-system/prompts/pipeline-full.md` (instrucciones canónicas)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):**
  - `integrations/letta/README.md` → standalone markdown, referencia `vantadb_letta.VantaDBVectorStore` solo en ejemplo, no import real
  - `integrations/letta/vantadb_letta/vectorstore.py` → `import vantadb_py as vanta` (PyO3), `uuid`, `typing` — no tocado por esta tarea (doc-only)
- **Archivos que referencian a los editados (referencias entrantes):**
  - `docs/plans/2026-08-25-integrations-research-wins.md` — QW-6 (H-08) referencia `integrations/letta/README.md`
  - `integrations/letta/tests/test_vectorstore.py` — no referencia README (tests solo vectorstore.py)
  - `integrations/*/README.md` ×9 — patrón Why VantaDB compartido, letta sigue mismo patrón + sección experimental
  - Ningún módulo Rust core, web/, u otro adapter depende de este README (grep confirma aislamiento)
- **Veredicto impacto:** mínimo — impacto localizado a `integrations/letta/README.md`, seguro para edit. Fix ya en HEAD (96e143ec), verify-only. No toca código ejecutable, no requiere auditoría concurrencia/perf/security.

## Spec

N/A — doc-fix con contrato mecánico (Wave 2 QW-6). No agrega símbolos públicos nuevos; solo declara estado experimental en README.

Problema: Letta es plataforma stateful con memoria propia y sin contrato público de vector-store; el adapter VantaDB×Letta no tiene garantía de compatibilidad estable. Sin declaración explícita, usuarios asumen soporte oficial y reportan bugs de contrato inexistente.

Criterio: `integrations/letta/README.md` contiene `## Status: experimental` (case-insensitive grep `experimental`) + párrafo explicativo que menciona "stateful platform with its own memory layer and no public vector-store contract" (o traducción equivalente ES: "plataforma stateful con memoria propia — sin contrato de vector-store público") + nota de que no es integración oficial soportada y API puede cambiar.

Alcance: `integrations/letta/README.md:34-40` (sección Status: experimental, 7 líneas).

Decisiones: documentar experimental como decisión mínima lazy (ponytail rung 1: mínimo que satisface contrato). Alternativa "retirar adapter" descartada — Letta no confirmó incompatibilidad, solo ausencia de contrato público. Inglés en README (fuente de verdad técnica), español solo en plan/Backlog (Doc Language Split). Sin ADR separado — decisión reversible, teardown trivial (borrar directorio).

## Contrato

```
grep -i "experimental" integrations/letta/README.md  # debe matchear "## Status: experimental"
grep -i "stateful platform" integrations/letta/README.md  # debe matchear justificación
cat integrations/letta/README.md | grep -A5 "experimental"  # verifica párrafo completo
```

Verificación mecánica:
1. `Select-String -Path integrations/letta/README.md -Pattern "experimental"` → 1 match en línea 34 `## Status: experimental` ✅
2. `Select-String -Path integrations/letta/README.md -Pattern "stateful platform"` → 1 match en línea 36 `Letta is a stateful platform...` ✅
3. `git show 96e143ec:integrations/letta/README.md` → sección experimental ya en HEAD (no diff pendiente) ✅

## Herramientas

Skills cargadas: `documentation-and-adrs`, `ponytail` (+ SDP sin candidatos adicionales)

- **documentation-and-adrs:** documentar decisión experimental, cuándo escribir ADR vs nota en README, lifecycle PROPOSED→ACCEPTED (Regla 5). Para QW-6, README es suficiente — ADR solo si decisión cambia a retiro (ver Spec Decisiones).
- **ponytail (full):** ladder mínimo — rung 1 YAGNI check: ¿retirar adapter? No, sin evidencia de incompatibilidad (Letta no confirma). Rung 2 reuse: patrón Why VantaDB ya en 9 READMEs (96e143ec). Mínimo diff: 7 líneas markdown, sin código, sin deps nuevas.

**SKILLS_CARGADAS (SDP):** documentation-and-adrs, ponytail + SDP sin candidatos adicionales
Lifecycle mapping: DEFINE (documentation-and-adrs) + BUILD (ponytail minimal) + VERIFY (grep experimental)
Grep SKILLS-MANIFEST.md por `letta|experimental|stateful|vector-store|README|integration` → sin skill directa; `writing-guidelines` considerada pero no añade valor para grep mecánico `experimental` + párrafo ya cumple voz/tone (inglés técnico, sin claims Regla 11); `spec-driven-development` no aplica (doc-fix, no feature-add sin spec). `vantadb` skill es guía genérica de uso, no doc-authoring.
Discovery ≤8 skills → 2 cargadas, justificadas arriba. Base ≤6 de campaign_load_skills + 0 adicionales = 2 total < 8.

## Steps

### Step 1: DISCOVERY — verificar fix ya aplicado y blast radius
- **Archivos:** `integrations/letta/README.md`, `integrations/letta/vantadb_letta/vectorstore.py`, `docs/plans/2026-08-25-integrations-research-wins.md:49-54`
- **Acción:** Confirmar que fix QW-6 (96e143ec) ya está en disco: `## Status: experimental` + párrafo "Letta is a stateful platform with its own memory layer and no public vector-store contract...". Mapear Regla 0 completa arriba. Validar que `git diff HEAD -- integrations/letta/README.md` vacío (no pendiente). Verificar pyproject.toml no requiere bump (doc-only).
- **Verify:** `git show 96e143ec --stat | grep letta/README` → 19 ++; `Select-String experimental` → 1 match línea 34; `Select-String "stateful platform"` → 1 match línea 36; `git diff HEAD -- integrations/letta/README.md` → vacío (fix ya en HEAD) ✅
- **Estado:** ✅ DONE (2026-08-27 — fix ya en HEAD 96e143ec; Regla 0 mapeada; 57 líneas README verificadas)

### Step 2: VERIFY — contrato grep experimental + justificación stateful
- **Archivos:** `integrations/letta/README.md:34-40`
- **Acción:** Ejecutar verificación mecánica del contrato: grep case-insensitive `experimental` (debe matchear), grep `stateful platform` + `memory layer` + `no public` (justificación estado experimental por qué), verificar sección completa 7 líneas incluye "community convenience, not officially supported" + "API may change" + "Prefer Letta's native memory...". Validar Inglés como fuente de verdad (Regla Doc Language Split).
- **Verify:** `Select-String -Path integrations/letta/README.md -Pattern "experimental"` → 34: ## Status: experimental ✅; `Select-String -Pattern "stateful platform"` → 36: Letta is a stateful platform... ✅; `Get-Content integrations/letta/README.md | Select-String -Pattern "no public"` → match ✅; `Get-Content README.md:34-40` → 7 líneas correctas ✅
- **Estado:** ✅ DONE (2026-08-27 — contrato QW-6 pasa: experimental + stateful + no public vector-store contract)

### Step 3: CIERRE — verify full + no-commit + recitation + progreso
- **Archivos:** `.opencode/skills/campaign-executor/tasks/QW-6.md`, `docs/plans/2026-08-25-integrations-research-wins.md`
- **Acción:** No commit por instrucción usuario (Reglas: no commit). Actualizar task file con steps ✅ y Context Save Point; actualizar plan file con recitation QW-6 si corresponde (sin commit); ejecutar skill progreso (dry-run, no push); handoff.
- **Verify:** task file steps ✅ 3/3, `git diff HEAD -- integrations/letta/` vacío (fix ya en commit 96e143ec), no commit por instrucción explícita; `campaign_update_task_state` → completed con recitation; `campaign_memory_write` innecesario (doc-only verify-only, sin lección nueva)
- **Estado:** ✅ DONE (2026-08-27 — verify: grep experimental línea 34 + stateful plataforma línea 36, diff vacío ya en HEAD; no commit por regla usuario; task file 3/3 DONE)

## Dependencias
- Ninguna (Wave 2 QW-6 independiente; QW-4/QW-5 disjuntos en adapters distintos; fix ya en HEAD 96e143ec junto a QW-1..5, QW-8)

## Notas
- Fix ya merged en 96e143ec (19 ++ en `integrations/letta/README.md:34-40` — sección `## Status: experimental` + 5 líneas justificación). Esta tarea pipeline-full re-verifica contrato mecánico (`grep experimental`) y documenta por qué (Letta stateful, sin contrato público). Si no hace falta código adicional, cierra como verify-only con evidencia (patrón QW-1..QW-3).
- Ponytail: 7 líneas markdown, sin código, sin ADR separado (decisión reversible, teardown = borrar dir). Skipped: ADR formal `docs/architecture/adr/NNN_letta-experimental.md` — add when Letta confirma incompatibilidad o adapter se retira (requiere Context/Decision/Consequences tabla humana). Skipped: retirar adapter entero — add when Letta documenta contrato vector-store público incompatible.
- Regla 11 (Claims performance): README no contiene claims de performance sin benchmark — sección Why VantaDB usa descriptivos ("embedded & local-first", "no server") no numéricos, cumple Regla 11. Regla 3 (Doc sync): README actualizado en mismo PR que vectorstore (96e143ec) — cumple.
- Inglés en README (source of truth técnica): "Letta is a stateful platform..." — español del contrato del plan ("Letta es plataforma stateful...") es traducción planning, no debe copiarse literal al README.

## Context Save Point
- **Fecha:** 2026-08-27T16:45
- **Branch:** develop
- **CI pendiente:** no — grep experimental + stateful plataforma verificados en HEAD 96e143ec (3/3 grep ✅)
- **Decisiones:** mantener fix actual (7 líneas Status: experimental líneas 34-40), no añadir ADR formal ni retirar adapter; minimal doc wins (ponytail rung 1); inglés en README, español en plan
- **Problemas conocidos:** ninguno — contrato QW-6 ✅ COMPLETO, diff vacío (verify-only), no commit por instrucción usuario
- **Próxima tarea:** QW-7
