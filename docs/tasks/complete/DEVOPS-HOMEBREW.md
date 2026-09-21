# DEVOPS-HOMEBREW: Homebrew formula

## Metadata
- **Plan file:** P8 Post-Launch & Enterprise
- **Fuente:** `docs/Backlog.md:197`
- **Esfuerzo:** 🟢 4h
- **Prioridad:** 🟡
- **Tipo:** CI/CD
- **Turns estimados:** 5-8
- **Estado:** ✅ COMPLETED — 2026-07-26
- **Resultado:** `Formula/vantadb.rb` con livecheck, 4 plataformas, install + test. Placeholder SHA256 — actualizar antes de publish.

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | Homebrew users running `brew install vantadb` |
| Callees | Release binary artifacts (GitHub Releases) |
| Implicaciones | New file `Formula/vantadb.rb`. No afecta código existente. |

## Contrato
"`brew audit --new-formula Formula/vantadb.rb` pasa sin errores críticos. La formula referencia URLs de GitHub Releases y SHA256 checksums."

## Pasos
1. Investigar estructura de Homebrew formula + release binary naming
2. Crear `Formula/vantadb.rb`
3. `brew audit --new-formula Formula/vantadb.rb`
4. Documentar en README cómo instalar via brew
