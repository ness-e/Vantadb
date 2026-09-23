# WEB-DOCS-MOVE — Mover docs históricas web a Vantadb-web + reescribir refs vivas

> **Plan:** `docs/plans/2026-09-20-estabilizacion-total.md` Fase 2 (decisión owner 2026-09-22)
> **Tipo:** docs (campaign_detect_task_type) · **Estado:** ⏳ IN PROGRESS
> **SDP:** documentation-and-adrs, writing-guidelines (cargadas) + base campaign-executor/writing-plans/progreso (auto)
> **Destino:** `C:\Users\Eros\VantaDB Proyect\web\docs\history\` + índice `README.md`

## Contrato

- (a) archivos movidos existen en destino con contenido idéntico (diff vacío salvo índice)
- (b) 0 links rotos en archivos VIVOS de VantaDB (archive/ e historial/ congelados, no se editan)
- (c) `npx markdownlint-cli2 "docs/**/*.md"` 0 issues en VantaDB + lint limpio en web repo
- (d) CI docs intacto (gate-docs paths, frontmatter, validate-docs-coverage — ningún check lista paths movidos)

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** plan Fase 2 (§W-01..W-06), `.opencode/rules/README.md`,
  `definition-of-done.md`, `SPEC.md` (contexto), `gate-docs.yml`, `check-avance-coverage.ps1`,
  `validate-docs-coverage.ps1`, `.markdownlint-cli2.yaml`,
  `docs/reviews/README.md:40-58`, `docs/reports/INDEX.md:23-37`, `docs/avance/fuentes-vivas.md:12-31`
- **Set a mover (13 archivos, existencia verificada):**
  1. `docs/reviews/archive/web-design-audit-2026-08-24.md`
  2. `docs/reviews/archive/research-web-prod-20260825.md`
  3. `docs/plans/archive/2026-08-19-web-design-audit.md` + `.budget.json` (mismo nivel ✅)
  4. `docs/plans/archive/2026-08-04-launch-web-campaign.md` + `.budget.json` (mismo nivel ✅)
  5. `docs/plans/archive/2026-08-25-research-web-quickwins.md` (CORRECCIÓN: el .md vive en
     `archive/`, NO en `docs/plans/` como decía el brief) + `docs/plans/2026-08-25-research-web-quickwins.budget.json` (al mismo nivel NO — está en `docs/plans/`, el .md en `archive/`)
  6. `docs/research/archive/INV-005-error-boundary-web.md` (0 refs en repo ✅)
  7. `docs/glosario/web-to-vantadb-mapping.md` (0 refs en repo ✅)
  8. `docs/avance/historial/campanas/detalle-inv-web-seo.md` + `web-docs-mcp-tooling.md` + `web-launch-campana.md`
- **Referencias entrantes:** 0 markdown links `](...)` vivos hacia el set (grep exhaustivo
  absoluto + relativo). Solo 3 links `](campanas/*.md)` dentro de
  `docs/avance/historial/fuentes/README.md:127,135,139` — archivo congelado, eximido por contrato (b).
  Resto de menciones = citas históricas en backticks/texto dentro de archivos PROHIBIDOS
  (tasks/*, Backlog, CHANGELOG, avance/activo) o frozen (archive/, historial/, dora.md) — no se tocan.
- **Referencias salientes a reescribir (índices vivos editables, 4 filas en 3 archivos):**
  `docs/reviews/README.md:51,58` · `docs/reports/INDEX.md:27` · `docs/avance/fuentes-vivas.md:22`
- **Veredicto:** blast radius = 13 `git rm` + 4 líneas en 3 índices + 14 archivos nuevos en web.
  Sin código, sin API, sin hot path. Gate D: NO dispara. Ningún archivo se excluye:
  avance-coverage itera directorios dinámicamente (no exige paths nominales) → no aplica exclusión A8.

## Gates evaluados

- Gate D: no dispara (docs-only, contrato explícito cubre congelados, motivo: sin-api-sin-hotpath)
- Gate V: no dispara (ningún verify falló 2× mismo-error)
- Gate C: pendiente al cierre (verificar staging selectivo, WIP ajeno `.gitignore` intacto en web)

## Steps

- [x] **STEP 1 — DISCOVERY:** set verificado (`Test-Path` 10/10 md + 3/3 budget),
  `git grep -l` por archivo, links `](...)` = 0 vivos / 3 congelados, lint set = 0 issues
  (4 in-situ + 6 vía temp con misma config), avance-coverage baseline 1038/1038,
  gate-docs + validate-docs-coverage leídos (no listan paths del set). Verify: evidencia arriba.
- [x] **STEP 2 — COPIAR a web:** crear `docs/history/`, copiar 13 archivos byte-idénticos,
  crear `README.md` (tabla origen→destino + nota congelada 2026-09-22). Verify: `Get-FileHash` 13/13 OK ✅.
- [x] **STEP 3 — GIT RM + refs:** `git rm` 12 trackeados + 4 filas en 3 índices
  (reviews/README:51,58 · reports/INDEX:27 · fuentes-vivas:22).
  Verify: staging solo paths esperados; links vivos nuevos = 0; markdownlint VantaDB 1457 files 0 issues ✅.
  Desviación: `docs/plans/2026-08-25-research-web-quickwins.budget.json` es gitignored+untracked
  (`.gitignore:273`) → `git rm` imposible; copiado a destino con hash OK, original conservado
  en disco fuera del repo (git no lo ve). No viola contrato (a).
- [x] **STEP 4 — VERIFY contrato + commits:** (a) 13/13 hashes ✅ · (b) 0 links vivos (3 frozen
  eximidos) ✅ · (c) lint VantaDB 1457 files 0 issues + web history 11 files 0 issues ✅ ·
  (d) frontmatter PASS + validate-docs-coverage OK salvo skills-mirror drift 2/10 PREEXISTENTE
  (no tocado por este cambio) + avance-coverage 1031/1031 (delta -7 IDs, sigue 100%) ✅.
  Commits sin push: VantaDB `d174a868` + web `5b155972` ✅.
- [x] **STEP 5 — CIERRE:** recitation + RESULTADO §7. Progreso parcial: doc-impact ✅, coverage ✅,
  memoria ✅; Trigger 1.D (avance/activo) OMITIDO por prohibido explícito; Backlog sin fila (nada
  que eliminar); CHANGELOG prohibido (sin entrada).

## Deuda conocida (no bloquea)

- 3 links `](campanas/*.md)` en `historial/fuentes/README.md` apuntarán a rutas inexistentes
  tras el move — aceptado por contrato (b), cubierto por tabla origen→destino en índice destino.
- avance-coverage FINAL cambiará (menos IDs en fuentes) — script dinámico, sin exit-code; reportar delta.
- Web repo sin lint config propia — lint verificado con config VantaDB como referencia externa.
