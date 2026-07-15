---
description: Pre-push certification — all layers, all skills
---

Cargá la skill vantadb-certify.

Ejecutá las 8 layers SECUENCIALMENTE. Detenete al primer error.

Para cada layer:
1. Mostrá "Layer N: [nombre]" antes de empezar
2. Ejecutá el/los comandos mecánicos reales
3. Si falla → mostrá el error exacto + "❌ LAYER N FAILED — abortando"
4. Si pasa → mostrá "✅ Layer N: [nombre]"

Si el diff (`git diff --name-only HEAD`) está vacío, usá `git diff --name-only HEAD~1`.

Capas CI/CD Parity (7a): Para cada cambio en Cargo.toml/package.json/pyproject.toml,
verificar que los .github/workflows/*.yml reflejen las nuevas dependencias y env vars.
Si el diff omite actualizar un workflow → FAIL.

Skills de review (7b): cargar una por una con `skill <nombre>`.
Cada skill puede vetar el push. Si veta, registrá su objeción y abortá.

Al final:
- Si todas las layers pasaron ✅: "✅ CERTIFY PASSED — safe to push"
- Si alguna falló ❌: "❌ CERTIFY FAILED — fix errors above before pushing"
