# DEF-08: Install SLO + telemetría opt-in + fallback visible

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 15, Fase F1)
- **Fuente:** Backlog `P55` fila `DEF-08` (L943) + plan Task 15
- **Esfuerzo:** 🟢 1d · **Prioridad:** 🟠 · **Tipo:** Docs (+ FIND si la instrumentación excede el día)
- **Turns estimados:** 5-10
- **Creado:** 2026-09-26 · **last-synced:** 2026-09-26
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0 abiertas (instrumento ya existe: fichas FASE-A) · **Pendientes (downhill):** 4 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/dev/FASE-A.md` (§1 fichas ya miden tiempo hasta instalación/primer resultado; §2 checklist del gate), `docs/dev/strategy/DISTRIBUTION.md` §4 (checklist de anuncio → ítem 6 "monitor error rate/P95"), `docs/user/QUICKSTART.md` §7 (nota de fallback), `SPEC.md` (riesgo admitido F6 / Success Criteria #1), `setup-embeddings.ps1` (wizard — lugar natural del mensaje), `src/llm.rs` (comportamiento real del fallback) |
| Callees | FASE-A instrument (fichas humanas), `embeddings/verify.py` + `embeddings/manifest.json` (estado del modelo), `src/llm.rs:341-412` (pre-check dylib + dummy fallback, FIND-100), DISTRIBUTION §2b thresholds (monitoreo post-anuncio) |
| Implicaciones | Docs-first con instrumentación mínima. Código solo si ≤1d: (a) evento de telemetría **local** first_recall, (b) mensaje CLI de fallback → si excede, FIND. Telemetría = datos → review de privacidad obligatoria en P2-01. No cambia contratos de API ni performance. |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `docs/user/QUICKSTART.md` (268 L), `docs/dev/strategy/DISTRIBUTION.md` (134 L), `docs/dev/FASE-A.md` (106 L), `SPEC.md` (122 L), `setup-embeddings.ps1` (≥100 L), `src/llm.rs` (grep dirigido + líneas 341-412/1078-1100)
- **Archivos referenciados hacia dentro:** `embeddings/download.py --check` (contrato EMB-11, invocado por el wizard L73-78), `embeddings/verify.py --check` (verificación CI-friendly, QUICKSTART L252), manifest de modelos
- **Archivos que referencian a los editados (grep):** research P55 §6 (FIND-100 / EMB-10..18), `SPEC.md` Boundaries L74 ("fallback avisado (`fallback:true`), nunca silencioso"), SPEC F6 (L24), master Task 15
- **Veredicto impacto:** bajo (docs) / medio si Step 4 ejecuta mensaje CLI (toca wizard/CLI → worker + test; stop condition del master lo acota).

## Contrato
"SLO definido (tiempo al primer recall + tasa de éxito) con método de medición Y telemetría opt-in especificada (privacy-first, default off) Y fallback visible especificado (mensaje + docs) — secciones verificables por `rg` en `DISTRIBUTION.md` y `QUICKSTART.md` Y `pwsh scripts/validate-docs-coverage.ps1` exit 0"

## Spec (SDD — decisiones de producto/doc/instrumentación)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Números SLO | A: TTFR ≤30 min + tasa de éxito ≥80% (4/5 fichas) / B: números más laxos "a definir" | A | ✅ decidido-por-evidencia (`SPEC.md:101` Success #1 "<30 min"; FASE-A mide 5 personas) — owner confirma en Review |
| 2 | Instrumento de medición | A: fichas FASE-A §1 (campos ya existentes: "Tiempo hasta instalación"/"Tiempo hasta primer resultado") / B: telemetría automática de red | A primero; B solo opt-in después | ✅ decidido-por-evidencia (`FASE-A.md:35,40`; cero código) |
| 3 | Telemetría | A: local-only default + opt-in anónimo remoto / B: default-on remoto / C: sin telemetría | A (red default OFF: `install_completed`, `first_recall`, `fallback_used`; sin contenido/keys/PII; consent explícito) | ✅ decidido-por-evidencia (research §6 + pre-mortem Task 15: "telemetría default-on → PROHIBIDO") |
| 4 | Fallback visible | A: mensaje claro en wizard/CLI + nota en QUICKSTART + remediación / B: solo logs | A; si el mensaje CLI requiere >1d de código → spec + FIND (stop condition master) | ✅ decidido-por-evidencia (`src/llm.rs:341-364` ya emite fallback con flag; falta UX visible) |

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** "fallback avisado, nunca silencioso" (SPEC L74) — la spec no puede proponer silencio; telemetría **default OFF** y sin PII; FASE-A sigue siendo el instrumento humano del gate (no reemplazarlo); NO tocar el comportamiento de `src/llm.rs` (dummy fallback verde, FIND-100; test `f100_incompatible_dylib_never_panics`).
- **Comandos de verificación:** `rg -n "SLO|first recall|primer recall" docs/dev/strategy/DISTRIBUTION.md` (hits) · `rg -n "fallback" docs/user/QUICKSTART.md` (hit nuevo) · `pwsh scripts/validate-docs-coverage.ps1` exit 0 · si se toca código: test del mensaje (worker)
- **Deuda pendiente:** ninguna (si la instrumentación excede 1d → FIND abierto con spec exacta, no código a medias)

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← valor en este task file |
|------------------------|----------------------------|
| `activeGoal` | DEF-08 — Install SLO + telemetría opt-in + fallback visible |
| `lastAction` | Task file creado; SLO anclado a SPEC #1 + fichas FASE-A; privacy-first fijado |
| `result` | ⬜ pending (`OK` al completar) |
| `nextAction` | Step 1 — SLO en `DISTRIBUTION.md` (números + método) |
| `contract` | §Contrato + §Invariantes (verificación: rg anchors + validate-docs-coverage) |
| `nextTask` | Fin de F1 → F2 (WIRE-02, Task 16) |

```
=== RECITATION ===
Objetivo activo: DEF-08 — Install SLO + telemetría opt-in + fallback visible
Estado: pending
Última acción: task file creado (2026-09-26)
Resultado: —
Próxima acción: Step 1 — SLO en DISTRIBUTION.md
Contrato: ver ## Contrato (SLO + telemetría opt-in + fallback visible)
Invariantes: fallback nunca silencioso · telemetría default OFF sin PII · no tocar llm.rs
Deuda: ninguna
Próxima tarea si completa: WIRE-02 (F2)
last-synced: 2026-09-26
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda (docs + instrumentación mínima; si la instrumentación no cabe en 1d → FIND explícito, no deuda oculta).

## Definition of Done (contrato multi-nivel — P2-08)
- **Task:** contrato ✅ + las 3 piezas (SLO / telemetría / fallback visible) verificables por rg + validate-docs-coverage 0 gaps
- **Commit:** atómico `docs:` + `(DEF-08)`; si hay código de mensaje CLI: commit separado con test
- **Release:** N/A aplicable — specs de producto; el SLO se consume en FASE-A (owner-side) y el anuncio (F6)

## Herramientas necesarias
- `rg`/grep, `pwsh` (validate-docs-coverage), lectura de `src/llm.rs`/wizard, `campaign_*`
- **Skills cargadas (SDP v3, pre-run 2026-09-27):** `campaign-executor` · `progreso` · `writing-guidelines` · `writing-plans` · `incremental-implementation` · `test-driven-development` · `context-engineering` · `source-driven-development` · **PINNED (policy):** ninguno
  - Descartadas del SDP (no aplican): `frontend-ui-engineering`, `test-driven-development`, `context-engineering`, `api-and-interface-design`

## Investigation Notes
- FASE-A.md ya mide el SLO sin código nuevo: ficha §1 campos "Tiempo hasta instalación" / "Tiempo hasta primer resultado" (L35; regla "reloj, no estimación" L46) + 5 fichas = tasa de éxito. El SLO debe DEFINIR el umbral, no inventar otro instrumento.
- `SPEC.md:101` (Success #1): "1 comando → recuerdo guardado y recuperado por sinónimo en su agente (<30 min, sin compilar ni clonar)" → ancla del TTFR.
- El fallback YA existe: `src/llm.rs:341-412` pre-check de dylib sin panic + dummy con `fallback = true` + mensaje "ONNX Runtime dylib unusable; using deterministic dummy embeddings" (L364); test `f100_incompatible_dylib_never_panics` (L1087). Falta: visibilidad UX para el usuario final + doc (SPEC L74 lo exige como contrato).
- `setup-embeddings.ps1` setea `ORT_DYLIB_PATH` (L11-14) y verifica con `python embeddings/download.py --check` (L73-78) → lugar natural del mensaje de fallback y de la remediación documentada.
- `DISTRIBUTION.md` §4 ítem 6: "monitor error rate / P95 per plan §2b thresholds" → la telemetría opt-in se especifica ahí (contexto del gate de anuncio).
- Historial: FIND-100 + EMB-10..18 en un solo release (research P55 §6) — la promesa central es el componente más frágil; el SLO convierte "cero config" en medible.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 4 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — APLICA PARCIAL: telemetría = potencial dato de usuario → especificar privacy-first (opt-in, default OFF, sin contenido/keys/PII, almacenamiento local por defecto); cargar `security-and-hardening` al ejecutar y dejar hallazgos en Notas; el Review P2-01 debe validar el diseño de privacidad.
- [x] **PERFORMANCE** — N/A: sin hot path; el evento local (si se instrumenta) es fuera del camino de búsqueda/ingesta.

## Steps

### Step 1: SLO de instalación en `DISTRIBUTION.md`
- **Archivos:** `docs/dev/strategy/DISTRIBUTION.md`
- **Acción:** sección nueva "Install SLO — Fase A gate": TTFR (primer recall) ≤30 min y tasa de éxito ≥80% (4/5 fichas), definiciones exactas de los dos timestamps (inicio de instalación / primer recall con éxito), método de medición = fichas FASE-A §1 + condición GO/NO-GO del anuncio.
- **Verify:** sección presente con números + método; `rg -n "SLO|primer recall|first recall" docs/dev/strategy/DISTRIBUTION.md` → hits; ninguna métrica sin fuente (Regla 11).
- **Estado:** ⬜ PENDING

### Step 2: Especificación de telemetría opt-in (privacy-first)
- **Archivos:** `docs/dev/strategy/DISTRIBUTION.md` (sección junto al SLO / §4 ítem 6)
- **Acción:** eventos `install_completed`, `first_recall`, `fallback_used`; default **OFF** con red; log local primero; transmisión anónima solo con consent explícito; lista explícita de lo que NO se recolecta (contenido de memoria, keys, paths, prompts); sin identificadores estables por defecto. Registrar en Notas la nota de privacidad para Review.
- **Verify:** spec con eventos + default OFF explícito + 0 PII; `rg -n "opt-in|default off" docs/dev/strategy/DISTRIBUTION.md` → hit.
- **Estado:** ⬜ PENDING

### Step 3: Fallback visible (mensaje + docs)
- **Archivos:** `docs/user/QUICKSTART.md` (§7, nota de troubleshooting), `docs/dev/strategy/DISTRIBUTION.md` (referencia)
- **Acción:** spec del mensaje visible al usuario cuando ORT/modelo no cargan (dummy + `fallback:true`; qué verá, cómo se corrige: re-ejecutar `setup-embeddings.ps1` / `python embeddings/verify.py --check`); agregar la nota en QUICKSTART referenciando `src/llm.rs` (comportamiento existente) — sin tocar código en este step.
- **Verify:** nota presente en QUICKSTART (`rg -n "fallback" docs/user/QUICKSTART.md`); remediación cita comandos existentes; 0 claims nuevos sin fuente.
- **Estado:** ⬜ PENDING

### Step 4: Cierre + FIND si la instrumentación excede docs
- **Archivos:** `docs/` + (si aplica) Backlog fila FIND
- **Acción:** evaluar si el mensaje CLI visible / evento local caben en ≤1d; si sí → delegar a worker como slice acotado; si no → fila FIND con la spec exacta (evento + mensaje) sin ejecutar código a medias. Commit `docs: install SLO + telemetry/fallback specs (DEF-08)`.
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` exit 0; `npx markdownlint-cli2 "docs/dev/strategy/DISTRIBUTION.md" "docs/user/QUICKSTART.md"` 0 errores; FIND registrado o "docs-only complete" en Notas.
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna dura. **DEF-05 (soft):** relación con Success Criteria/North Star (coherencia de vocabulario).
- Consume FASE-A (owner-side) como instrumento de medición — sin bloquear: el SLO se define ahora y se mide cuando FASE-A corra.
- Siguiente: F2 → WIRE-02 (Task 16).

## Review (GATE — agente distinto, P2-01)
- **Revisor:** [PENDIENTE al ejecutar — `vanta-audit`/`vanta-review`, agente distinto al implementador]
- **Enfoque:** ¿SLO medible con lo que FASE-A ya captura? ¿telemetría respeta privacy-first? ¿el fallback spec nunca permite silencio?
- **Cómo se probó:** [a poblar: re-ejecución de rg + lectura adversarial del diseño de telemetría]
- **Checklist anti-hábitos tóxicos:** según plantilla (especial atención: no degradar chequeo en paths de datos personales)
- **Veredicto:** [PENDIENTE — ✅ approve | ❌ cambios requeridos]

## Notas
- Stop condition (master Task 15): si la instrumentación requiere código >1d → solo spec + FIND.
- Pre-mortem F1 del master: telemetría default-on → **PROHIBIDO** (opt-in únicamente).
- No modificar `src/llm.rs` (fallback ya implementado y testeado, FIND-100); el delta es visibilidad + doc.
- Doc técnico (DISTRIBUTION/QUICKSTART) en inglés; este task file en español.
- Commit `docs:` atómico; push owner-gated.
