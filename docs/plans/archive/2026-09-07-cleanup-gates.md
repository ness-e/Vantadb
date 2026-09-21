# Plan de Ejecución: Cleanup Gates + Higiene 2026-09-07

> **Campaign ID:** dc17423f-72c8-45d4-827e-167e46273d7c
> **Inicio:** 2026-09-07
> **Estado:** ✅ COMPLETADA 3/3 (GOV-TK2 + DOC-SYNC-01 vía sub-agentes; STABLE-05 cierre lead-inline)
> **Fuente:** docs/Backlog.md (post-campañas 8/8 + 4/4)
> **Autonomous:** false
> **SPEC:** no existe SPEC.md — DO set sin feature-add greenfield; no se genera SPEC.
> **SDP:** triage read-only; skills base: campaign-executor, brainstorming,
> writing-plans, planning-and-task-breakdown, progreso, ponytail (full),
> spec-driven-development.
> **Selección owner:** las 3 tareas elegidas por el usuario vía `question`
> (STABLE-05 + GOV-TK2 + DOC-SYNC-01).

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 3 (elegidas por el owner, 1 por 1) |
| 🟡 DEFER | resto (splits, PRX/FUT/INTG/STABLE-09, MEM, BLOG-CTA, DESKTOP, PROV, TS, WSM, WEB-09, FIND-20/21, CI-01) |
| ❌ SKIP | 4 (REVIEW-10, UX-10, UX-11 + MOD-05/FIND-47 ya archivados) |
| 🔴 BLOQUEADO | 7 (MKT-18f/i, AUD-042, DESKTOP-41/43/44, PRX-10, TS-12) |

Status: ⬆️ uphill = 1 (GOV-TK2 alcance real del gap) · ⬇️ downhill = 3

## Gate P — confirmación usuario (2026-09-07)

Triage + explicación simple + selección por tarea vía `question` (multiple) →
owner marcó las 3. Sin disputas.

## Verificación real global (Paso 0)

- `rustup target list --installed` incluye `wasm32-unknown-unknown` +
  `wasm-pack.exe` presente + `vantadb-wasm/pkg/` existe → STABLE-05 ✅ ejecutable.
- `skills/vantadb-mcp/SKILL.md:116` dice "Available MCP Tools (79)" vs fila
  GOV-TK2 (15 vs 33) → números del row stales; gap real por re-verificar →
  GOV-TK2 ✅ como tarea de verificación/re-scoping (doc-only si el gap murió).
- `STABLE-06` row dice "264 tests" pero la suite está en 280 (MOD-24 +2 y
  crecimiento); header dice "99 activas" pero se removieron ~24 filas en las
  2 campañas → DOC-SYNC-01 ✅ real (higiene).
- `src/cli_server.rs` = shim de 13 líneas ("REVIEW-10 split the 5327-line
  god-file"); `src/server/` tiene 10 módulos → REVIEW-10 ✅ hecho → SKIP.
- `DataExplorer.tsx:172` (w-20 + max-w) y `:62,829-831` (empty state con salida)
  con comentarios UX-10/UX-11 → ambos ✅ implementados → SKIP.

## Tasks

### Task 1: STABLE-05 — Validar vantadb-wasm (gates 1-8)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb-wasm/Cargo.toml`, `vantadb-wasm/src/`, `vantadb-wasm/pkg/`
- **Verificación real:** ✅ CÓDIGO-REAL — toolchain presente (wasm-pack + target).
- **Gate Justificación:** último gate de paquete sin validar (node ✅, ts ✅);
  cierra la fila P47 y desbloquea STABLE-09.
- **Gate Result:** ✅ DO (elegida por el owner)
- **Contrato:** `rustup target add wasm32-unknown-unknown` ok +
  `cargo check -p vantadb-wasm` 0 warnings + `wasm-pack build --target bundler`
  exit 0 + `cargo fmt --check` 0 + docs `WASM_PERSISTENCE.md` al día +
  `cargo deny check` 0
- **Pre-mortem:**
  - **Fallo probable 1:** `wasm-pack test --node` requiere Chrome/headless →
    si bloquea, gates 1-8 sin ese sub-paso + nota (no inventar test).
  - **Fallo probable 2:** `pkg/` commiteado diverge del build → no commitear
    artefactos regenerados salvo que el repo los versione a propósito.
  - **Fallo probable 3:** toolchain Windows + wasm-pack lento → timeout generoso,
    1 sola corrida + nota de tiempo.
- **Stop conditions:** toolchain roto sin fix en 1h → BLOQUEADO con evidencia.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟢 | test --node sin browser | scoping + nota | primer intento |
  | 🟢×🟡 | pkg/ divergente | no commitear build | diff |
  | 🟢×🟢 | build lento | timeout largo | 1 corrida |
- **Cynefin:** 🟦 obvio (checklist de gates).
- **Top 3 riesgos:** browser; pkg/; tiempo.
- **Uphill/Downhill:** ⬇️ downhill.
- **DoD task:** contrato ✅ · sync · recitation.
- **Shape Up:** sí (último gate) / sí (≤1d) / sí (toolchain listo).
- **Task file:** `docs/tasks/STABLE-05.md`
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Commit:** 9d338a18 (cierre lead-inline: G1-G8 del ejecutor + re-verify + review)

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

### Task 2: GOV-TK2 — Re-verificar gap tools MCP (verificación, 0 código si murió)

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 2-4h
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-mcp/src/` (handlers/tools), `skills/vantadb-mcp/SKILL.md:116`
- **Verificación real:** ✅ CÓDIGO-REAL — skill declara 79 tools; fila dice
  15-vs-33 (stale). Gap real desconocido hasta contar el binario.
- **Gate Justificación:** la fila no es ejecutable como está (números falsos);
  primero medir la realidad, después decidir si hay trabajo.
- **Gate Result:** ✅ DO (elegida por el owner)
- **Contrato:** reporte con conteo real de tools expuestas por el binario
  (`vanta-cli server --mcp` vs handlers) + veredicto: gap real (re-scoping con
  lista exacta) o gap muerto (fila a historial con evidencia)
- **Pre-mortem:**
  - **Fallo probable 1:** contar tools requiere compilar el binario (lento) →
    contar handlers en código como proxy + 1 corrida si hace falta.
  - **Fallo probable 2:** "tools" mezcla skill_* + code_* + wiki_* + base →
    contar por prefijo, no total ciego.
  - **Fallo probable 3:** tentación de implementar el gap en el mismo task →
    PROHIBIDO: esta tarea solo verifica; el fix es tarea nueva si aplica.
- **Stop conditions:** binario no compila en 30min → veredicto con conteo
  estático + nota (no bloquear).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟢 | build lento | conteo estático primero | 30min |
  | 🟢×🟡 | scope creep a fix | prohibido por contrato | review |
  | 🟢×🟢 | conteo por prefijo | tabla por prefijo | reporte |
- **Cynefin:** 🟦 obvio (contar y comparar).
- **Top 3 riesgos:** build; creep; conteo.
- **Uphill/Downhill:** ⬆️ uphill: 1 (alcance real) · resto ⬇️.
- **DoD task:** contrato ✅ · sync · recitation.
- **Shape Up:** sí (fila no-ejecutable) / sí (≤1d) / sí (bloquea decisión).
- **Task file:** `docs/tasks/GOV-TK2.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

### Task 3: DOC-SYNC-01 — Sincronizar descripciones stales del Backlog

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Archivos clave:** `docs/Backlog.md` (header + fila STABLE-06)
- **Verificación real:** ✅ CÓDIGO-REAL — suite ts en 280 (no 264); ~24 filas
  removidas en 2 campañas y el header sigue diciendo 99.
- **Gate Justificación:** los agentes planean sobre estos números; backlog que
  miente genera triages falsos. Higiene de 10 minutos.
- **Gate Result:** ✅ DO (elegida por el owner)
- **Contrato:** header con conteo real de filas activas (contado por script, no
  a ojo) + STABLE-06 dice 280 tests + `git diff --stat` muestra solo Backlog.md
- **Pre-mortem:**
  - **Fallo probable 1:** contar "activas" a ojo → contar con comando
    (filas `|` con estado pendiente) y pegar el número + comando usado.
  - **Fallo probable 2:** tentación de reescribir secciones enteras →
    solo los 2 textos, nada más.
  - **Fallo probable 3:** 280 puede moverse si otro task añade tests →
    fechar la medición en la fila.
- **Stop conditions:** ninguna (XS); si aparece drift mayor, nota aparte sin scope creep.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟢×🟢 | conteo a ojo | comando + fecha | review |
  | 🟢×🟢 | creep | 2 textos | diff |
- **Cynefin:** 🟦 obvio.
- **Top 3 riesgos:** conteo; creep.
- **Uphill/Downhill:** ⬇️ downhill.
- **DoD task:** contrato ✅ · sync · recitation.
- **Shape Up:** sí/sí/sí.
- **Task file:** `docs/tasks/DOC-SYNC-01.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** corre última (los conteos dependen del estado final) o primera
  como baseline — a criterio del run (documentar hora de medición).

## SKIP con evidencia (no re-proponer sin evidencia nueva)

| ID | Evidencia de SKIP |
|----|-------------------|
| REVIEW-10 | `src/cli_server.rs` = shim 13 líneas; split en `src/server/` (10 módulos) |
| UX-10 | `DataExplorer.tsx:172,279-282` (w-20 + max-w responsivo) |
| UX-11 | `DataExplorer.tsx:62,829-831` (empty state con salida) |

## DEFER (resumen — detalle en Backlog.md)

- Splits: FIND-48/49/50 · Proxy PRX-02..13 · Futuro FUT-02..14, OLD-01
- Estrategia: BND-08, TS-10/11/12/13, WSM-14, INTG-01/02 (mini-spec), STABLE-06/09
  (solo hygiene de su texto en DOC-SYNC-01), GOV-TK5/8, BLOG-CTA,
- Desktop: DESKTOP-40/42/43/45, FIND-20/21, resto UX (06/08/09/12/15)
- Web: WEB-09 · CI-01 · CI/CD varios

## BLOQUEADO (con causa)

| ID | Causa |
|----|-------|
| MKT-18f | acción humana PyPI |
| MKT-18i | upstream AnythingLLM |
| AUD-042 | upstream tantivy ≥0.27 |
| DESKTOP-41/44 | sesión humana / VM |
| DESKTOP-43 | firma (wontfix DEVOPS-10) |
| PRX-10 | requiere PRX-03 |
| TS-12 | requiere BND-08 |

## Grafo de dependencias (orden)

```
STABLE-05 (independiente) ┐
GOV-TK2 (independiente) ┤→ Wave0 ×3 (DOC-SYNC-01 documenta hora de medición)
DOC-SYNC-01 (independiente) ┘
```

Waves (`FAIL_MODE=parallel`, MAX 3): Wave0 {STABLE-05, GOV-TK2, DOC-SYNC-01}.

## Riesgos globales

| Riesgo | Respuesta |
|--------|-----------|
| wasm-pack sin browser para test --node | scoping + nota, no inventar |
| GOV-TK2 tienta a implementar el gap | prohibido: solo verifica |
| Conteos del backlog a ojo | comando + fecha de medición |

=== RECITATION GOV-TK2 ===
Campaign ID: dc17423f-72c8-45d4-827e-167e46273d7c
Objetivo activo: GOV-TK2: re-verificar gap tools MCP (solo verificacion)
Estado: completed
Última acción: Reporte escrito en docs/tasks/GOV-TK2.md con tabla por prefijo + veredicto gap-muerto; lesson registrada
Resultado: OK
Próxima acción: ninguno — orquestador archiva fila con evidencia
Contrato: verificacion: rg conteo estatico + merge tools.rs:1003-1011 + perfiles 1028-1134 (sin compilacion, permitido por stop condition) | evidencia: 49 base unicos + 30 extendidas (6+8+6+1+3+6) = 79 == skill 79; fila 15-vs-33 stale | artefactos: docs/tasks/GOV-TK2.md (unico archivo tocado) | invariantes: 0 cambios fuente; plan file intacto; archivos STABLE-05/DOC-SYNC-01 no tocados | deuda: ninguna | queda_pendiente: orquestador mueve fila GOV-TK a historial; no abrir fix
Próxima tarea si completa: ninguno
=== END RECITATION ===

=== RECITATION 1 ===
Campaign ID: dc17423f-72c8-45d4-827e-167e46273d7c
Objetivo activo: STABLE-05 — Validar vantadb-wasm (gates 1-8)
Estado: in-progress
Última acción: DISCOVERY completo (codegraph, coverage, reglas js-ecosystem, Backlog P47, toolchain) + task file docs/tasks/STABLE-05.md creado con 8 steps
Resultado: PARTIAL
Próxima acción: Step G1: rustup target add wasm32-unknown-unknown
Contrato: verificacion: DISCOVERY completo, gates pendientes de ejecucion | evidencia: toolchain presente (wasm-pack 0.15.0, target instalado, deny 0.19.9) | artefactos: docs/tasks/STABLE-05.md | invariantes: pkg/ gitignored nunca commiteado; no tocar plan file ni archivos GOV-TK2/DOC-SYNC-01 | deuda: ninguna | queda_pendiente: ejecutar gates 1-8
Próxima tarea si completa: ninguna (Wave0, STABLE-09 fuera del plan)
=== END RECITATION ===

## Retrospectiva de cierre (Start / Stop / Continue + 1 acción medible)

**Start:**
- Wave0 ×3 sin colisión; Paso 0 detectó 3 SKIP (REVIEW-10 split hecho, UX-10/11
  implementados) + selección de tareas por el owner 1-por-1 vía `question`.
- Staging quirúrgico con patch filtrado cuando el worktree trae trabajo ajeno
  (ORG-01..11 de otra sesión quedaron sin staged, intactos).

**Stop:**
- Confiar en que el sub-agente "no tocó nada más": GOV-TK2/DOC-SYNC-01 sí
  escribieron plan recitations + Backlog (declarado en sus reportes, verificado
  en diff — esta vez sin daño, pero el plan file concurrente es frágil).
- `campaign_verify_cmd` sigue con bug (`autoTransition is not defined`) → bash.

**Continue:**
- Contratos mecánicos + re-verify lead con cwd correcto + sync por wave.

**Acción medible:** first-try COMPLETO 2/3 + 1 cierre lead-inline sobre WIP 90%
(3/3 efectivas, North Star >90% ✅). Aborts infra: 3 en esta campaña (1 Wave0 +
2 STABLE-05) — si se repiten, escalar a investigación del runner (no es
varianza de tarea).

**Commits campaña (develop):** f604cf8b GOV-TK2+DOC-SYNC-01 · 9d338a18 STABLE-05 ·
+ sync final (este commit).
Deuda: ninguna. STABLE-09 desbloqueado (P47 00..07 verdes salvo 3-corridas).
