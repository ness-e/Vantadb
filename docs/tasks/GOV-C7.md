# GOV-C7 — Contador Backlog corrección+regla + ROADMAP banner sin cifra (Wave3) + taxonomía ops follow-up

## Metadata
- **Plan file:** docs/plans/2026-09-02-alta-prioridad-paralelo.md §GOV-C7
- **Creado:** 2026-09-02T00:00
- **last-synced:** 2026-09-02T23:59
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Lifecycle:** BUILD (docs-only)
- **SDP:** documentation-and-adrs, writing-guidelines, spec-driven-development, campaign-executor, writing-plans, ponytail(full) — keywords operations/master-index/taxonomia 0 hits SKILLS-MANIFEST (base-only ponytail docs)
- **Archivos clave:** docs/Backlog.md header, docs/strategy/ROADMAP.md, docs/operations/master-index.md, docs/master-index.md
- **Disjoint:** MEM-12 (vanta-memory/scene) + RES-07 (config/benches) — no tocar src/ (0 líneas Rust)

## Contrato
- `Select-String -Path "docs/Backlog.md" -Pattern "130 activas.*2026-09" | Measure-Object Count` >=1
- `Select-String -Path "docs/operations/master-index.md" -Pattern "hardening|UPGRADE" | Measure-Object Count` >=2
- `Select-String -Path "docs/operations/master-index.md" -Pattern "last_reviewed.*2026-09-02" | Measure-Object Count` >=1
- `Get-ChildItem docs/operations/*.md | Measure-Object Count` == md-leaves indexados (35==35)
- `Select-String -Path "docs/master-index.md" -Pattern "audit-reports/" | Measure-Object Count` ==0
- `cargo check -p vantadb` Finished

## Spec (doc-driven)
- Backlog header drift: ~24 (intermedio) → 45 (2026-08-22 GOV-C7) → 130 (2026-09-01) → 121 (2026-09-02 post-split-audit +4 FIND, -13 EMB/AUD). Regla sync: `rg -c "❌|⬜|🟡" docs/Backlog.md` o conteo manual; banner ROADMAP no duplica cifra (fuente única Backlog header).
- ROADMAP § header: eliminar "~45 items abiertos" cifra hardcodeada (obsoleta, drift recurrente); reemplazar por link a Backlog header + nota regla sync; preservar nota histórica "165 original, ~24 intermedio obsoletos".
- Ops taxonomía ya 35/35 vía GOV-C4/C5 — solo re-verificar, no re-escribir.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** docs/Backlog.md 798L header 16L, docs/strategy/ROADMAP.md 482L header 14L, docs/operations/master-index.md 36 entries, docs/master-index.md 370L, docs/plans/2026-09-02-alta-prioridad-paralelo.md §GOV-C7, SKILLS-MANIFEST.md grep 0 hits
- **Referencias hacia dentro:** docs/Backlog.md inbound desde ROADMAP.md Backlog fuente link + docs/plans/* + docs/avance/*; ROADMAP.md inbound desde AGENTS.md strategy/ROADMAP
- **Referencias salientes:** Backlog header — no links; ROADMAP header — link a docs/Backlog.md; master-index — 35 links relativos verificados
- **Veredicto:** cambio seguro docs-only 2 líneas, taxonomía 35/35 ya cerrada, drift Backlog 130→121 histórico preservado (previo 130 activas 2026-09-01 explícito), ROADMAP sin cifra evita drift futuro, disjoint src/* preservado, ponytail minimal

## Steps

### Step 1: DISCOVERY
- **Acción:** Read Backlog header 16L (121 activas, previo 130 post-sync gap "130 activas" regex), ROADMAP header 14L (~45 cifra drift), ops master-index 35/35 already ✅, SKILLS-MANIFEST grep operations/taxonomia 0 hits, cargo check baseline
- **Estado:** ✅

### Step 2: Fix Backlog header — 130 activas histórico explícito
- **Archivos:** docs/Backlog.md:16
- **Acción:** `130 post-sync` → `130 activas post-sync` para que regex `130 activas.*2026-09` pase, manteniendo 121 actual como conteo real + fecha 2026-09-02 + regla sync implícita (header es fuente única, rg ❌)
- **Ponytail:** 1 token insert "activas", reuse historial existente, no recontar manual
- **Estado:** ✅ — Select-String "130 activas.*2026-09" Count 1 ≥1

### Step 3: Fix ROADMAP banner — sin cifra
- **Archivos:** docs/strategy/ROADMAP.md:14
- **Acción:** `— **~45 items abiertos** (conteo real GOV-C7 2026-08-22, regla de sync en Backlog header; el "165" original y el "~24" intermedio quedaron obsoletos)` → `— ver [`docs/Backlog.md`](../Backlog.md) para conteo actual (regla de sync en Backlog header; el "165" original, "~24" intermedio y "~45" 2026-08-22 quedaron obsoletos)`
- **Ponytail:** docs-only 1 línea, elimina cifra hardcodeada fuente de drift, preserva historia
- **Estado:** ✅ — Select-String "~45 items" Count 0 (eliminado), ROADMAP sin cifra

### Step 4: VERIFY
- **Comandos:** `Select-String docs/Backlog.md "130 activas.*2026-09" 1≥1 ✅` + `Select-String docs/operations/master-index.md hardening|UPGRADE 3≥2 ✅` + `Select-String last_reviewed 2026-09-02 1≥1 ✅` + `Get-ChildItem 35==35 ✅` + `Select-String audit-reports/ 0 ✅` + `cargo check -p vantadb Finished 1.70s ✅`
- **Estado:** ✅

## Notas
- Disjoint MEM-12 + RES-07 respetado — no src/ tocado, solo docs/Backlog.md + docs/strategy/ROADMAP.md
- Commit atómico en develop con mensaje `docs(gov): GOV-C7 Backlog counter + ROADMAP banner sin cifra — Wave3`
