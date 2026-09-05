# OLD-12: Pilot Program Formal (Early Adopters)

**Fuente:** Backlog Phase 9 (Old Docs Rescue)  
**Estado:** ✅ COMPLETED (2026-07-26, verificado batch 6: `PILOT_PROGRAM.md` + 3 templates — agreement, feedback, onboarding)  
**Effort:** 🟡 1 sem  
**Dependencia:** PyPI publicado ✅  

## Gate
✅ DO — `docs/operations/PILOT_PROGRAM.md` existe pero es solo spec. El programa formal necesita: landing page, signup flow, SLA template, feedback loop, NPS tracking.

## Objetivo
Convertir el spec de pilot program en un programa ejecutable con materiales para early adopters.

## Archivos

| Archivo | Qué hacer |
|---------|-----------|
| `docs/operations/PILOT_PROGRAM.md` | Actualizar de spec → programa formal con secciones ejecutables |
| New: `docs/operations/pilot-agreement-template.md` | Template de acuerdo de piloto (términos, NDA básico, SLA) |
| New: `docs/operations/pilot-feedback-template.md` | Template de feedback loop semanal |
| New: `docs/operations/pilot-onboarding-checklist.md` | Checklist de onboarding para early adopter |
| `web/src/pages/pilot.astro` o similar | Landing page de pilot program (si web existe) |
| `docs/Backlog.md` | Marcar OLD-12 ✅ |

## Pasos

### 1. Leer spec actual
Leer `docs/operations/PILOT_PROGRAM.md` para entender el alcance actual.

### 2. Actualizar spec → programa formal
Agregar secciones:
- **Objetivo:** qué busca validar el piloto
- **Perfil de early adopter:** criterios de selección (tamaño, caso de uso, stack)
- **Compromisos de VantaDB:** soporte dedicado, SLA, acceso prioritario a features
- **Compromisos del early adopter:** feedback semanal, bug reports, case study potential
- **Duración:** típica 4-8 semanas
- **KPI de éxito:** retention, performance benchmarks, NPS

### 3. Crear templates
- `pilot-agreement-template.md` — Acuerdo simple con términos
- `pilot-feedback-template.md` — Weekly feedback form (qué funciona, qué no, qué falta)
- `pilot-onboarding-checklist.md` — Pasos para integrar a un early adopter

### 4. Landing page (si aplica)
Si existe `web/` con Astro, agregar `src/pages/pilot.astro` con:
- Hero: "Be an Early Adopter"
- Benefits list
- Signup CTA → form o mailto
- Testimonials placeholder

### 5. Verificación
```bash
# Si web existe
cd web && npm run build
```

### 6. Progreso
- Marcar OLD-12 ✅ en Backlog.md
- Agregar entry en progreso/README.md
- Auto-commit
