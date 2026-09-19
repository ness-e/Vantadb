# FIND-123 — fuentes en `vs-table.tsx` (Regla 11)

> Campaign: 0ad2d7e2-94e3-4313-8f5c-e8d57c08a6af · Plan: `docs/plans/2026-09-19-ci-green.md` · Wave0 segunda en secuencia (FIND-124 ✅ antes) · Ruta: vanta-worker · Branch: develop · Appetite: max 1h · Esfuerzo: 🟢 · Prioridad: 🟢 Baja

## Objetivo

Cada celda comparativa de `web/src/components/vanta/vs-table.tsx` con fuente o calificada honestamente (Regla 11: claims numéricos DEBEN salir de `docs/operations/BENCHMARKS.md`; web AGENTS.md). `vantadb:"1.2ms"` OK (fuente `docs/operations/BENCHMARKS.md:34` p50 HNSW 10K); 5 celdas competidoras sin cita (`pinecone:"~50-150ms"`, `weaviate:"~20-80ms"`, `chroma:"~5-30ms"`, `"$1,800/mo"`, `"$600/mo"`).

## Contrato

1. Cada celda con fuente o calificada (aprox/fechado).
2. Resto de filas del archivo verificado.
3. `npx tsc --noEmit` en `web/` si se toca código (o N/A con motivo).

## Acceptance criteria (del plan)

- (a) fuente por celda o calificación honesta;
- (b) resto de filas del archivo verificado;
- (c) tsc si se toca código.

## Archivos

- **Clave:** `web/src/components/vanta/vs-table.tsx` (celdas 23-26, 78-79 + resto de filas 29-82).
- **Relacionados:** `docs/operations/BENCHMARKS.md` (fuente de verdad, §1 línea 34 p50 10K = 1.2ms); `web/src/lib/dictionaries.ts:1465-1482,2954-2971` (solo claves `vsTable.*` de labels — valores de celdas hardcodeados en el tsx, no requieren cambio de dict).
- **Prohibidos (NO tocar):** rediseñar la tabla o la página; `reparacion.bat`; `.opencode`; `Justfile`; `ocr-*`; `completions/*`; `desktop/src-tauri/Cargo.lock`; stash@{0} GOV-C4; `docs/Backlog.md`; plan file (solo recitation al cierre); `C:/Users/Eros/.vantadb*` (datos vivos); `src/`; `desktop/` (FIND-124 ✅); resto de `web/` fuera del archivo.

## Dependencias

- Wave0 segunda en secuencia (FIND-124 ✅; archivos disjuntos `desktop/` vs `web/src/components/vanta/vs-table.tsx`).
- Sin bloqueantes.
- NextTask tras cierre: FIND-125 (la ejecuta el orquestador, no vos).

## Referencias

- Rules: `.opencode/rules/frontend-web.md` (lectura completa hecha — R-FE-4 light-only, R-FE-2 sin deps nuevas, R-FE-5/6 N/A a este cambio).
- Refs: `.opencode/references/definition-of-done.md` (standing checklist); `SPEC.md` raíz (sin cambios — 0 greenfield); Tabla Spec: N/A (docs/contenido).
- Commands: `.opencode/commands/pipeline.md` (ejecución).

## Spec

N/A — docs/contenido, cero lógica nueva, cero símbolos públicos nuevos. Cambio: footnote de fuentes/calificación + calificación de celdas competidoras como estimaciones aproximadas fechadas. Sin API, sin comportamiento, sin tipos nuevos.

## Skills (SDP Paso 0b, phase=BUILD, keywords [vs-table, comparativa, fuentes, Regla-11, docs], ≤8)

`campaign_discover_skills_v2` real ejecutado 2026-09-19. SDP: documentation-and-adrs, frontend-ui-engineering, source-driven-development, incremental-implementation, test-driven-development, context-engineering (+ base auto: campaign-executor, progreso, ponytail).

- `documentation-and-adrs` — sugerida del plan; documentar por qué (fuente vs calificación) sin ADR nueva (tiny, no tradeoff arquitectónico).
- `frontend-ui-engineering` — toca `web/` tsx; accesibilidad del footnote (`<p>`, sin targets nuevos), sin romper diseño.
- `source-driven-development` — BENCHMARKS.md es la fuente; competidoras sin fuente sólida → calificar, no inventar.
- `incremental-implementation` — 1 slice vertical (footnote + calificación) → verify → commit.
- `test-driven-development` — N/A lógica (contenido-only): RED/GREEN no aplica; verificación = `tsc` + inspección visual de celdas.
- `context-engineering` — context pack: rules → plan → slice + BENCHMARKS.md:34 + dictionaries.ts.

Las 2 restantes del SDP (`design-taste-frontend`, `doubt-driven-development`) no se cargan: sin rediseño visual ni trust boundary (scope discipline, YAGNI).

## Herramientas + MCP

- `grep` (celdas + resto filas + dict keys) ✅ hecho.
- `npx tsc --noEmit` en `web/` (se toca código tsx) — obligatorio.
- `campaign_verify_cmd` (bug exit -1 conocido → bash directa + mención en RESULTADO).
- `codegraph` N/A (1 archivo web, ya leído completo).
- Cargo N/A (no se toca Rust).
- Internet: SOLO para fuentes competidoras con URLs verificadas; sin fuente sólida → calificar como aproximado/fechado Sep 2026, NUNCA inventar fuente. Decisión: calificar (pre-mortem 1 del plan); latencias/costos competidores varían por tier/región y no hay bench reproducido en repo → footnote honesto.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `web/src/components/vanta/vs-table.tsx` (199 líneas); `docs/operations/BENCHMARKS.md` (§1-§2, fuente línea 34); `docs/plans/2026-09-19-ci-green.md` (Task 2 §67-82); `.opencode/rules/frontend-web.md` (completa); `.opencode/references/definition-of-done.md` (completa); `web/src/lib/dictionaries.ts` (grep `vsTable`, 36 matches — solo labels).
- **Referencias hacia dentro (qué importa el archivo):** `Check, ArrowRight` (lucide-react), `Link` (next/link), `Reveal` (./reveal), `VANTA` (./vanta-data), `useLanguage` (@/lib/language-provider). Cambio no toca imports.
- **Referencias entrantes (quién usa el archivo):** `/why-vantadb` (y posiblemente home) renderiza `<VsTable/>`; web AGENTS.md lo lista como vivo (NO muerto). Cambio solo añade footnote + deja celdas con `~`/calificación — sin cambio de props/API, sin riesgo para consumidores.
- **Veredicto de impacto:** 🟢 mínimo — 1 archivo, sin API, sin i18n, sin deps, sin estilos globales. Rollback = `git revert` del commit.

## Investigación código (DISCOVERY)

N/A código; DISCOVERY = archivo completo leído + BENCHMARKS.md como fuente + verificación fila por fila:

| Fila | Celdas | Veredicto |
|------|--------|-----------|
| 0 Latency | `1.2ms` ✅ fuente BENCHMARKS.md:34 (p50 HNSW 10K, 128d cosine) · competidoras `~50-150/~20-80/~5-30ms` ya con `~` pero sin fuente → calificar en footnote como rangos aprox. públicos Sep 2026, no reproducidos | fuente + calificación |
| 1 Network hops | `0/1+/1+/0-1` — topología (embedded in-process vs cloud) | simplificación honesta, footnote |
| 2 Deployment | `pip install/Cloud account/Docker cluster/pip install` — install paths típicos | simplificación honesta |
| 3 Crash recovery | `WAL + CRC32C/Managed/WAL/Limited` — VantaDB real (WAL core); resto simplificado | simplificación honesta |
| 4 Hybrid search | `BM25 + HNSW · RRF/Vector only*/BM25 + HNSW/Vector only` — `*` de Pinecone SIN footnote existente → añadir explicación (`*` = keyword/sparse vía índice separado, no híbrido nativo único) | footnote `*` |
| 5 Data egress | `None/Cloud/Self-host or cloud/None` — modelo de despliegue | simplificación honesta |
| 6 Cost @ 1M | `$0/$1,800/mo/$600/mo/$0` — `$0` = self-host sin infra incluida; competidoras estimaciones públicas por tier/región → calificar fechado Sep 2026 | calificación fechada |

## Investigación problema

Fuente-por-celda vs calificación honesta: si la fuente no existe o es débil, calificar es correcto, no es fracaso (pre-mortem 1). Competidoras: sin bench reproducido en repo + precios por tier/región → footnote honesto fechado. VantaDB latency: cita exacta BENCHMARKS.md.

## Investigación internet

N/A — sin URLs competidoras con fuente sólida verificable en budget tiny; calificar como aproximado/fechado Sep 2026. Digest: no se inventa ninguna fuente (Regla 11). Deuda: ninguna (la calificación ES el cumplimiento del contrato).

## Steps atómicos

- [x] Step 1 (único, ~100 líneas): añadir footnote de fuentes bajo la tabla en `vs-table.tsx` (VantaDB 1.2ms → BENCHMARKS.md p50 10K; competidoras = estimaciones aprox. Sep 2026 no reproducidas; `*` Pinecone explicado; costos estimados por tier/región) vía `tt(key, fallback)` sin tocar dictionaries + `npx tsc --noEmit` en `web/` ✅ (0 errores) + commit selectivo. [Contrato a/b/c]

## Context Save Point

- Estado: DISCOVERY completo, listo para ACT Step 1.
- Próximo: editar `vs-table.tsx` (inserción tras `</table></div></Reveal>` del bloque tabla, antes del CTA `Reveal delay={120}`), luego `npx tsc --noEmit` en `web/`.
- Gates: D evaluado — blast radius 1 archivo, sin símbolos públicos nuevos, contrato claro → NO disparado (sin `question` tool disponible en este runner; registrado aquí).

## Cierre (al completar)

- Verify contrato (a/b/c) + `campaign_verify_cmd` (o bash directa por bug exit -1 + mención).
- OCR delegation N/A-justificado si es contenido-only (sin lógica, sin trust boundary).
- DoD 3 niveles: Correctness (a/b/c) + Quality (scope 1 archivo, lint/fmt) + Ship (tsc verde, revert limpio).
- P2-01 lo hace el orquestador (no vos).
- RESULTADO §7 obligatorio.
