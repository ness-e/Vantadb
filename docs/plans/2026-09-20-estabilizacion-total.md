# Plan de Ejecución: Estabilización total + imagen del proyecto — 2026-09-20

> **Campaign ID:** 550f7331-35bf-469f-ba10-1959407cbaf9
> **Inicio:** 2026-09-20
> **Estado:** 🔄 EN PROGRESO (Fase 0: R-01✅ R-02✅ R-03🔄publish-real-en-curso R-04✅superseded R-05⬜; Fase 1: C-01✅ C-02✅ C-03🔄lint-ok-falta-confirmar-parse C-07🔄25→3prs; resto ⬜; Fase 2 ⬜)
> **Fuente:** análisis profundo 2026-09-20 (inspección repo + docs oficiales Vercel/GitHub) + decisiones owner (5/5)

## Decisiones del owner (vinculantes)

1. **docs/** → reorganizar DENTRO (`docs/user/` + `docs/dev/`), no mover fuera.
2. **web/** → repo propio en **Fase 2**, tras estabilizar (no ahora; rompería PR #182).
3. **TS/npm** → `0.6.0` con publish (todo el ecosistema en el mismo número).
4. **.opencode** → fuera del git YA (`git rm --cached` + `.gitignore`, local intacto).
5. **PRs/ramas** → el lead cura (cerrar con motivo, mergear útiles, borrar ramas).

## Resumen

Cerrar la release 0.6.0 en los 3 registros, dejar `main` verde e igual a `develop`,
organizar CI sin duplicados, reorganizar docs por audiencia, separar tooling personal,
y dejar `web/` lista para extracción. Sin reescribir historia git en ningún momento.

## Fase 0 — Cierre release 0.6.0 (puerta de todo lo demás)

- [x] R-01 · Mergear PR #182 a `main` (`--squash`; main solo permite squash/rebase).
  Contrato: los 11 checks requeridos verdes + Semver verde (0.6.0 absorbe breakings).
  Rollback: `git revert` del squash en `main`.
  ✅ 2026-09-21 mergeado (`cd22b55a`). Incidente: GitHub auto-borró `develop` (head del PR);
  recuperado reseteando a `main` (árboles idénticos verificados) + `deleteBranchOnMerge=false`.
  Deuda: #187 DIRTY (release-plz la actualiza sola; ver R-04).
- [x] R-02 · PyPI `vantadb-py 0.6.0`: diagnosticar por qué el tag no disparó publish
  (sin corrida tag en Actions) y publicar (re-push tag o dispatch documentado).
  Contrato: `pip index versions vantadb-py` muestra 0.6.0 + `pip install vantadb-py==0.6.0` en venv limpio importa `Client`.
  ✅ TestPyPI 0.6.0 verificado (venv limpio, `Client` OK). PyPI prod 0.6.0: NO viable desde tag
  (contenido stale) → se salta a 0.6.1 con #187. Fix durable: pyproject dinámico desde Cargo
  (commit `4af85b54`, wheel local probado `vantadb_py-0.6.0`).
- [ ] R-03 · npm `0.6.0` (bump ✅ · trusted publisher ✅ · dry-run ✅ · wasm 0.6.0 PUBLICADO ✅ 2026-09-22 · fix WSM-06 snippets ✅ cc68b95d · BLOQUEO 2026-09-22: TS smoke falla porque instala `vantadb-wasm@0.6.0` ROTO del registry — requiere `npm unpublish vantadb-wasm@0.6.0` por owner con 2FA y republicar; veredicto owner: re-publicar 0.6.0). Contrato: `npm view` 0.6.0 en ambos.
  (verificar trusted publishing; `npm view` confirma). `vantadb-wasm` igual si aplica.
  Contrato: registros Rust+Python+npm en 0.6.0 el mismo día.
- [x] R-04 · ✅ SUPERSEDED 2026-09-22: #187/#201/#205/#209 cerrados por release-plz sin mergear; sin v0.6.1. El ciclo release es automático; v0.6.1 saldrá cuando corresponda. Sin acción.
  decidir con CI verde (si sus cambios ya viven en `develop` vía #182, cerrarla como superseded con motivo).
- [ ] R-05 · Test de usuario real (post-publish): instalar desde registros en entorno limpio
  y correr QUICKSTART + tutorial 01 paso a paso; registrar fricciones como FIND.
  Contrato: flujo extraño completo sin errores; `tsc`/`lint`/`build` web verdes con deps 16.3.5/0.35.4.

## Fase 1 — Limpieza e imagen (main estable, sin mover carpetas)

- [ ] C-01 · `.opencode` fuera del git: `git rm --cached .opencode` + línea en `.gitignore`
  + retocar referencia en `AGENTS.md`. Local intacto, task-system sigue funcionando.
  Contrato: `git status` limpio de submodule; `git clone` fresco no lo trae; skills raíz intactas.
- [ ] C-02 · Docs por audiencia: `docs/user/` (QUICKSTART, tutorials, FAQ, blog, api/, operations user-facing)
  + `docs/dev/` (plans, tasks, avance, research, reviews, backlog). Mover con `git mv`
  (conserva historia). Actualizar: README links relativos, `llms.txt`, gate-docs paths,
  markdownlint scope, `validate-docs-coverage.ps1`, frontmatter check.
  Contrato: lint+frontmatter+coverage verdes; 0 links rotos (`grep` + spot-check render);
  `docs/CHANGELOG.md`, `docs/api/openapi.yaml`, `docs/api/MCP.md` NO se mueven
  (los consumen release-plz y el chequeo de versiones).
- [ ] C-03 · CHANGELOG (lint MD049/MD003 + frontmatter huérfano ✅ 14ce37ce; WARN "can't parse" desaparecido en runs release-plz — falta confirmación final de parse).
  verificado duplicado) manteniendo entradas; verificar `markdownlint` 0 + release-plz
  vuelve a parsear (desaparece el WARN "can't parse changelog").
- [ ] C-04 · CI sin duplicados: quitar `develop` de `push.branches` en los 7 workflows
  con par push[main,develop]+PR[main] (propuesta FIND-128 §188); verificar con 1 push
  que cada workflow corre 1 vez. Contrato: 0 runs duplicados mismo SHA.
- [ ] C-05 · Renombres reconocibles: `ci-rust-10.yml`→`ci-rust.yml`, `gate-docs-21.yml`→`gate-docs.yml`,
  etc. (los gates usan nombres de check, no de archivo: no se rompen rulesets).
  Actualizar badges del README en el mismo commit. Contrato: `actionlint` 0 + badges resuelven.
- [ ] C-06 · Fast vs Heavy: sacar bench/fuzz pesados del critical path de PR
  (schedule + `workflow_dispatch`), dejando Fast Gate <10 min. Contrato: PR corre solo fast.
- [ ] C-07 · Curar PRs/ramas (25→3 abiertos 2026-09-22; los 3 diferidos a Fase 2; ramas merged borradas; dependabot en solo-alertas). Contrato: PRs abiertos = solo vivos ✅ salvo diferidos.
  borrar ramas mergeadas/stale (orben `gh pr close`, `git push --delete`).
  Dependabot ya está en modo solo-alertas: no vuelve el ruido. Contrato: PRs abiertos = solo vivos.
- [ ] C-08 · CodeQL post-release: activar default setup correcto (el check actual apunta
  a `codeql.yml` inexistente) + triage de alertas. Contrato: check verde o declarado.
- [ ] C-09 · Greptile fuera (owner, 2 min, `github.com/settings/installations`).
- [ ] C-10 · README (paridad ES/EN ✅ 7ae4168e; falta verificar badges verdes + links tras C-02∅).

## Fase 2 — Extracción `web/` (solo tras Fase 0+1 verdes)

- [ ] W-01 · Preparar destino `C:\Users\Eros\VantaDB Proyect\web` como repo git nuevo
  (init + remote nuevo; nombre a confirmar por owner).
- [ ] W-02 · `git mv` de `web/` al repo nuevo **con historia** (`git subtree split -P web`
  o `filter-branch` SOLO sobre la copia, nunca sobre VantaDB) + copiar `.opencode`
  (tooling local, no versionado) + docs/guías marcadas para la web.
- [ ] W-03 · Reconectar Vercel al repo nuevo (Root Directory pasa a `/`; verificar deploy verde).
- [ ] W-04 · CI propia mínima en repo web (build+lint+tsc+e2e) + badges propios.
- [ ] W-05 · En VantaDB: borrar `web/` (`git rm`), limpiar `ci-web-11.yml`/badges,
  política de versiones: web con ritmo propio (cierra contradicción "misma versión").
- [ ] W-06 · Sync de contenido Regla 11: checklist manual por release (cifras web ← BENCHMARKS.md).
  Contrato: Vercel verde en repo nuevo + VantaDB sin referencias rotas a `web/`.

## Gates

- **Gate P:** decisiones 1-5 del owner (2026-09-20) + Gate P anterior (cierre-total).
- **Reglas de ejecución:** `git mv` siempre (nunca reescribir historia); staging selectivo;
  WIP ajeno intocable; `cargo -j 2`; conventional commits con ID; NO PUSH sin verify;
  cada push a `develop` re-dispara CI + release (vigilar `release-plz-release`).
- **Stop honesto:** si Semver/PyPI/npm exigen rediseño → DEFER con diagnóstico, no forzar publish.

## Deuda conocida (no bloquea)

- CHANGELOG WARN "can't parse" (se resuelve en C-03).
- TS `0.5.0` hasta R-03; web `0.2.1` con ritmo propio tras W-05.
- Benchmarks pesados informativos en rojo/cancelado (fuera del critical path tras C-06).

=== RECITATION ===
Objetivo activo: PLAN estabilizacion-total — plan creado
Estado: plan (Fase 0: 5 tasks · Fase 1: 10 tasks · Fase 2: 6 tasks)
Última acción: plan creado desde análisis 2026-09-20 + 5 decisiones owner
Resultado: ✅
Próxima acción: `/pipeline run docs/plans/2026-09-20-estabilizacion-total.md` (Fase 0: R-01 merge #182)
Contrato: plan file existe con fases, contratos, rollbacks y gates; task files bajo demanda
Próxima tarea: R-01 (merge #182 a main)
last-synced: 2026-09-20
=== END RECITATION ===

=== RECITATION C-02 ===
Campaign ID: 550f7331-35bf-469f-ba10-1959407cbaf9
Objetivo activo: C-02 mover SOLO docs sin dependencias a docs/user/
Estado: completed
Última acción: DISCOVERY completo + verifies locales verdes + task file + commit selectivo ccc8ee4e sin push
Resultado: OK
Próxima acción: orquestador: C-10 con docs/tasks/C-02.md como input
Contrato: verificacion: markdownlint 1433 files 0 issues + ](docs/ 49 links 0 rotos + parent-links 0 rotos + spot-check 4/4 + frontmatter 0 faltantes + commit ccc8ee4e (solo C-02.md, hooks verdes) | evidencia: claim subset=∅ → docs/tasks/C-02.md (tabla 7 filas) | confianza: alta; artefactos: docs/tasks/C-02.md; invariantes: PROHIBIDOS intactos (api, operations, workflows, scripts, src, web, desktop, Backlog, avance, plans, tasks, CHANGELOG), NO PUSH; deuda: CI re-run delegado (requiere push); queda_pendiente: orquestador C-10 + decision re-scope C-02 (cerrar NO-APPLICA o wave ampliada)
Próxima tarea si completa: C-10
=== END RECITATION ===
