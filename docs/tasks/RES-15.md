# RES-15 — Institucionalizar meta-001 C: split backlog negocio/técnico (RES-15-C)

- **Plan:** `docs/plans/2026-09-03-quality-gtm-wave.md` Task 11 · **Ruta:** vanta-lead (docs/process) · **Wave3/5** paralelo con RES-03
- **SDP:** base-only (keywords: backlog, split, negocio, rg-counter) — tarea docs/proceso pura, sin candidatos del manifest
- **Criterio (Gate P):** lo que requiere agente/código → técnico (queda); lo que requiere abogado/plata/decisión humana/publicación → negocio (se mueve)

## Impacto mapeado (Regla 0)

**Leídos completos:** `docs/Backlog.md` (788L), `docs/avance/meta.md` (199L), `pipeline-full.md`.
**Enumeración ID-por-ID en Backlog.md** (rg, pre-mortem #1 doble-match): cada ID movido aparece exactamente en su fila + referencias explícitas en Exec Summary (líneas 28/29/31/40/48), nota sección humanas (89) y contador (16) — todas se reescriben sin el ID literal.

**Filas movidas (15, criterio verificado fila-por-fila):**
| ID | Motivo negocio |
|---|---|
| LEG-01 | abogado + pago USPTO/EUIPO |
| MKT-04 | publicación Reddit con identidad humana (REDDIT_POSTS.md:13 lo confirma) |
| CLD-01/02/04 | producto/cloud/ventas — cuenta Fly.io con pago, pitch, pilot real |
| BIZ-01b | enterprise — decisión de negocio |
| PRO-01..06 | producto Pro — trigger de inicio es negocio; implementables por agentes cuando arranque Pro (nota en destino) |
| DISC-01/02 | community ops, UI manual Discord no-API-accessible |
| BND-07 | DNS/invite externos, acción del owner |

**Decisión documental — BLOG-CTA:** queda TÉCNICO. La fila = fix CTA + metadata + redactar posts 6-7 (markdown escribible por agente en `web/`); sólo la publicación final es humana, lo que aplica a todo contenido y no justifica fila de negocio. (Instrucción de la tarea: "decidir en discovery y documentar".)
**Quedan en técnico P6:** MKT-18f, MKT-18i, BLOG-CTA. **DISC-03** queda (ICEBOX, no se mueve).
**Referencias entrantes (pre-mortem #2):** `docs/strategy/ROADMAP.md:14,429` · `docs/strategy/REDDIT_POSTS.md:11` · `docs/strategy/GO_TO_MARKET.md:379,408,409,420` → se actualizan con redirect "→ docs/Backlog-negocio.md". Prosa histórica (`docs/master-index.md:167`, `docs/book/src/case_studies/index.md:4`, archive/snapshots/changelog/memory) se deja intacta: cita el ID, que sigue resolviendo en Backlog-negocio.md.
**Parser /pipeline (pre-mortem #3):** lee solo `docs/Backlog.md` → documentar en `meta.md` que el triage técnico no ve filas de negocio (deseado: no contaminan métricas de prioridad).
**GOV-TK5:** split del Manual Estratégico (otro split) — se enlaza desde meta.md, fila intacta.
**Veredicto:** blast radius = 6 archivos docs, cero código, cero BENCHMARKS. Race Backlog.md: re-leer disco + commitear ya (regla demostrada hoy).

## Steps

- [x] S1 Discovery + este task file
- [x] S2 Crear `docs/Backlog-negocio.md` (cabecera + criterio + 15 filas con columnas de origen)
- [x] S3 Re-leer `docs/Backlog.md` del disco → aplicar las 17 ediciones (filas + summaries + contador + cross-link) → commit inmediato
- [x] S4 `meta.md` regla dos-backlogs + sync; cross-ref `ROADMAP.md` (sin cifras, GOV-C7) + redirects `REDDIT_POSTS.md`/`GO_TO_MARKET.md`
- [x] S5 Verify contrato mecánico: Test-Path ✅ · rg grupo negocio =16 ≥14 ✅ · 15 IDs movidos rg=0 en técnico ✅ · cross-links+contadores (107=123−16; negocio 15 filas-data, 19 líneas `^\|`X incl. 4 de la tabla borderline) ✅ · markdownlint-cli2 0 issues en los 6 archivos tocados ✅
- [x] S6 Cierre: commit, RES-15 eliminada, avance, memory decision, recitation

## Contrato (del plan)

- `Test-Path docs/Backlog-negocio.md` True con cabecera + criterio + tablas
- `rg -c "LEG-01|CLD-0[124]|BIZ-01b|PRO-0[1-6]|DISC-0[12]|BND-07|MKT-04" docs/Backlog-negocio.md` ≥14
- Cada ID movido: `rg -c` en Backlog.md == 0
- Cabeceras con cross-link + contadores coherentes con rg
- `npx markdownlint-cli2 docs/Backlog-negocio.md` 0 issues
