---
description: "Unified VantaDB audit pipeline — all skills, all tools, all phases"
---

# /audit — VantaDB Audit Pipeline

Audita el proyecto completo en un pipeline de fases secuenciales. Como `/pipeline` pero para revisión.

## Usage

| Comando | Qué hace |
|---------|----------|
| `/audit` | Pipeline completo — fases 1 a 8 |
| `/audit quick` | Solo fase 1 (checks CLI) — ~2min |
| `/audit certify` | Pre-push gate: fases 1 → 2 → 3 → 4 → 8 |
| `/audit review` | Revisión profunda: fases 4 → 5 → 6 → 7 |

## Architecture

```
/audit
  │
  ├─ Phase 0: Pre-check (git diff, estado del repo)
  ├─ Phase 1: CLI Mechanical  ← audit-all.ps1
  ├─ Phase 2: Security        ← +skill security-and-hardening
  ├─ Phase 3: Performance     ← +skill performance-optimization + cargo bloat
  ├─ Phase 4: Code Review     ← +skill code-review-and-quality + ponytail-review
  ├─ Phase 5: Root Cause      ← +skill systematic-debugging (si hay issues)
  ├─ Phase 6: Deep Module     ← +skill review-deep (per-module loop)
  ├─ Phase 7: Full ISO        ← +skill vantadb-full-review
  ├─ Phase 8: Certify         ← +skill vantadb-certify + nocturnal_suite
  │
  └─ Report → docs/audit-reports/audit-<fecha>.md
```

## Implementation

1. Cargá `skill vantadb-audit` al inicio
2. Seguí las fases secuencialmente según el modo
3. Cada fase: mostrá "Phase N: [nombre]" → ejecutá → ✅/❌
4. Skills review cargadas con `skill <nombre>` — pueden vetar
5. Si CLI falla (Phase 1), abortar inmediatamente
6. Al final: reporte en `docs/audit-reports/` con timestamp
