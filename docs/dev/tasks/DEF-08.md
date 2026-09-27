# DEF-08: Install SLO + telemetría opt-in + fallback visible

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 15, Fase F1)
- **Fuente:** Backlog `P55` fila `DEF-08` (L943) + plan Task 15
- **Esfuerzo:** 🟢 1d · **Prioridad:** 🟠 · **Tipo:** Docs (+ FIND si la instrumentación excede el día)
- **Turns estimados:** 5-10
- **Creado:** 2026-09-26 · **last-synced:** 2026-09-27
- **Estado:** ✅ COMPLETED (2026-09-27 — review fresco P2-01 ✅ + fixes O1/O2/O3/N1/N2; commit LEAD)
- **Incógnitas (uphill):** 0 abiertas (instrumento ya existe: fichas FASE-A) · **Pendientes (downhill):** 0 (4/4 ✅ — review fresco ✅; commit LEAD)

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
| `lastAction` | Steps 1-4 ejecutados: DISTRIBUTION §5 SLO + §6 telemetría + §7 fallback spec; QUICKSTART §7 nota; coverage 0 gaps; markdownlint 0 issues |
| `result` | OK (review fresco ✅; fixes O1/O2/O3/N1/N2; commit LEAD) |
| `nextAction` | LEAD: commit `docs: install SLO + telemetry/fallback specs (DEF-08)` |
| `contract` | §Contrato + §Invariantes (verificación: rg anchors ✅ + pwsh scripts/validate-docs-coverage.ps1 exit 0 ✅) |
| `nextTask` | Fin de F1 → F2 (WIRE-02, Task 16) |

```
=== RECITATION ===
Objetivo activo: DEF-08 — Install SLO + telemetría opt-in + fallback visible
Estado: completed (review fresco ✅; commit LEAD)
Última acción: docs editados (DISTRIBUTION §5-§7 + QUICKSTART §7); coverage 0 gaps; markdownlint 0 issues
Resultado: OK (review fresco ✅ + fixes O1/O2/O3/N1/N2)
Próxima acción: LEAD — commit docs: install SLO + telemetry/fallback specs (DEF-08)
Contrato: rg SLO/opt-in/fallback ✅ + pwsh scripts/validate-docs-coverage.ps1 exit 0 ✅
Invariantes: fallback nunca silencioso · telemetría default OFF sin PII · no tocar llm.rs
Comandos de verificación: rg anchors ✅ · coverage exit 0 ✅ · markdownlint 0 ✅
Deuda: code-follow-up especificado (fallback notice UX ≤1d — LEAD decide worker/FIND)
Próxima tarea si completa: WIRE-02 (F2)
last-synced: 2026-09-27
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
| Pendientes de ejecución (downhill) | 0 steps (4/4 ✅) |
| % completado | 100% — review fresco ✅; commit LEAD |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — APLICA PARCIAL: telemetría = potencial dato de usuario → especificar privacy-first (opt-in, default OFF, sin contenido/keys/PII, almacenamiento local por defecto); cargar `security-and-hardening` al ejecutar y dejar hallazgos en Notas; el Review P2-01 debe validar el diseño de privacidad.
- [x] **PERFORMANCE** — N/A: sin hot path; el evento local (si se instrumenta) es fuera del camino de búsqueda/ingesta.

## Steps

### Step 1: SLO de instalación en `DISTRIBUTION.md`
- **Archivos:** `docs/dev/strategy/DISTRIBUTION.md`
- **Acción:** sección nueva "Install SLO — Fase A gate": TTFR (primer recall) ≤30 min y tasa de éxito ≥80% (4/5 fichas), definiciones exactas de los dos timestamps (inicio de instalación / primer recall con éxito), método de medición = fichas FASE-A §1 + condición GO/NO-GO del anuncio.
- **Verify:** sección presente con números + método; `rg -n "SLO|primer recall|first recall" docs/dev/strategy/DISTRIBUTION.md` → hits; ninguna métrica sin fuente (Regla 11).
- **Estado:** ✅ COMPLETED (2026-09-27) — §5 "Install SLO — Fase A gate": TTFR ≤30 min + ≥4/5 fichas, timestamps t0/t1/t2 definidos, método = fichas FASE-A, GO/NO-GO; `rg` ✅ (hits L139-163)

### Step 2: Especificación de telemetría opt-in (privacy-first)
- **Archivos:** `docs/dev/strategy/DISTRIBUTION.md` (sección junto al SLO / §4 ítem 6)
- **Acción:** eventos `install_completed`, `first_recall`, `fallback_used`; default **OFF** con red; log local primero; transmisión anónima solo con consent explícito; lista explícita de lo que NO se recolecta (contenido de memoria, keys, paths, prompts); sin identificadores estables por defecto. Registrar en Notas la nota de privacidad para Review.
- **Verify:** spec con eventos + default OFF explícito + 0 PII; `rg -n "opt-in|default off" docs/dev/strategy/DISTRIBUTION.md` → hit.
- **Estado:** ✅ COMPLETED (2026-09-27) — §6 "Telemetry — opt-in, privacy-first": 3 eventos, default OFF, log local primero, no IDs estables, deny list explícita; `rg` ✅ (hits L177-194)

### Step 3: Fallback visible (mensaje + docs)
- **Archivos:** `docs/user/QUICKSTART.md` (§7, nota de troubleshooting), `docs/dev/strategy/DISTRIBUTION.md` (referencia)
- **Acción:** spec del mensaje visible al usuario cuando ORT/modelo no cargan (dummy + `fallback:true`; qué verá, cómo se corrige: re-ejecutar `setup-embeddings.ps1` / `python embeddings/verify.py --check`); agregar la nota en QUICKSTART referenciando `src/llm.rs` (comportamiento existente) — sin tocar código en este step.
- **Verify:** nota presente en QUICKSTART (`rg -n "fallback" docs/user/QUICKSTART.md`); remediación cita comandos existentes; 0 claims nuevos sin fuente.
- **Estado:** ✅ COMPLETED (2026-09-27) — QUICKSTART §7 "If ONNX Runtime or the model does not load" (2 triggers + remediación `setup-embeddings.ps1` / `verify.py --check`); spec del mensaje en DISTRIBUTION §7; `rg` ✅ (hit L265)

### Step 4: Cierre + FIND si la instrumentación excede docs
- **Archivos:** `docs/` + (si aplica) Backlog fila FIND
- **Acción:** evaluar si el mensaje CLI visible / evento local caben en ≤1d; si sí → delegar a worker como slice acotado; si no → fila FIND con la spec exacta (evento + mensaje) sin ejecutar código a medias. Commit `docs: install SLO + telemetry/fallback specs (DEF-08)`.
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` exit 0; `npx markdownlint-cli2 "docs/dev/strategy/DISTRIBUTION.md" "docs/user/QUICKSTART.md"` 0 errores; FIND registrado o "docs-only complete" en Notas.
- **Estado:** ✅ COMPLETED (2026-09-27) — docs-only complete; coverage exit 0 (0 gaps) ✅; markdownlint 0 issues ✅; code-follow-up ≤1d especificado en Notas (LEAD decide worker/FIND)

## Dependencias
- Ninguna dura. **DEF-05 (soft):** relación con Success Criteria/North Star (coherencia de vocabulario).
- Consume FASE-A (owner-side) como instrumento de medición — sin bloquear: el SLO se define ahora y se mide cuando FASE-A corra.
- Siguiente: F2 → WIRE-02 (Task 16).

## Review (GATE — agente distinto, P2-01)
- **Revisor:** `vanta-review` fresco (sesión `ses_f1b51d51cffekZkFitR0oKhxP6`; ≠ autor) — **✅ APPROVE** (2026-09-27) — 0 Critical / 0 Required; 3 Optional (O1 SLO-proxy → fix aplicado; O2 correlación server-side → fix aplicado; O3 semántica local/consent → fix aplicado + pin en rule 6) + 3 Nits (N1/N2 aplicados; N3 evidencia).
- **Enfoque:** ¿SLO medible con lo que FASE-A ya captura? ¿telemetría respeta privacy-first? ¿el fallback spec nunca permite silencio?
- **Cómo se probó (evidencia del autor, para el revisor):** `rg -n "SLO|primer recall|first recall"` ✅ · `rg -n "opt-in|default off"` ✅ · `rg -n "fallback"` QUICKSTART ✅ · `pwsh scripts/validate-docs-coverage.ps1` exit 0 (0 gaps) ✅ · `npx markdownlint-cli2` 0 issues ✅ · lectura de `src/llm.rs:341-419` + test `f100_incompatible_dylib_never_panics` para citar comportamiento real (no inventado)
- **Checklist anti-hábitos tóxicos:** cumplido — lectura adversarial de §6 sin huecos (default OFF sin camino a on; deny list; sin IDs estables); chequeo de privacidad no degradado.
- **Cómo se probó (revisor):** `rg SLO/opt-in/fallback` (7/5/1 hits) ✅ · `validate-docs-coverage` exit 0 "0 gaps" ✅ · `markdownlint-cli2` 0 issues ✅ · lectura línea a línea de `SPEC.md:74/101`, `FASE-A.md:35/40/46/55`, `src/llm.rs:206/341-419/526-531/1087`, `console.rs:106-149`, `setup-embeddings.ps1:11-14/73-78` ✅
- **Fixes post-review (2026-09-27):** O1 (passing recall exige modelo real + scope note) · O2 (deny list + connection metadata; anonimato client-side) · O3 (semántica local/consent + pin de implementación en rule 6) · N1 (fila 1 definicional) · N2 (`fallback=true` en QUICKSTART). O2/O3 se arrastran también al spec del follow-up de telemetría (FIND-175).
- **Veredicto:** ✅ **APPROVE** — review fresco `vanta-review` (sesión `ses_f1b51d51cffekZkFitR0oKhxP6`, 2026-09-27); 0 Critical/Required; Optional/Nits aplicados. ACCEPT habilitado (payload review fresh para HARD-07).

## Notas
- Stop condition (master Task 15): si la instrumentación requiere código >1d → solo spec + FIND.
- Pre-mortem F1 del master: telemetría default-on → **PROHIBIDO** (opt-in únicamente).
- No modificar `src/llm.rs` (fallback ya implementado y testeado, FIND-100); el delta es visibilidad + doc.
- Doc técnico (DISTRIBUTION/QUICKSTART) en inglés; este task file en español.
- Commit `docs:` atómico; push owner-gated.
- **Docs-only complete (Step 4):** las 3 piezas quedaron como spec/docs — 0 código modificado (invariante: no tocar `src/llm.rs`). La instrumentación ejecutable requiere código → code-follow-up registrado abajo (LEAD decide worker/FIND).
- **Code-follow-up (spec exacta — slice ≤1d):**
  1. **Fallback visible (UX):** aviso visible al usuario cuando el provider cae a dummy, cubriendo AMBOS triggers: (a) ORT no usable — hoy solo `tracing::warn!` (mensajes en `src/llm.rs`:361/380/412, log-only); (b) modelo ausente — hoy **silencioso** (`from_llm_cfg` `src/llm.rs`:206 + `embed_as` `src/llm.rs`:526-531 — hallazgo del autor). Contrato del mensaje: DISTRIBUTION §7 (flag `fallback: true` + consecuencia + remediación). Superficie mínima: wizard live test + salida CLI/MCP donde se reporta estado de embeddings. Test: patrón `f100` (nunca pánico + aviso presente).
  2. **Telemetría local (opcional, sin red):** eventos `install_completed` / `first_recall` / `fallback_used` a log JSONL local — default OFF red, sin IDs estables, con la deny list de DISTRIBUTION §6. Si excede 1d o toca transmisión remota → FIND separado (la red queda spec-only).
  - Stop condition master Task 15 respetada: nada >1d; sin código a medias.
- **Nota de privacidad (para Review P2-01):** la spec de telemetría (§6) es opt-in, default OFF, sin identificadores estables y con deny list explícita; validarla ANTES de cualquier implementación (telemetría = dato de usuario).

## Context Save Point
- **Fecha:** 2026-09-27
- **Branch:** develop
- **CI pendiente:** no (docs; coverage + markdownlint locales ✅)
- **Decisiones:** SLO = TTFR ≤30 min + ≥4/5 fichas (ancla `SPEC.md`:101 + FASE-A); telemetría opt-in default OFF (pre-mortem F1); fallback spec cubre también el gap silencioso de modelo ausente (citado en §7 con líneas reales)
- **Problemas conocidos:** review P2-01 + commit pendientes (LEAD); implementación UX/telemetría = code-follow-up ≤1d
- **Próxima tarea:** WIRE-02 (F2)
