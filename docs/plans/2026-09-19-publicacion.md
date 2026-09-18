# Plan de Ejecución: Publicación MVP (PyPI + parity + showcase + gate Fase A) — 2026-09-19

> **Campaign ID:** 2c1b931f-977c-4500-8eb3-62de7c934bd7
> **Inicio:** 2026-09-19
> **Estado:** ⬜ PENDIENTE (plan listo, sin iniciar)
> **Fuente:** `docs/Backlog.md` (PROV-12, FIND-98 re-DEFER, SHOW-02/03, TS-10, WSM-14, EXE-03) + smoke E2E 7 fases verdes (`docs/plans/2026-09-18-smoke-e2e.md`, GO sin bugs nuevos) + Gate P owner 2026-09-19
> **Autonomous:** false (PROV-12 exige secrets del owner; EXE-03 exige humanos)
> **FAIL_MODE:** `parallel` declarado (MAX 3) con **ejecución secuencial por rate-limit** (precedente 2 campañas: 0/12 y 9/9 en secuencial; retrospectiva cierre-mvp: secuencial-desde-inicio tras primer rate-limit)
> **SPEC:** `SPEC.md` raíz (§ Alcance cierre-mvp + Success Criteria: usuario nuevo 1 comando → `fallback:false` + recuerdo por sinónimo <30 min) — sin cambios en este plan (0 greenfield, todo distribución/showcase/release).
> **Gate P:** set de 6 aprobado por owner (propuesto en análisis integral + smoke GO, sin objeciones).
> **SDP (plan):** base fija abajo; cada sub-agente ejecuta `campaign_discover_skills_v2` (phase PLAN→BUILD) en pipeline-full Paso 0b y declara `SKILLS_CARGADAS:`.

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 6 (FIND-98-retry, SHOW-02, SHOW-03, DIST-10/14, PROV-12, EXE-03-prep) |
| 🟡 DEFER | resto del Backlog (P51 IMPL-MGR, UX-*, DESKTOP-43/44, FIND-118 futuro) |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 (PROV-12 y EXE-03 tienen Gate V/stop honesto, no bloqueo) |

Status: ⬆️ uphill = 1 (estrategia PyPI del owner) · ⬇️ downhill = 6 tasks con contrato definido
**Tras este plan el MVP es publicable:** wheels instalables + binario en parity + showcase que prueba valor + distribución planeada + kit Fase A listo para humanos.

## Gate P — alcance confirmado (2026-09-19)

Orden: FIND-98-retry + SHOW-02 → SHOW-03 + distribución → PROV-12 (con secrets) + EXE-03-prep (kit, la ejecución con humanos es owner-side). Sin features nuevas (FIND-120/121/122 y P51 esperan a después de publicar).

## Verificación real global (Paso 0, 2026-09-19 — smoke E2E + estado repo)

- Smoke 7 fases verde (`172f63c3`): MCP 79/87, loop memoria + `fallback:false`, sinónimos discriminativos, temporal 12/12, versions/supersede/delete/TTL, hooks 50/50, wizard/dry-runs/demo/roundtrip/salvage, superficie honesta, fmt/clippy + **full nextest 3331 passed**.
- Lock Windows liberado (Fase 0: cero PIDs) → retry FIND-98 desbloqueado.
- Alcance retry corregido por smoke: rebuild con `--features embed-local` + ORT 1.30 (`C:/Users/Eros/AppData/Local/Temp/opencode/ort130`), par `vanta-cli`+`vantadb-server` obligatorio (el server es wrapper), ORT ≥1.27 (1.26 paniquea en binario pre-FIND-100).
- `vantadb-ts/examples/` existe con 0 referencias → SHOW-05 ya lo referenció (README:65, QUICKSTART:53); SHOW-02 construye encima.
- PROV-12: `pyproject.toml` + maturin existen (a verificar en DISCOVERY); falta CI multiplataforma + secrets.

## Marco normativo — reglas, referencias, comandos, agentes, MCP (obligatorio en ejecución)

Todo sub-agente, antes de codificar, carga y cita en su task file:

- **Reglas** (`.opencode/rules/` — la de su área, lectura completa): `release-ci.md` (98-retry, PROV-12) · `python-bindings.md` (PROV-12, SHOW-03) · `server-mcp.md` (SHOW-02/03) · `README.md` del dir (formato).
- **Referencias** (`.opencode/references/`): `definition-of-done.md` (todas) · `task-system.md` (lead) · `skills-engineering.md` (SDP) · `clean-code-clean-architecture.md` Ap. V (tareas con código) · `ocr-review.md` (gates VERIFY) · `dev-tools.md` + `test-suite.md` (verify).
- **Comandos** (`.opencode/commands/`): `pipeline.md` (ejecución) · `audit.md` (verify L9/post-tarea) · `ship.md`/`rollback.md` (solo al cierre).
- **Agentes** (routing por `Ruta`): `vanta-worker` (98-retry, SHOW-02/03) · `vanta-lead` (DIST, PROV-12, EXE-03-prep) · `vanta-review` (P2-01 todas, batch al cierre).
- **MCP/tools:** `codegraph` (primero si hay código) · `campaign` (task-system del lead) · `ocr` CLI (`ocr-review.ps1` en CIERRE) · agent-search/webfetch SOLO si ambigüedad (URLs verificadas o marca NO VERIFICADA + deuda TSYS-13).

## Tasks

### Wave0 — parity + recetas (disjuntos: fuera-del-repo+target · web+examples)

**Task 1: FIND-98-retry — rebuild + reinstall parity 87 (alcance corregido por smoke)**
- **Appetite:** 1d · **Esfuerzo:** 🟢 · **Prioridad:** 🔴 Alta · **Archivos clave:** `target/debug/vanta-cli.exe` (rebuild), `C:/Users/Eros/.cargo/bin/vanta-cli.exe` + `vantadb-server.exe` (par obligatorio fuera del repo), ORT 1.30 store, `docs/tasks/FIND-98.md` (lectura: evidencia lock + hallazgo par)
- **Verificación real:** ✅ CÓDIGO-REAL — instalado 79 vs fuente 87 en vivo; lock libre desde Fase 0 smoke; debug actual sin `embed-local` (siempre `fallback:true`); instalado pre-FIND-100 paniquea con ORT 1.26.
- **Gate Justificación:** sin parity el usuario no ve 8 tools; el smoke probó que copiar no basta (hay que compilar la feature).
- **Contrato:** rebuild `cargo build --bin vanta-cli --bin vantadb-server --features embed-local -j 2` (verificar flags exactos contra EMB-10 en DISCOVERY) + reinstall del par + `tools/list` = **87** en instalado + smoke `initialize` OK + `embed_texts` `fallback:false` (ORT 1.30) + `docs/tasks/FIND-98.md` sync; si el lock reaparece → STOP sin forzar (re-DEFER sigue válido).
- **Pre-mortem:** (1) lock reaparece al copiar (MCP vivo de la sesión) → STOP, no matar; (2) build frío 10min+ → `-j 2`, no matar el build.
- **Stop:** lock persistente tras 2 intentos → re-DEFER con evidencia (no forzar, no matar procesos).
- **Risk Register:** 🟡×🟢 lock → STOP | 🟢×🟡 build frío → paciencia.
- **Cynefin:** 🟦 obvio. **Top 3:** lock / features exactas / smoke-87.
- ⬆️ 0 / ⬇️ 2 steps. **DoD:** contrato + task file sync + recitation; commit `fix:` (task file; binarios fuera del repo); sin deuda.
- **Skills (≤8):** systematic-debugging · incremental-implementation · shipping-and-launch (distribución) (+ base).
- **Herramientas+MCP:** MCP stdio smoke + `cargo build -j 2` + `campaign_verify_cmd` (bug exit -1 → bash).
- **Referencias:** rules `release-ci.md` · agents `vanta-worker` + `vanta-review` P2-01.
- **Investigación:** código (flags EMB-10, entry bins, lock por handle) en DISCOVERY; internet N/A.
- **Dependencias:** Wave0 (SHOW-02 en paralelo-secuencial, disjuntos). Sin bloqueantes. NextTask: SHOW-02 (orquestador).
- Task file `docs/tasks/FIND-98.md` (EXISTE — continuar, no re-crear) · ⬜ PENDING · Ruta vanta-worker. Branch develop. Commit `fix: FIND-98-retry — ...`.

**Task 2: SHOW-02 — recetas clicables del playground (5-6)**
- **Appetite:** 2d · **Esfuerzo:** 🟡 · **Prioridad:** 🟠 Media-Alta · **Archivos clave:** `web/src/app/playground/`, `web/src/components/vanta/code-playground*`, `vantadb-wasm` (playground real)
- **Verificación real:** 🟡 VERIFICAR en DISCOVERY — reutiliza `CodePlayground` + iframe WASM existente (verificar que existen y corren; si el playground está roto → reportar, no reconstruir).
- **Gate Justificación:** showcase que prueba valor público (RAG, híbrido, grafo, TTL, batch, persistencia); prerrequisito del anuncio.
- **Contrato:** 5-6 recetas clicables corriendo contra el playground (cada una: setup + run + resultado visible) + 1 línea `docs/api/MCP.md`: ops dependientes en invocaciones secuenciales (nota smoke Fase 2) + coverage 0 gaps si toca docs.
- **Pre-mortem:** (1) playground roto de base → diagnosticar y DEFER con motivo (no reconstruir la web); (2) WASM pesado en CI → recetas solo-local con nota.
- **Stop:** playground base no corre → DEFER con diagnóstico (no rediseñar web/).
- **Risk Register:** 🟡×🟡 playground roto → DEFER | 🟢×🟢 alcance → 6 recetas máximo.
- **Cynefin:** 🟨 complicado (hasta ver el playground). **Top 3:** playground-vivo / 6-recetas / nota-secuencial.
- ⬆️ 1 (estado playground) / ⬇️ 3 steps. **DoD:** contrato + task file + recitation; commit `feat:`; sin deuda.
- **Skills:** test-driven-development (receta = test e2e) · systematic-debugging · documentation-and-adrs (+ base).
- **Herramientas+MCP:** Playwright MCP si aplica + `campaign_verify_cmd`.
- **Referencias:** rules `frontend-web.md` si toca `web/` · agents `vanta-worker` + `vanta-review` P2-01.
- **Investigación:** código (playground existente) en DISCOVERY; internet N/A.
- **Dependencias:** Wave0 (FIND-98-retry en secuencia, disjuntos). Sin bloqueantes. NextTask: SHOW-03 (orquestador).
- Task file `docs/tasks/SHOW-02.md` · ⬜ PENDING · Ruta vanta-worker. Branch develop. Commit `feat: SHOW-02 — ...`.

### Wave1 — demo RAG + distribución (disjuntos: examples · docs/strategia)

**Task 3: SHOW-03 — prototipo estrella RAG-sobre-PDFs (100% local)**
- **Appetite:** 3d · **Esfuerzo:** 🟠 · **Prioridad:** 🔴 Alta · **Archivos clave:** `examples/` (nuevo `rag_pdf_chat/`), `vanta-memory` ingestores/`haystack_documentstore.py` como referencia (lectura)
- **Verificación real:** 🟡 VERIFICAR en DISCOVERY — spec de ingesta PDF en MGR-25 (leer alcance: ¿qué formato ya se ingiere?); `wiki_ingest` hoy solo `.md` (`src/wiki/sources.rs:82`) → el slice decide chunk-embed-chat con lo que haya, sin implementar ingestores nuevos (eso es MGR-25).
- **Gate Justificación:** demo que vende el caso RAG local con citas; sin ella el anuncio no tiene prueba pública.
- **Contrato:** 1 comando, sin credenciales: subir PDF → chunk → embed → chat con citas verificables (assert mecánico: la cita existe en el PDF) + README del ejemplo.
- **Pre-mortem:** (1) PDF sin extractor disponible → PDF de prueba simple o fixture con texto extraíble, no implementar parser; (2) credenciales/LLM externo → prohibido, 100% local.
- **Stop:** sin vía local en 3d → DEFER con diagnóstico (no meter dependencias nuevas).
- **Risk Register:** 🟡×🟡 extractor → fixture simple | 🔴×🟢 credenciales → prohibidas.
- **Cynefin:** 🟨 complicado. **Top 3:** extracción / local-only / cita-verificable.
- ⬆️ 1 (vía de ingesta PDF) / ⬇️ 3 steps. **DoD:** contrato + task file + recitation; commit `feat:`; sin deuda.
- **Skills:** test-driven-development (demo como e2e) · systematic-debugging · documentation-and-adrs (+ base).
- **Herramientas+MCP:** pytest/smoke + `campaign_verify_cmd`.
- **Referencias:** rules `server-mcp.md` (+`python-bindings.md` si es Python) · agents `vanta-worker` + `vanta-review` P2-01.
- **Investigación:** código (MGR-25 + wiki ingestores) en DISCOVERY; internet solo si formato incierto.
- **Dependencias:** Wave1 (DIST-10/14 en secuencia, disjuntos). Sin bloqueantes. NextTask: DIST-10/14 (orquestador).
- Task file `docs/tasks/SHOW-03.md` · ⬜ PENDING · Ruta vanta-worker. Branch develop. Commit `feat: SHOW-03 — ...`.

**Task 4: DIST-10/14 — plan distribución + adopción npm (estrategia escrita)**
- **Appetite:** 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🟠 Media-Alta · **Archivos clave:** `docs/` (estrategia), `vantadb-ts/README.md`, `web/` sección Why (lectura), `package.json` npm (lectura)
- **Verificación real:** 🟡 VERIFICAR en DISCOVERY — TS-10 (playground + docs-site + comparativa) y WSM-14 (README npm nicho "browser AI agent memory", estrategia H-21) son estrategia escrita, no código.
- **Gate Justificación:** sin plan de distribución no hay lanzamiento, solo código; es el prerrequisito documental del anuncio.
- **Contrato:** `docs/DISTRIBUTION.md` (o sección equivalente decidida en DISCOVERY) con canales (PyPI/npm/GH releases), comparativa honesta vs Orama (TS-13 contenido, con números verificados o sin números), posicionamiento nicho + checklist de anuncio; coverage 0 gaps.
- **Pre-mortem:** (1) claims sin fuente (Regla 11) → cada número con bench+comando o se quita; (2) duplicar docs existentes → referenciar, no copiar.
- **Stop:** N/A (docs; si falta un dato verificado → se marca TODO con owner, no se inventa).
- **Risk Register:** 🔴×🟢 claim sin fuente → prohibido (Regla 11) | 🟢×🟢 duplicación → referenciar.
- **Cynefin:** 🟦 obvio (con datos) / 🟨 (comparativa). **Top 3:** fuentes / no-duplicar / checklist.
- ⬆️ 0 / ⬇️ 2 steps. **DoD:** contrato + task file mínimo + recitation; commit `docs:`; sin deuda.
- **Skills:** documentation-and-adrs (+ base).
- **Herramientas+MCP:** grep referencias + coverage ps1 + `campaign_verify_cmd`.
- **Referencias:** agents `vanta-lead` (distribución) + `vanta-review` P2-01.
- **Investigación:** N/A código (inventario docs); internet solo para datos de comparativa con URLs verificadas.
- **Dependencias:** Wave1 (SHOW-03 en secuencia, disjuntos). Sin bloqueantes. NextTask: PROV-12 (orquestador).
- Task file `docs/tasks/DIST-10-14.md` · ⬜ PENDING · Ruta vanta-lead. Branch develop. Commit `docs: DIST-10/14 — ...`.

### Wave2 — release + humanos (disjuntos: packaging · kit-docs)

**Task 5: PROV-12 — publicar wheels PyPI (con Gate V del owner)**
- **Appetite:** 2d · **Esfuerzo:** 🟡 · **Prioridad:** 🔴 Alta · **Archivos clave:** `vantadb-python/pyproject.toml`, maturin, workflow release nuevo, `docs/operations/CI_POLICY.md` (lectura)
- **Verificación real:** 🟡 VERIFICAR en DISCOVERY — pyproject + maturin existen; falta CI multiplataforma + secrets; prereqs PROV-01/02/04 (verificar estado: si faltan → esta task se re-triagea, no se ejecuta a ciegas).
- **Gate Justificación:** sin `pip install` el claim de instalación es falso; es el desbloqueador técnico del anuncio.
- **Contrato:** `TestPyPI` primero (recomendado) → `pip install` en entorno limpio + smoke `Client` verde; luego PyPI con la misma evidencia; docs con versión instalada == código.
- **Pre-mortem:** (1) sin token no hay publish → Gate V ANTES de codificar (no después); (2) matriz multiplataforma rompe en un OS → ship con la matriz que pase + nota, no bloquear todo.
- **Stop:** Gate V sin respuesta del owner (token/estrategia/versión) → en espera, NO ejecuta; prereqs ausentes → re-triage.
- **Risk Register:** 🔴×🔴 secrets → Gate V previo, nunca a disco | 🟡×🟡 matriz → ship parcial.
- **Cynefin:** 🟨 complicado. **Top 3:** secrets / matriz / prereqs.
- ⬆️ 1 (estrategia + secrets del owner) / ⬇️ 3 steps. **DoD:** contrato + task file + recitation; commit `ci:`/`feat:`; sin deuda.
- **Skills:** ci-cd-and-automation · git-workflow-and-versioning · security-and-hardening (secrets) (+ base).
- **Herramientas+MCP:** maturin + TestPyPI + entorno limpio + `campaign_verify_cmd`.
- **Referencias:** rules `release-ci.md`, `python-bindings.md` · agents `vanta-lead` + `vanta-review` P2-01.
- **Investigación:** código (pyproject/maturin/CI) en DISCOVERY; internet solo si docs PyPI inciertas.
- **Dependencias:** Wave2 (EXE-03-prep en secuencia, disjuntos). Bloqueante honesto: Gate V del owner (token/estrategia/versión) ANTES de codificar. NextTask: EXE-03-prep (orquestador).
- Task file `docs/tasks/PROV-12.md` · ⬜ PENDING · Ruta vanta-lead. Branch develop. Commit `ci: PROV-12 — ...`.

**Task 6: EXE-03-prep — kit gate Fase A listo (la ejecución es owner-side)**
- **Appetite:** 1d · **Esfuerzo:** 🟢 · **Prioridad:** 🟠 Media-Alta · **Archivos clave:** `docs/` (checklist Fase A), plantilla usuario-01, README estado Early Access (verificar que existe y está honesto)
- **Verificación real:** ✅ CÓDIGO-REAL parcial — gate definido en Plan de acción (stranger-test + checklist todo-SI + 5 usuarios); EXE-03 pendiente porque requiere humanos.
- **Gate Justificación:** el gate Fase A es el desbloqueador del anuncio; esta task deja todo listo para que el owner solo tenga que conseguir las personas.
- **Contrato:** kit completo: checklist Fase A imprimible (producto/calidad/docs/comunidad/comunicación) + plantilla usuario-01 por persona + README verificado honesto (límites declarados) + doc `docs/FASE-A.md` (o equivalente) con instrucciones para los testers; coverage 0 gaps.
- **Pre-mortem:** (1) README deshonesto (features que no existen) → auditar contra `tools/list` real antes de entregar el kit; (2) duplicar plantillas → una sola fuente.
- **Stop:** N/A (kit docs; la ejecución con humanos queda fuera, en manos del owner).
- **Risk Register:** 🔴×🟢 README deshonesto → auditoría previa obligatoria | 🟢×🟢 alcance → kit, no ejecución.
- **Cynefin:** 🟦 obvio. **Top 3:** honestidad-README / kit-completo / handoff-owner.
- ⬆️ 0 / ⬇️ 2 steps. **DoD:** contrato + task file mínimo + recitation; commit `docs:`; sin deuda.
- **Skills:** documentation-and-adrs (+ base).
- **Herramientas+MCP:** grep/auditoría README-vs-código + coverage ps1 + `campaign_verify_cmd`.
- **Referencias:** agents `vanta-lead` + `vanta-review` P2-01.
- **Investigación:** N/A código (auditoría docs); internet N/A.
- **Dependencias:** Wave2 última en secuencia (PROV-12 en secuencia, disjuntos). Sin bloqueantes. NextTask: ninguna — última del plan (cierre del orquestador: P2-01 batch + progreso + archive).
- Task file `docs/tasks/EXE-03-prep.md` · ⬜ PENDING · Ruta vanta-lead. Branch develop. Commit `docs: EXE-03-prep — ...`.

## SKIP

Ninguno (todo lo triagiado con gap real va a DO).

## DEFER (fuera de publicación)

- P51 IMPL-MGR, FIND-120/121/122 (features post-publicación, explícitamente no ahora).
- FIND-118 (trigger 0.6.0), UX-*, DESKTOP-43/44, TS-11/13, WEB-09, SHOW resto: no bloquean anuncio.
- DESKTOP-41 (VM limpia): sigue DEFER (Gate P cierre-mvp vigente).

## BLOQUEADO

Nada (PROV-12 y EXE-03 tienen stops honestos, no bloqueos).

## Grafo de dependencias / Waves (FAIL_MODE=parallel declarado, ejecución secuencial por rate-limit)

```
Wave0: FIND-98-retry + SHOW-02   (bins/docs-task · web+examples — disjuntos)
Wave1: SHOW-03 + DIST-10/14      (examples · docs — disjuntos)
Wave2: PROV-12[Gate V owner] + EXE-03-prep   (packaging · kit — disjuntos)
```

Tras este plan: MVP publicable (instalable + showcase + distribución + kit Fase A). El anuncio espera a EXE-03 ejecutado (humanos, owner-side).

## Riesgos globales

| Riesgo | Respuesta |
|--------|-----------|
| Rate-limit proveedor | secuencial desde el inicio; backoff 2min + retry |
| `campaign_verify_cmd` bug exit -1 | bash directa + mención en RESULTADO |
| rustc crash paralelo / OOM | `-j 2` siempre en cargo |
| Lock binario Windows (98-retry) | STOP sin forzar, re-DEFER sigue válido |
| Secrets PyPI a disco (PROV-12) | Gate V previo; prohibido; review lo audita |
| Claims sin fuente (DIST) | Regla 11: número sin bench+comando se quita |
| Posts con features v1.0.0 | pausados hasta Fase A; DIST no los publica, solo los prepara |
| WIP ajeno (`completions/*`, `.opencode`, `reparacion.bat`, stash GOV-C4, `C:/Users/Eros/.vantadb*`) | staging selectivo, intocable |

## Notas

- SKILLS_CARGADAS base sesión (plan): campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, spec-driven-development, ponytail(full).
- MCP/tools por tarea en su ficha (codegraph primero si hay código; campaign vía lead).
- Routing: `vanta-worker` (98-retry, SHOW-02/03) · `vanta-lead` (DIST, PROV-12, EXE-03-prep) · `vanta-review` P2-01 todas (batch al cierre por área).
- Estructura tomada de los 5 planes previos: Resumen + Gate P + Verificación real + Marco normativo + Tasks por wave + SKIP/DEFER/BLOQUEADO + Grafo + Riesgos + Notas + Recitation.

=== RECITATION ===
Objetivo activo: PLAN publicación-mvp (FIND-98-retry/SHOW-02/SHOW-03/DIST/PROV-12/EXE-03-prep)
Estado: plan (6 DO, 1 uphill de estrategia, Wave0 lista)
Última acción: plan creado desde Backlog + smoke GO (FIND-98 alcance corregido rebuild-feature, SHOW-02/03 acotados, DIST combina TS-10+WSM-14, PROV-12 con Gate V, EXE-03 solo kit)
Resultado: ✅
Próxima acción: `/pipeline run docs/plans/2026-09-19-publicacion.md` (Wave0: FIND-98-retry + SHOW-02, secuencial) o `/pipeline task <ID>`
Contrato: plan file existe con 6 tasks DO + resumen + waves + gates; task files bajo demanda
Invariantes: no se tocó código (modo plan read-only); Backlog intacto (6 filas pendientes)
Comandos de verificación: existencia del plan file
Deuda: PROV-12 espera secrets owner + EXE-03 ejecución humana + P51/features post-pub
Próxima tarea si completa: FIND-98-retry (Wave0)
last-synced: 2026-09-19
=== END RECITATION ===
