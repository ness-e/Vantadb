# TIR-03 — Decisión: ¿fase de contención en el pipeline de bugs?

> Investigación/decisión. Origen: gap-01-agents.md FALTA#15, REPORTE-FINAL.md §3.3-15,
> docs/Backlog.md P18 TIR-03 (🔴 prioridad). Estado: RESUELTO 2026-08-12.

## 1. Fuentes analizadas

| Fuente | Qué dice |
|--------|----------|
| `eng-02-systems.md:209-214` (§4.2 Incident response SRE) | "Mitigation — restaurar servicio PRIMERO (rollback, feature-flag off, scale-out). El debugging en caliente daña la experiencia: debuggear *después* de mitigar." Principio: **mitigar primero, root-cause después**. |
| `eng-02-systems.md:397-400` (§10 Fase 0 — Contención) | "Solo si hay impacto en producción: mitigar primero (rollback, feature flag off, restart, scale-out). No debuggear en caliente. Registrar el incidente. Entrar al protocolo solo cuando el sistema esté estable." |
| `docs/references/bug-workflow.md` (76L, completo) | **NO tiene fase de contención.** Fase 0 = Diagnosticar; Fase 1 = Aislar Causa Raíz; Fase 2 = Fix; Fase 3 = Commit. Un agente ante un build roto empieza debuggeando. |
| `RULES.md:204-219` (§10b Iron Law) | No fixes sin investigación de causa raíz primero. Exigible, pero **no prescribe estabilizar antes**. |
| `plan.md` (triage + risk register) | Ya tiene 🔴 BLOQUEADO y stop-conditions/triggers por riesgo — cubre "cuándo parar el plan". |
| `subagent-recovery.md` (SARL) | Escalera RESUME→RETRY→STRATEGY→ESCALATE para sub-agentes fallidos — cubre recuperación de ejecución. |

## 2. Diagnóstico del gap

- El protocolo SRE dice "mitigar primero" pero el `bug-workflow.md` del pipeline no lo refleja: un agente con un build roto entra directo a "Diagnosticar el Error".
- El caso de uso real de VantaDB **no es** un servicio con usuarios en producción (el §10:397 "solo si hay impacto en producción" no aplica literalmente). El caso real es **el propio pipeline de tareas**: build roto en `develop`, test suite fallando en masa, backoff del server de agentes.
- Lo que ya cubre el pipeline: `plan.md` (🔴 BLOQUEADO, stop-conditions), SARL (recuperación de sub-agentes). Lo que falta: **un paso explícito de estabilización antes de debuggear** para bugs que rompen el build/CI.

## 3. Opciones evaluadas

| Opción | Costo | Riesgo | Veredicto parcial |
|--------|-------|--------|-------------------|
| **A. Implementar** — añadir "Fase 0.5 Contención" a `docs/references/bug-workflow.md` (docs, ~10 líneas) | 🟢 mínimo | Bajo: no toca tooling ni state machine; no contradice Iron Law (es un paso ANTES, no lo reemplaza) | ✅ elegida |
| **B. WONTFIX (YAGNI)** — "no hay producción, el Iron Law + stop-conditions bastan" | 0 | Medio: un agente con build roto sigue debuggeando en caliente; el reporte ya lo marcó como gap; la fricción para el humano es real | descartada: el caso de uso (build roto en CI/develop) es frecuente y el fix es trivial |
| **C. Deferir** — esperar a que un incidente real lo demuestre | 0 | Medio: gap conocido no cerrado; el costo de implementar es menor que el de esperar | descartada por mismo motivo que B |

## 4. Veredicto: **IMPLEMENTAR (docs mínimos)**

Añadir a `docs/references/bug-workflow.md` una **Fase 0.5 — Contención/Estabilización** entre la Fase 0 (Diagnosticar) y la Fase 1 (Aislar Causa Raíz):

- **Disparador:** el bug rompe el build (`cargo check`/`clippy`/`nextest` falla en masa), rompe CI, o afecta un flujo activo del pipeline (backoff del server, test suite en rojo).
- **Acción:** estabilizar ANTES de debuggear — revert del último commit sospechoso (`git revert`) o pausar el plan actual y aislar el cambio. Registrar el incidente (qué se rompió, timeline) aunque no haya causa.
- **Después:** entrar a Fase 1 (Iron Law) solo con el sistema estable.
- **No reemplaza** el Iron Law: la contención es estabilizar el entorno; el RCA y el fix siguen siendo obligatorios.

**Justificación ponytail:** es la rung correcta de la escalera — la solución existe parcialmente (stop-conditions, SARL) y el cambio mínimo que cierra el gap es una sección de docs que *ordena* la secuencia existente. No se crea tooling, ni estados C0, ni scripts nuevos. Si en el futuro el gap persiste pese a la doctrina, el siguiente escalón sería un gate mecánico (script que detecte build roto y exija revert antes del fix) — hoy no hace falta.

## 5. Cambio aplicado

- `docs/references/bug-workflow.md`: nueva **Fase 0.5 — Contención/Estabilización** (sección corta antes de Fase 1), documentada con disparador, acción y relación con el Iron Law.

## 6. Revisión P2-01

- **Revisor:** vanta-review (sesión `ses_0070d5217ffe8TO33SJKYgLzKz`)
- **Enfoque:** veredicto IMPLEMENTAR correcto — gap real confirmado, alternativas evaluadas de verdad (no strawman), Fase 0.5 no contradice el Iron Law (es paso ANTES, Fase 1 sigue obligatoria), no toca state machine C0, proporcionado (ponytail).
- **Cómo se probó:** verificación por comando/lectura directa — `bug-workflow.md:18` (Fase 0.5 existe), `rg "Contención"` (líneas 18 y 31), citas de eng-02/RULES/plan/subagent-recovery verificadas verbatim.
- **Hallazgos:** ninguno bloqueante. 🟡 opcionales aplicados: (1) cross-reference Fase 0.5 en header de bug-workflow.md; (2) ruta explícita de eng-02-systems.md.
- **Veredicto:** ✅ **approve**.

## 7. Acción de seguimiento

- ✅ Cerrar TIR-03: migrar fila del backlog → `docs/progreso/README.md` + commit `docs:` con task ID.
- Nota de dominio: no requiere gate mecánico hoy; el disparador es la doctrina escrita (Fase 0.5).
