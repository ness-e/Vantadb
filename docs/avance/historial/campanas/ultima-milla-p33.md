# P33 — Última Milla (integración producto end-to-end)

> **Plan:** `docs/plans/archive/2026-08-22-vanta-ultima-milla.md` · **Fechas:** 2026-08-22 · **Resultado:** ✅ 10/10
> **Fuente:** auditoría de integración final (`docs/reviews/2026-08-22-auditoria-integracion-final.md`) — los 3 huecos críticos 🔴 + brechas medias + oportunidades de descartes re-evaluados.

## Resumen

El producto funciona end-to-end en los 3 escenarios: desktop embebido, coding agent → proxy con loop agéntico de memory-tools, y server API con skills/wiki. Suites finales: vanta-proxy **89/89**, desktop **88/88**, vanta-memory **472/472** (+precise-tokens), core skills HTTP 16/16.

## Tareas

| Task | Tarea | Commit |
|---|---|---|
| 1 | MEM-50 write-back wired (capture_turn post-forward, H1) | `1b88776c` |
| 2 | MEM-51 O2 interceptor stream + loop agéntico cap 3 iteraciones (H2) | `a9b65224` |
| 3 | MEM-52 fachada wiki_ingest/status MCP, worker split begin/execute (H3) | `dd13e398` |
| 4 | MEM-54 skills CRUD HTTP optimistic lock + owner 404 anti-enumeración (H5) | `9d63bd96` |
| 5 | BND-03 tiktoken-rs 0.12 feature-gate precise-tokens (enmienda D21) | `784b27b9` |
| 6 | MEM-57 parser claude-code classify+extract sin system-reminders | `f76f2c23` |
| 7 | MEM-55 conversation/add→L1 vía ConversationTrigger + HttpCaptureBridge (H6) | `632ac36e` |
| 8 | MEM-53 desktop 7 comandos IPC pipeline (H4) | `f296feee` |
| 9 | MEM-58 consolidación UI ↔ context engine real | `50b463fa` |
| 10 | MEM-56 hook Langfuse OTLP-JSON manual sin SDK, off-by-default | `7f1eab2b` |

## Decisiones

- **D46 (H2):** interceptor O2 elegido por el usuario — loop agéntico dentro del proxy: parsear SSE acumulando deltas, ejecutar `vanta_memory_*` server-side, sintetizar tool_result estándar, re-request; streamear solo el response final.
- **D47:** capture vía WriteBack::track — un solo camino de writes L0.
- **D48:** cap duro 3 iteraciones del loop agéntico.

## Lecciones

- Verify del lead SIEMPRE con `--all-targets` (lección MEM-48 aplicada y reforzada).
- Sub-agentes agotados por memoria en tareas grandes → lead cierra directamente el trabajo heredado (MEM-58).
- Sesión GOV paralela introdujo regresión nextest `-p` scoped (BND-06) y gate docs-coverage bloqueante — resuelto con MCP.md 56 tools documentadas; coordinación inter-sesión es real.

## Deuda colateral registrada

BND-06 (nextest -p roto por filtro GOV-C1) · wiring productivo del host para HttpCaptureBridge pendiente (nota MEM-55) · skill api-reference.md con conteos viejos (GOV-MCPDOCS nota).
