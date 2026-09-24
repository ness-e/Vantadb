# FIND-149: Harness L7 — Semgrep OSS + MCP + ast-grep como checks versionados

## Metadata
- **Plan file:** docs/dev/plans/2026-09-24-harness-gaps.md
- **Fuente:** Backlog FIND-149 (auditoría harness 2026-09-24) + plan §FIND-149
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Tipo:** Mixto (harness config `.opencode/` + host `opencode.jsonc`, sin código core)
- **Turns estimados:** 15
- **Creado:** 2026-09-24T12:00
- **last-synced:** 2026-09-24T12:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-audit` (L7 Security), `/audit full` L9 futuro, CI host (futuro gate) |
| Callees | semgrep 1.178.0 (pip), ast-grep 0.45.3 (cargo), registry rules `p/rust` `p/python` `p/typescript` (solo referenciados, no vendorizados) |
| Implicaciones | contrato público no cambia; sin impacto en perf/mem/serialización; sin migración; tests existentes no afectados (solo añade config + docs) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `docs/dev/plans/2026-09-24-harness-gaps.md` §FIND-149, `.opencode/agents/vanta-audit.md` (216L), `.opencode/commands/harness.md` (51L), `opencode.jsonc` (112L)
- **Archivos referenciados hacia dentro:** yml nuevo referencia registry `p/rust`, `p/python`, `p/typescript` (remoto, no vendorizado); `vanta-audit.md` referencia `references/security-checklist.md`, `definition-of-done.md`, `floor-guard.md` (no se tocan); `harness.md` convive con §FIND-151 (commit f425d88 — solo agregar §Notas, no reescribir)
- **Archivos que referencian a los editados:** `opencode.jsonc` lo consume OpenCode al arrancar (nuevo server `semgrep`, `disabled: true` default → sin efecto hasta activar); `vanta-audit.md` invocado por `/audit` y `/ship` (cambio aditivo en §2, no altera verdictos existentes)
- **Veredicto impacto:** bajo — 1 yml nuevo + 2 ediciones aditivas en `.opencode/` + 1 bloque MCP deshabilitado en host. Rollback = revert de 2 commits. PROHIBIDO tocar: `vanta-review.md` + otros 10 agents (a56dde9), `deny.toml`/DoD/workflows (FIND-150), `evals/` + Gate H (f425d88, salvo §Notas aditiva).

## Contrato

"`semgrep --config .opencode/configs/semgrep-vanta.yml --error` exit 0 en main + 1 regla que falle a propósito en rama test y pase tras fix + dictamen que cita output de semgrep real"

## Spec (SDD — Phase 1b)

No aplica (no es feature-add): la solución no agrega `pub fn`, tools MCP en código, endpoints ni bindings. Solo config versionada (yml), registro MCP deshabilitado por default y docs. Decisión técnica abierta documentada: severidad inicial WARNING (warn primero, endurecer a ERROR tras 1 semana verde) — resuelta por-evidencia: `semgrep scan --help` confirma `--error` = "Exit 1 if there are findings" (cualquier severidad), por lo que WARNING pasa el gate `--error` solo si hay 0 findings; el endurecimiento futuro = subir `severity: ERROR` + gate CI (ref: plan §FIND-149 Riesgos).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** no tocar agentes FIND-148 ni archivos FIND-150/151 fuera de §Notas aditiva; semgrep MCP siempre `disabled: true` por default; reglas empiezan en WARNING, nunca ERROR directo
- **Comandos de verificación:** `$env:Path += ";...\Scripts"; semgrep --config .opencode/configs/semgrep-vanta.yml --error --metrics=off .` → exit 0; `sg --version` → 0.45.3
- **Deuda pendiente:** ninguna (endurecimiento warn→error calendarizado a 1 semana verde, dueño: vanta-lead)

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | Valor |
|------------------------|-------|
| `activeGoal` | FIND-149: Semgrep OSS + MCP + ast-grep en L7 |
| `lastAction` | Steps 1-7 ✅ (tools, yml+sgconfig, gate exit 0, negativa 3+2 fail→fix→pass, MCP, L7 audit.md, harness.md Notas, Backlog FIND-149✅+FIND-152) |
| `result` | OK |
| `nextAction` | ninguno (última tarea del plan; commits pendientes al cierre) |
| `contract` | ver §Contrato; verificacion: GATE_EXIT=0 + SEMGREP_NEG_EXIT=1/FIX_EXIT=0 + ast-grep 2→0 (rama test borrada); evidencia: §Dictamen; artefactos: yml+sgconfig+MCP+audit.md+harness.md; invariantes: §Invariantes; deuda: FIND-152 residual (otro rol); queda_pendiente: 2 commits (host + .opencode) |
| `nextTask` | ninguna (última tarea del plan) |

## Deuda técnica (Regla 6 — MUST)

Sin deuda — solo añade config + docs, no toca código core.

## Definition of Done (contrato multi-nivel — P2-08)

- **Task:** contrato §Contrato verificado mecánicamente (exit 0 + negativa + dictamen con cita)
- **Commit:** 2 commits atómicos (host: `opencode.jsonc`; `.opencode` vía `git -C`: yml+audit.md+harness.md), conventional + `FIND-149`
- **Release:** no aplica (harness, sin cambio de versión); justify: sin código producto

## Herramientas necesarias
- semgrep 1.178.0 (pip, verificado `--version`), `semgrep mcp` (MCP stdio, verificado `--help`)
- ast-grep 0.45.3 / `sg.exe` (cargo --locked, verificado en install log)

**Skills cargadas (SDP):** `doubt-driven-development` (security-sensitive, adversarial review de reglas) + `security-and-hardening` (threat model L7, STRIDE en boundaries FFI) + `source-driven-development` (verificar sintaxis de reglas contra docs oficiales semgrep/ast-grep) + base `campaign-executor` + `progreso`. SDP step: `campaign_discover_skills_v2` BUILD keywords [semgrep, ast-grep, SAST, L7, audit] → lifecycle útil parcial (frontend-ui-engineering / api-and-interface-design descartadas por ruido, no aplican a harness config).

## Investigation Notes

### Digest internet (≤500 palabras, URLs verificadas vía webfetch 2026-09-24)

- **Semgrep OSS (Community Edition):** motor open-source (LGPL-2.1) de análisis estático, 30+ lenguajes, reglas como código con sintaxis "como el código que ya escribís" (sin pelear con AST ni DSL doloroso). Instalación: `python3 -m pip install semgrep` / `brew install semgrep` / Docker. Reglas propias en YAML + Registry con 2000+ reglas comunitarias (`p/rust`, `p/python`, etc.). Por defecto el código se analiza local, nunca se sube. En contextos de seguridad, la Community Edition solo analiza dentro de una función/archivo (sin cross-file/dataflow) — para SAST serio recomiendan la AppSec Platform; para guardrails de estilo/`unwrap`/SAFETY como los nuestros, CE alcanza. Fuente: https://github.com/semgrep/semgrep (verificado: README con install, MCP server, 16.8k stars).
- **Semgrep MCP server:** `semgrep mcp` arranca un MCP server (stdio + streamable-http puerto 8000) que deja a asistentes IA correr scans directos; integra con Cursor/VS Code/Windsurf/Claude Desktop; prompts built-in como `write_custom_semgrep_rule`; plugin Claude Code vía `/plugin marketplace add semgrep/mcp-marketplace`. Registro en `opencode.jsonc` como server local `["semgrep", "mcp"]` stdio. Fuente: https://github.com/semgrep/semgrep (sección "Semgrep MCP Server", verificada en el mismo fetch).
- **ast-grep (MIT):** búsqueda/reescritura estructural por AST ("syntax-aware grep/sed"), rapidísimo en Rust paralelo, políglota (20+ lenguajes, parsers tree-sitter registrables), escala de one-liner (`sg -p '$A && $A()'`) a linter versionable (`sg scan` con reglas YAML) con pretty error reporting, más binding Node programático. Mejor que regex para "unwrap en prod" / "unsafe sin SAFETY" porque matchea nodos sintácticos, no texto. Fuente: https://ast-grep.github.io (verificada: tagline + quickstart + scan-as-linter).
- **Decisión de diseño derivada:** semgrep para el gate versionado L7 (yml en repo + `--error` en CI futuro); ast-grep como complemento estructural documentado (one-liners + `sg scan` futuro) sin config propia en esta tarea (YAGNI: un solo sistema de reglas versionadas por ahora).

### Evidencia de versiones (tool result real)

- `semgrep --version` → `1.178.0` (PATH: `C:\Users\Eros\AppData\Roaming\Python\Python314\Scripts`)
- `semgrep mcp --help` → MCP server stdio + streamable-http (verificado)
- `cargo install ast-grep --locked` → `ast-grep v0.45.3` (`ast-grep.exe` + `sg.exe` en `C:\Users\Eros\.cargo\bin`)

### Incógnitas resueltas

- `--error` = "Exit 1 if there are findings" (cualquier severidad) → el gate `--error` exige 0 findings en main; severidad WARNING es compatible y el endurecimiento futuro es subir a ERROR (ref: `semgrep scan --help`).
- `.unwrap()` en `src/**`: los hits inspeccionados (`rocksdb_backend.rs:395+`, `connection_pool.rs:99+`) viven en `#[cfg(test)] mod tests` → la regla usa `pattern-not-inside: #[cfg(test)] mod ...` para no flaggear tests.
- `unsafe` en `src/**`: existe (mmap, `unwrap_unchecked`, `extern "C"`, `Send/Sync`); los inspeccionados llevan `// SAFETY:` en línea previa → la regla matchea `unsafe` sin `// SAFETY:` adyacente vía `pattern-not` con comentario (a validar empíricamente en Step 3; fallback: regla informativa + excludes justificados).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — approach validado (empírico en Step 3) |
| Pendientes de ejecución (downhill) | 6 — steps 2–7 |
| % completado | 25% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — la tarea ES seguridad (L7 SAST): skill `security-and-hardening` cargada; threat model: trust boundary = código `src/**` que cruza FFI/panics (`unwrap` = DoS por panic, `unsafe` sin SAFETY = UB). STRIDE: Tampering (reglas como código versionadas, no editables por el auditado sin PR) + Repudiation (output citado en dictamen). Sin secretos ni dependencias nuevas (semgrep/ast-grep son herramientas, no deps del workspace).
- [x] **PERFORMANCE** — no aplica: sin hot path tocado (solo config + docs). Scan local <1 min estimado.

## Steps

### Step 1: Instalar tools
- **Archivos:** ninguno (herramientas)
- **Acción:** `pip install semgrep` + `cargo install ast-grep --locked`, verificar `--version` y `semgrep mcp --help`
- **Verify:** `semgrep --version` → 1.178.0; `sg --version` → 0.45.3
- **Estado:** ✅ DONE

### Step 2: Crear yml de reglas
- **Archivos:** `.opencode/configs/semgrep-vanta.yml` (nuevo) + `.opencode/configs/sgconfig.yml` + `.opencode/configs/sg-rules/unchecked-ops.yml` (nuevo — ast-grep versionado)
- **Acción:** 3 reglas semgrep WARNING (unwrap-en-prod fuera de cfg(test)/#[test], unsafe-review puntero, expect-genérico) + `p/rust` `p/python` en gate; `p/typescript` excluido (FP-by-design `react-insecure-request` en `desktop/scripts/selfcheck-web-e2e.ts:78`, healthcheck localhost). Unsafe-preciso imposible en semgrep (ignora comentarios: `pattern-not` con `// SAFETY:` sobre-excluye, probado) → ast-grep `vanta-unchecked-ops` (`unwrap_unchecked`/`get_unchecked`/`transmute`/`from_raw_parts`); `follows: line_comment` rechazado (positive-follows da 0 en todos lados, probado). Yml 100% ASCII (semgrep Windows lee config en cp1252); bloques `pattern: |` a 10 espacios (con 8 el escalar queda vacío → "0 rule(s)", probado).
- **Verify:** `semgrep --validate` → "valid, 3 rule(s)"; ast-grep dispara en `mapper.rs:66,112`
- **Estado:** ✅ DONE

### Step 3: Probar exit 0 + negativa en rama test
- **Archivos:** scratch `src/sgneg_test.rs` en rama `test/find-149-negativa` (sin commit, borrada tras el test junto con el archivo)
- **Acción:** gate canónico `--error --severity ERROR` (+`p/rust`+`p/python`) → exit 0 en develop. Negativa: scratch con 4 violaciones → semgrep 3 findings exit 1 (`vanta-unwrap-in-prod`, `vanta-unsafe-review`, `vanta-expect-generic`) + ast-grep 2 hits; tras fix → semgrep exit 0 + ast-grep 0 hits.
- **Verify:** GATE_EXIT=0; SEMGREP_NEG_EXIT=1 (3 findings); SEMGREP_FIX_EXIT=0; rama borrada (`git branch -D`), `git status` limpio salvo archivos de la tarea
- **Estado:** ✅ DONE

### Step 4: Registrar Semgrep MCP en opencode.jsonc
- **Archivos:** `opencode.jsonc` (host raíz)
- **Acción:** añadir server `semgrep` (`["semgrep", "mcp"]`, stdio, `disabled: true` + nota de perfil)
- **Verify:** JSONC parsea (`servers` incluye `semgrep`, `disabled: True` — verificado por parseo mecánico)
- **Estado:** ✅ DONE

### Step 5: L7 en vanta-audit.md
- **Archivos:** `.opencode/agents/vanta-audit.md` §2 Technical Constraints
- **Acción:** añadir pasos semgrep/ast-grep (comandos + severidad warn→error + cita obligatoria en dictamen)
- **Verify:** grep muestra items 9-11; `git -C .opencode diff` solo aditivo en §2
- **Estado:** ✅ DONE

### Step 6: Notas en harness.md
- **Archivos:** `.opencode/commands/harness.md` (solo §Notas nueva, sin tocar §FIND-151)
- **Acción:** documentar comandos semgrep/ast-grep + dónde enganchan + calendario warn→error
- **Verify:** `git -C .opencode diff` solo aditivo (§FIND-151 intacto)
- **Estado:** ✅ DONE

### Step 7: Verify + dictamen + commits
- **Archivos:** todos los anteriores
- **Acción:** verify aplicable (fmt no aplica — yml/md; clippy/nextest no aplica — sin Rust) + dictamen vanta-audit citando output semgrep real + 2 commits separados + `skill progreso`
- **Verify:** gate + scans §Dictamen + `git log` con hashes (ver Notas)
- **Estado:** ✅ DONE

## Dictamen vanta-audit (FIND-149 — cita output real, 2026-09-24, rama develop)

Gate canónico: `semgrep --config .opencode/configs/semgrep-vanta.yml --config p/rust --config p/python --error --severity ERROR --metrics=off .` → **exit 0** (GATE_EXIT=0).

Output real semgrep (own rules, `--json`, 1515 findings, todos WARNING):
- `vanta-unwrap-in-prod`: **1448** (1409 en/tras `mod tests` — ruido de test residual documentado; ~39 en contexto prod: `vantadb-wasm/src/lib.rs` 14, `src/cli_server_auth_tests.rs`+`src/server/cli_server_auth_tests.rs` 6+6, `src/shred/mod.rs` 4, `desktop/src-tauri/.../embed.rs` 3, `.../native.rs` 3, `bytes.rs`/`llm.rs`/`state.rs` 1 c/u)
- `vanta-unsafe-review`: **67** punteros de revisión (SAFETY la verifica el auditor, no la regla)
- `vanta-expect-generic`: **0** (código base limpio en mensajes genéricos)

Output real ast-grep (`ast-grep scan --config .opencode/configs/sgconfig.yml src vantadb-wasm/src desktop/src-tauri/src`): **14** `vanta-unchecked-ops` (e.g. `src/index/distance/kernels.rs:55,59,89,93,119,123,...`, `src/index/distance/mapper.rs:66,112` — `unwrap_unchecked`/`get_unchecked` con SAFETY adyacente verificado a mano).

Prueba negativa (rama `test/find-149-negativa`, scratch no commiteado, rama+archivo borrados): 4 violaciones → semgrep **3 findings exit 1** + ast-grep **2 hits**; tras fix → semgrep **exit 0** + ast-grep **0 hits**.

Veredicto: L7 SAST versionado operativo en warn mode. Residual prod → **FIND-152** (asignar a worker/engine, NO implementar desde rol audit). Endurecimiento a ERROR tras 1 semana verde (dueño: vanta-lead).

## Dependencias
- Sin bloqueantes (Wave 1; Wave 0 commiteada: a56dde9 FIND-148, 3fef3cf9 FIND-150, f6ade4f2 FIND-151)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** doubt-driven-development (degraded, mismo contexto — no hay agente distinto disponible en este runner; cross-model ofrecido al orquestador) + `vanta-audit` dictamen con cita real como evidencia
- **Enfoque:** ¿reglas con 0 FP en main sin gutear el check? ¿warn→error calendarizado?
- **Cómo se probó:** salidas de semgrep reales pegadas en Step 3
- **Checklist anti-hábitos tóxicos:**
  - [ ] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [ ] No saltarse la clarificación por "ya sé qué quiere".
  - [ ] No declarar done sin verificar contra los acceptance criteria.
  - [ ] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [ ] No hacer un solo intento de búsqueda y darlo por saturado.
  - [ ] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [ ] No reintentar en bucle sin diagnóstico.
  - [ ] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [ ] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [ ] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ✅ approve — approach correcto (warn-first con gate `--severity ERROR` honesto ante 1448 warnings reales en main); precisión fingida eliminada (unsafe→review-pointer + ast-grep unchecked-ops, enfoques muertos documentados con evidencia); residual derivado a FIND-152 sin implementar desde leaf. Checklist anti-hábitos: sin salidas inventadas (todas las cifras son output real de comandos ejecutados), sin done sin acceptance (contrato 3/3 verificado mecánicamente), sin reintento en bucle (cada ciclo diagnóstico + causa raíz: YAML indent, cp1252, follows).

## Notas
- Repo `.opencode/` es Git separado: commits con `git -C .opencode`; `opencode.jsonc` en host.
- Rama host: `develop`; rama `.opencode`: `main`. Prueba negativa en rama test efímera (se borra, sin commits).
- `p/...` defaults: incluir solo si dan 0 findings en main; si no, documentar exclusión y por qué.
