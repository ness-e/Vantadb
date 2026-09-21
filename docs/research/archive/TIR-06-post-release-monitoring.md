# TIR-06: Post-release / monitoring en el loop

- **Fuente:** Backlog P18 — `docs/Backlog.md` línea 452
- **Fecha:** 2026-08-17
- **Tipo:** Investigación/Decisión — read-only, sin cambios de código ni prompts
- **Estado:** cerrado con recomendación **DEFERIR**

## 1. Análisis del gap

**Gap declarado (REPORTE-FINAL §3.3-27):** "Post-release / monitoring en el loop de tarea. El pipeline termina en CLOSE/commit sin verificación post-merge."

**Confirmado en el sistema actual:**
- La state machine C0 termina en `CLOSE` (`state-tools.mjs:50-54`), que permite `bash` para commit. No hay estado ni paso posterior que verifique qué pasó con lo commiteado después del merge.
- `pipeline-full.md` cierra con commit + recitation. Nada monitorea el post-merge.
- El único mecanismo "post" es reactivo: `progreso` Trigger 5 (postmortem tras falla/incidente) y Trigger 4 (sync de reportes de review/audit al backlog). Ambos se disparan cuando algo ya salió mal o ya se revisó — ninguno verifica proactivamente el estado post-merge.

**Qué cubre hoy el DoD monitoring (`definition-of-done.md`):**
- Línea 59-60 (standing DoD, per release): "after releasing, verify in production that the release broke nothing before closing the iteration" + "monitor logs, metrics, and error rates as part of that post-release verification".
- Línea 104 (feature shippable, item c): "Monitoring/observabilidad — log o métrica que evidencie que la feature funciona en producción". Es un gate **de la feature**, no del pipeline: la feature debe llevar su telemetría al shippear.
- Línea 94: "task ends when the work can ship, not when it commits" — el pipeline cierra en CLOSE por diseño: la tarea termina en el commit, la verificación de producción es per-release.

**Qué cubre hoy `progreso` (Triggers 1/4):**
- Trigger 1: migra la tarea completada a `docs/progreso/README.md` y la elimina del Backlog — registro, no monitoreo.
- Trigger 4: sincroniza reportes (`docs/reviews/` → `docs/reports/INDEX.md` → backlog) — feedback de hallazgos, reactivo.
- Trigger 5: postmortem tras incidente — reactivo por definición.
- **Ninguno cubre verificación post-merge proactiva.** Cubren la mitad del lazo: registro + reacción. No la observación de lo shippeado.

**Conclusión del gap:** el hueco real no es "falta un paso en el loop" sino "el lazo release → observar → feedback no tiene dueño mecánico". Pero ese lazo es per-release, no per-task: la mayoría de las tareas del pipeline son internas (índices, bindings, refactors) y no tocan producción; forzarles un paso post-release sería ruido.

## 2. Comparación de opciones

### (a) Paso de verificación post-release opcional en el pipeline
- **Qué:** estado/paso post-CLOSE (o extensión de ACCEPT) que, para tareas que shippean a producción, verifique post-merge: CI verde tras merge, smoke test, checksum/log del deploy.
- **Quién/cuándo:** el lead (único con git push/merge, Regla 7) tras merge a main; o el workflow de release.
- **Pros:** cierra el lazo explícitamente; da evidencia mecánica del post-merge.
- **Contras:** (1) duplica lo que ya hace release-plz + CI (el Release PR exige CI green antes del merge); (2) el loop de tarea es one-task-at-a-time — un paso que espera merge/deploy rompe el handoff; (3) over-engineering para la mayoría de tareas que no shippean; (4) la verificación real de producción (logs, métricas, error rates) ya está normada como DoD per-release (línea 59-60) — el pipeline no debería reimplementarla por tarea.

### (b) Delegación a progreso/registro
- **Qué:** usar el mecanismo existente: Trigger 4 (reportes nuevos → backlog) + Trigger 5 (postmortem) + registro en `docs/progreso/README.md`.
- **Pros:** cero costo, ya funciona, respeta el diseño (registro + reacción).
- **Contras:** no cubre el gap como está pedido: es reactivo. Un release que rompe algo en silencio no genera reporte ni incidente hasta que alguien lo nota. No hay "verificación post-merge" proactiva — solo registro y postmortem.

### (c) No hacer nada
- **Qué:** dejar el pipeline como está; el lazo post-release lo manejan el DoD per-release + `/ship` (GO/NO-GO) + release-plz.
- **Pros:** el DoD ya declara la verificación post-release (línea 59-60) y el monitoreo como parte de ella; el gate (c) de feature shippable ya exige la telemetría de la feature.
- **Contras:** el lazo completo (verificar → aprender → alimentar backlog) sigue sin dueño mecánico; queda en disciplina manual. Es exactamente el patrón que REPORTE-FINAL §3.8 diagnostica: "el sistema declara más de lo que enforceda".

## 3. Recomendación

### **DEFERIR** — no agregar paso al pipeline; el gap se cierra vía P0-1 (evals del pipeline), ya priorizado en REPORTE-FINAL §3.7.

**Justificación:**
1. **El pipeline no es el lugar.** CLOSE/commit termina la tarea por diseño (DoD línea 94: "task ends when the work can ship"). La verificación post-release es per-release (DoD línea 59-60) y ya tiene dueño: `/ship` + release-plz + CI green del Release PR. Un paso post-CLOSE en el loop genérico agregaría un estado que espera merge/deploy — rompe el one-task-at-a-time y penaliza a la mayoría de tareas internas que nunca shippean.
2. **La opción (b) no cubre el gap pedido** (es reactiva), y la (a) duplica infra existente. Ambas son peores que el status quo para el costo.
3. **El hueco real (feedback mecánico post-release) ya está priorizado como P0-1** ("Harness de evals del pipeline" — log por tarea, tasa de primer intento, falsos positivos, regresión 0) y P3-1 (DORA / rework). Instrumentar la North Star convierte la "regresión 0" de slogan en medición y da el feedback al backlog que hoy falta — sin un estado nuevo en la máquina C0.
4. **Cierre mínimo del lazo mientras tanto (opcional, si el lead lo quiere):** una línea en `pipeline-run.md` en CLOSE para tareas que shippean: "si la tarea shippea a producción, la verificación post-release es per-release (DoD `definition-of-done.md:59-60`), no per-task — el lead la ejecuta con `/ship`". Costo ~5 min, no cambia la máquina de estados.

**Qué NO hacer:** (a) agregar estado de post-release al loop de tarea — WONTFIT por over-engineering y duplicación; (b) declarar que progreso "cubre" el gap — no lo cubre, es reactivo.

## 4. Referencias

- `REPORTE-FINAL.md` §3.3-27 (gap), §3.7 P0-1/P3-1 (priorización), §3.8 (patrón "declara más de lo que enforceda")
- `definition-of-done.md:59-60` (post-release + monitoring), `:94` (task ends when it can ship), `:104` (feature (c) monitoring)
- `.opencode/skills/progreso/SKILL.md` Triggers 1, 4, 5 (registro + reacción, no verificación proactiva)