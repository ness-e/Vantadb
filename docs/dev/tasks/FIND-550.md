# FIND-550 — Re-inyectar `vantadb-node` en `optionalDependencies` en `release-npm-wasm.yml`

- **Estado:** ✅ COMPLETED
- **Tipo:** bug-fix (packaging / CI release)
- **Prioridad:** 🔴 P0 · Appetite: 2h · Wave 0
- **Origen:** Plan `docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md` Task 7
- **SDP:** ci-cd-and-automation, shipping-and-launch, source-driven-development

## Objetivo

El step "Rewrite dependency for publish" del job `publish-ts` elimina `vantadb-node`
(entradas `file:`) para que `npm ci` no resuelva una versión aún no propagada
(ETARGET, trampa 5 de PUBLISH.md), pero **nunca lo restaura** antes de
`npm publish`. El tarball publicado sale sin `optionalDependencies.vantadb-node`
→ quien instala `vantadb` en Node.js no recibe el binario nativo NAPI-RS y cae
en silencio a WASM (rompe EGO-17).

## Causa raíz

El borrado se hace sobre `package.json` en disco (mismo fichero que `npm pack`
empaqueta — `npm pack` incluye el `package.json` tal cual está en disco;
ver docs.npmjs.com/cli/v11/commands/npm-publish). No existe step de
restauración entre `npm ci` y `npm publish`.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `.github/workflows/release-npm-wasm.yml` (277L),
  `vantadb-ts/package.json` (77L), `vantadb-node/package.json` (cabecera), plan Task 7, PUBLISH.md §vantadb-node
- **Referencias hacia dentro:** `publish-ts` steps → `vantadb-ts/package.json`,
  `scripts/smoke-pack.mjs` (TS-07), `vantadb-node/package.json` (versión)
- **Referencias entrantes:** `docs/dev/workflow/PUBLISH.md` (runbook),
  `vantadb-ts/src/native.ts` (`import("vantadb-node")` — fallback WASM si falta)
- **Veredicto:** blast radius = tarball `vantadb` en npm + consumidores Node.js.
  Sin tests de CI locales rotos (el path `file:` solo se toca en el job de publish).
- **Archivos a tocar:** SOLO `.github/workflows/release-npm-wasm.yml`

## Diseño de la corrección

Restaurar `optionalDependencies["vantadb-node"] = <pkg.version>` DESPUÉS de
`npm ci` + build + rewrite, y ANTES de `npm pack`/smoke/publish:

- **Versión exacta:** `require('./package.json').version` de `vantadb-ts`
  (vantadb-node iguala la versión del workspace desde su primer publish —
  PUBLISH.md:206; hoy nunca publicado en npm, 404 documentado).
- **optionalDependencies, no dependencies:** npm tolera fallo de optional deps
  (docs.npmjs.com/cli/v11/configuring-npm/package-json#optionaldependencies) —
  una instalación fresca no rompe mientras `vantadb-node` no exista en npm,
  y el día que se publique, los usuarios reciben el binario nativo.
- **Aserción (Risk Register):** el mismo step falla si falta
  `optionalDependencies['vantadb-node']`, si sigue en `dependencies`, o si la
  versión inyectada ≠ `pkg.version`.

## Steps

1. ✅ Añadir step "Restore vantadb-node optionalDependency (FIND-550)" tras
   "Rewrite dependency for publish" y antes de "Verify package.json integrity (TS-05)"
2. ✅ Contrato python → `OK`
3. ✅ Validar YAML parsea + `npm pack --dry-run` local: simular set + verificar
   `optionalDependencies.vantadb-node` con versión exacta en el manifest empaquetado
4. ✅ Review P2-01 (tier: CI → fast) + commit local

## Contrato

```
python -c "import yaml; d=yaml.safe_load(open('.github/workflows/release-npm-wasm.yml',encoding='utf-8')); st=d['jobs']['publish-ts']['steps']; ix=lambda f:[i for i,s in enumerate(st) if f(s.get('run') or '')]; q=chr(39)+'vantadb-node'+chr(39)+']='; m=lambda r: (lambda t: 'optionalDependencies' in t and ('optionalDependencies.vantadb-node=' in t or q in t))(r.replace(' ','')); ci=ix(lambda r: 'npm ci' in r); pub=ix(lambda r: 'npm publish' in r); rs=ix(m); assert ci and pub, 'faltan npm ci / npm publish'; assert rs, 'ningun step restaura optionalDependencies[vantadb-node]'; assert ci[0] < rs[0] < pub[-1], ('orden invalido', ci, rs, pub); print('OK')"
```

## Evidencia ANTES

`AssertionError: ningun step restaura optionalDependencies[vantadb-node]`
(npm ci y npm publish presentes; falta el step de restauración).

## DoD

- Contrato `OK`
- Step con aserción fall-fast antes de publish
- `optionalDependencies.vantadb-node` = versión exacta del paquete en el manifest empaquetado
- Commit: `fix(release-npm): restore vantadb-node optionalDependency before npm publish` (FIND-550)

## Deuda conocida

- `vantadb-node` aún no existe en npm: el optionalDependency es forward-looking;
  documentado en PUBLISH.md:206 y en la fila de riesgo del plan (carrera de orden
  de publicación — `release-npm-node.yml` corre manual pre-pub).

## Evidencia DESPUÉS

- Contrato: `OK`
- `npm pack` real sobre vantadb-ts: tarball `vantadb-0.9.0.tgz` →
  `package/package.json` contiene `optionalDependencies: {"vantadb-node":"0.9.0"}`
  (versión exacta = pkg.version). Verificado localmente; package.json restaurado.
- Review P2-01: vanta-review (contexto distinto) → APPROVE.
