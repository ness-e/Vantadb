# GOV-D2 — Split del monolito progreso/README.md por campaña

## Metadata
- **Plan file:** docs/plans/2026-08-22-doc-governance-plan.md (NO editable)
- **Creado:** 2026-08-22
- **Appetite:** max 1d
- **Estado:** ✅ COMPLETED (2026-08-22) — 37 archivos campaña, README índice 10KB, dedup ×1, verify verde

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos o por secciones):**
  - `docs/progreso/README.md` — mapa completo de headers (373 `##`/`###`) + bordes de corte verificados línea a línea en las 60 fronteras del split; tramos clave leídos (1-60, 290-430, 1470-1500, 1760-1800, 1838-1852, 3975-4000, 4280-4335).
  - Plan file GOV-D2 (Task 23): contrato, pre-mortem, stop conditions.
- **Referencias hacia dentro (salientes del README):** links relativos a `../CHANGELOG.md`, `../Backlog.md`, `docs/plans/archive/*`, `docs/Investigaciones/*` — se preservan verbatim al mover contenido (mismo depth relativo: campanas/*.md está un nivel más profundo → **los paths relativos `../X` deben volverse `../../X`**). Corrección mecánica en el script: `](../` → `](../../` y backtick-paths no se tocan.
- **Referencias entrantes (consumidores):** rg "progreso/README" en scripts/ dev-tools/ .github/ .opencode/ docs/:
  - `scripts/check-avance-coverage.ps1`: solo lista archivos del dir `docs/progreso` (no parsea README) ✅
  - `.opencode/skills/campaign-executor/tasks/*`: referencias históricas de registro, no consumidores estructurales ✅
  - `docs/book/**`: build artifact regenerable, no fuente ✅
  - `docs/master-index.md` + otros docs: link genérico a `progreso/README.md` (el archivo sigue existiendo como índice) ✅
  - **Veredicto: ningún consumidor automático asume la estructura interna del README.** Split seguro.
- **Zonas ambiguas:** bloque legacy julio-agosto (L427-1484 y L1844-3984) mezcla temas por fecha → cortes contiguos temáticos gruesos documentados en el índice; permitido por step 5 ("misc agrupado documentado").
- **Duplicación conocida:** evento "archivado residuo-consolidado" ×3 (:399 Task5-ref, :411 cierre, :415 cierre detallado) → merge :411+:415 en una sección única conservando hechos únicos.

## Contrato
"README ≤50KB = índice + resumen vivo; cada campaña ≥1 archivo en campanas/; dedup del evento triplicado; 0 links rotos hacia progreso/ desde otros docs; suma bytes ≥97% original menos dedup; spot-checks ×5 OK; markdownlint 0 issues."

## Steps
### Step 1: DISCOVERY + Regla 0 ⬜→✅
### Step 2: Split mecánico por rangos verificados → campanas/*.md ⬜
### Step 3: Dedup residuo-consolidado ⬜
### Step 4: README índice ≤50KB ⬜
### Step 5: Verify (bytes/spot-checks/links/markdownlint) ⬜

## Context Save Point
- Decisión de corte: campañas recientes limpias (P32/P31/P30/P29/P27/P20-TSYS/P26-VS/SKL/GOV); legacy agrupado temático-contiguo (~28 archivos); MEM-21→p27, MEM-41/MEM-39→p29 (estaban mal ubicados bajo Vanta Studio).
