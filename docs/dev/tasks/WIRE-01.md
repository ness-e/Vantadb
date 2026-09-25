# WIRE-01: Loop de memoria del proxy completo + cost real + presupuesto de inyección

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-24-post-investigacion-integral.md` (W3 primero; M3; serializar con W2 si colisiona `vanta-proxy` el mismo día)
- **Fuente:** Backlog `P56:955` + plan `:100,133,157`
- **Esfuerzo:** 🟡 1-2d · **Prioridad:** 🔴 · **Tipo:** Rust (bug `fix:` + wiring)
- **Turns estimados:** 15-20
- **Creado:** 2026-09-25 · **last-synced:** 2026-09-25
- **Estado:** ✅ COMPLETED (commit 55395600)
- **Incógnitas (uphill):** 1 (decisión scheduler-tool → default NO) · **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `build_memory_block` ← `vanta-proxy/src/server.rs:403`; `PersonaRecord` 9 callers (incl. `desktop/src-tauri/.../memory.rs`); `SceneBlock` 16 callers |
| Callees | `inject.rs:92-93` → `get_persona` + `current_scene`/`list_scenes`; capture → `proxy-turns` (`capture.rs:15`); search → `l1/{session}` (`l1_reader.rs:22`); cost → `record_response_usage` (`cost.rs:201`, hoy solo tests `:400`) |
| Implicaciones | Falla de diseño (no regresión): namespaces disjuntos capture vs search/injection; cost con output siempre 0; presupuesto = tope nuevo en bloque `<vanta-memory>`; shape persona/escenas intacto (desktop lo lee) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `vanta-proxy/src/capture.rs`, `vanta-proxy/src/inject.rs`, `vanta-proxy/src/cost.rs:180-220,395-415`, `vanta-proxy/src/server.rs:236-260,395-410`, `vanta-memory/src/core/record/l1_reader.rs:1-40`
- **Referencias hacia dentro:** `TURNS_NAMESPACE`, `l1_namespace()`, `build_memory_block`, MEM-50/D47 (capture L0 existe)
- **Referencias entrantes:** handlers openai/anthropic/responses → server → inject/capture
- **Veredicto impacto:** medio — proxy-only; core intacto; desktop intacto si no cambia shape persona/escenas

## Contrato
"E2E sesión-1-captura → sesión-2-recupera con hits; `output_tokens ≠ 0`; presupuesto del bloque respetado; `cargo test -p vanta-proxy` verde."

## Spec (SDD — Phase 1b)
No es feature-add en scope recomendado (wiring + budget por config, cero símbolos públicos). Decisión abierta: exponer `scheduler_run_once` como tool MCP **sí sería feature-add** (requiere Spec + Gate P/D) → default **NO** (scheduler host = MEM-55, fuera de scope).

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** passthrough byte-identical sin session/tools (gate permanente proxy); capture best-effort nunca bloquea request; shape `PersonaRecord`/`SceneBlock` (desktop).
- **Comandos de verificación:** `cargo test -p vanta-proxy loop_e2e injection_budget` + suite proxy completa + clippy `-D warnings`
- **Deuda pendiente:** ninguna al abrir

## Recitation
```
=== RECITATION ===
Objetivo activo: WIRE-01 — loop proxy + cost + presupuesto
Estado: implementado (Steps 1-3 ✅ + verify); Step 4 cierre en curso
Última acción: pivot Opción B (dual-write L1) + cost + budget, suite proxy verde
Resultado: ⬜→✅ parcial
Próxima acción: Step 4 — verify_changed + OCR + commit fix: + progreso
Contrato: ver ## Contrato
Invariantes: passthrough intacto; capture non-blocking; shapes desktop intactos
Deuda: ninguna
Próxima tarea si completa: WIRE-02 (perfil MCP)
PIVOT (Opción A→B): inyección de turnos mataba PRX-09 (claves nunca estables,
3 fallos deterministas) + churn de prefijo PRX-04 cada turno. Opción B:
turn_job dual-write (proxy-turns + l1/Episodic session-only); bloque inyectado
intacto; search existente recupera. Evidencia: stash sin cambios → prx09 6/6.
last-synced: 2026-09-25
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)
**Saldo neto:** Sin deuda (cablea piezas existentes, elimina promesa rota).

## Definition of Done (3 niveles)
- **Task:** contrato verde + clippy + fmt
- **Commit:** atómico, `fix:` + WIRE-01, verify mecánico antes
- **Release:** N/A (sin cambio de API pública)

## Herramientas necesarias
- `cargo check/test -p vanta-proxy`, codegraph_explore, `campaign_verify_cmd`
- **Skills cargadas (SDP):** source-driven-development (base) + systematic-debugging (loop inerte = síntoma; causa = namespaces) + test-driven-development (RED e2e primero) + incremental-implementation (1 variable: loop → cost → budget) + api-and-interface-design (observable del bloque inyectado cambia)

## Investigation Notes
- Capture escribe `proxy-turns`; injection lee persona/escenas que el proxy nunca escribe; search lee `l1/{session}` → memoria decorativa en deployment proxy-only.
- `record_response_usage` solo invocada en tests → output_tokens siempre 0.
- Tests existentes a extender: `pipeline.rs`, `prx09_cache.rs`, `recall.rs`, `persona.rs`.
- Destraba: VER-03/04 + métrica North Star (put+search 7d).

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 1 — scheduler como tool MCP (default NO) |
| Pendientes | 4 steps |
| % completado | 0% |

## Fase 1 — Evidencia de Debugging (GATE Bug)
- **Repro:** sesión 1 `vanta_memory_capture` → sesión 2 `vanta_memory_search` mismo session_key → 0 hits.
- **Hipótesis:** capture → `proxy-turns`, search/injection → `l1/*` + persona/escenas no escritas por el proxy.
- **1 variable:** cerrar el loop primero (leer también `proxy-turns` o escribir `l1/{session}`); cost y budget después.
- **Test RED:** e2e captura→recupera que hoy da 0 hits.

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — inyección en prompts = trust boundary (skill LLM08: retrieval como boundary; partición por sesión/namespace — no mezclar sesiones). Sin PII nueva.
- [x] **PERFORMANCE** — presupuesto acota tokens inyectados (mejora prompt-cache); medir tokens antes/después. Fuera de hot path search.

## Steps
### Step 1: Cerrar el loop — OPCIÓN B (pivot documentado abajo)
- **Archivos:** `vanta-proxy/src/capture.rs` (`turn_job` dual-write), `vanta-proxy/tests/tool_loop.rs` (e2e)
- **Acción:** capture escribe `l1/{session}` (registro `Episodic` canónico) además de `proxy-turns`; el search existente (`perform_auto_recall` + tool `vanta_memory_search`) recupera con hits. Test RED→GREEN e2e + unit
- **Verify:** `cargo test -p vanta-proxy -- loop_e2e` (2 passed) + `--lib turn_job_dual` ✅
- **Estado:** ✅ DONE

### Step 2: Cost real
- **Archivos:** `vanta-proxy/src/cost.rs` (getter `output_tokens_by_session`), `vanta-proxy/src/server.rs` (`maybe_store` observa `usage` en bodies con buffer), `vanta-proxy/tests/prx03_cost.rs`
- **Acción:** cablear `record_response_usage` en path productivo (buffered) → `output_tokens ≠ 0`. Prove-It: sin wiring el test FAIL.
- **Verify:** `cargo test -p vanta-proxy --test prx03_cost` (4 passed) ✅
- **Estado:** ✅ DONE

### Step 3: Presupuesto de inyección
- **Archivos:** `vanta-proxy/src/config.rs` (`InjectionConfig.max_tokens`, default 2000), `vanta-proxy/src/inject.rs` (`build_memory_block` con budget + `fit_sections` por prioridad persona > escena > índice), `vanta-proxy/src/server.rs:413`, 16 literales `ProxyConfig` en tests (+1 línea c/u)
- **Acción:** tope de tokens configurable + test de respeto
- **Verify:** `cargo test -p vanta-proxy --lib injection_budget` ✅
- **Estado:** ✅ DONE

### Step 4: Cierre
- **Archivos:** —
- **Acción:** fmt+clippy+nextest proxy, `verify_changed.ps1`, commit `fix:`, progreso
- **Verify:** suite `cargo test -p vanta-proxy` verde + fmt + clippy proxy + OCR delegation (Rule Group 1, sin Critical/High) + P2-01 APPROVE + commit 55395600 (atómico, sin push — pushea vanta-lead)
- **Estado:** ✅ DONE

## Dependencias
- Ninguna (serializar con WIRE-09 si colisiona `vanta-proxy` el mismo día — plan W2/W3). Destraba VER-03/04.

## Review (GATE P2-01)
- **Revisor:** vanta-review (sesión fresca `ses_f2609180fffeiavJVCB6DtehJN`, distinto del implementador)
- **Enfoque:** ¿loop cerrado sin romper passthrough? ¿budget efectivo?
- **Cómo se probó:** e2e real sesión 1→2, no auto-reporte
- **Checklist anti-hábitos:** según plantilla
- **Veredicto:** ✅ APPROVE (condicionado: commit atómico solo `vanta-proxy/` + 2 fixes doc aplicadas en el mismo commit; Medium serde-footgun → follow-up, no bloquea)

## Notas
- WIP ajeno PROHIBIDO: archivos de API-01 (`src/sdk/types/`, `QueryResult`) — no tocar.
- Fuera de alcance: rediseño L0→L3 (MGR-08/VER-07), scheduler host (MEM-55).
- Follow-ups del review P2-01 (no bloquean, fuera de este commit): (1) serde footgun — `[injection]` vacía → `max_tokens=0` (container `#[serde(default)]` ignora el default 2000); fix: `#[serde(default="...")]` en campo + test TOML; (2) `MARKER.len()` bytes vs chars en `fit_sections` (edge multibyte); (3) metadata `episodic`/50 derivada del record en vez de literales; (4) output-side para SSE (drain tool-loop) si la North Star lo exige.
