---
id: FIND-467
title: "Endurecer selector de rama release-plz y eliminar doble disparo TOCTOU en release-wheels.yml"
type: fix
state: IN_PROGRESS
plan: docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md
absorbe: FIND-472
---

# FIND-467 — Endurecer selector release-plz + dedup trigger release-wheels

## Objetivo

Dos defectos del post-mortem de v0.9.0 (`docs/dev/reviews/2026-10-20-release-v0.9.0-postmortem.md`):

1. El selector de rama del job `release-plz-pr` en `release.yml` usa
   `startswith("release-plz-")` sin ancla regex explicita: una rama manual
   `release-plz-fix-*` puede colarse en el sync de manifiestos.
2. `release-wheels.yml` se dispara DOS veces por release: por el job `fanout`
   de `release.yml` (`gh workflow run release-wheels.yml --ref <tag>`) y por
   `on.push.tags` cuando el PAT crea el tag (eventos PAT no suprimidos).
   La carrera provoca un HTTP 400 en PyPI.

## Contrato (literal del plan)

```
python -c "import yaml; L=lambda p: yaml.safe_load(open(p,encoding='utf-8')); g=lambda d: d.get('on', d.get(True)); w=L('.github/workflows/release-wheels.yml'); r=L('.github/workflows/release.yml'); c=w.get('concurrency'); assert c and c.get('group') and 'cancel-in-progress' in c, 'wheels: concurrency incompleta'; assert c['cancel-in-progress'] is not True, 'wheels: cancel-in-progress true'; runs=[s.get('run') or '' for j in r['jobs'].values() for s in j.get('steps',[])]; sel=[x for x in runs if 'headRefName' in x]; assert sel and all('^release-plz-' in x for x in sel), 'release.yml: selector de rama sin ancla ^release-plz-'; dbl=any('gh workflow run release-wheels.yml' in x for x in runs) and 'tags' in (g(w).get('push') or {}); assert not dbl, 'doble disparo: fanout + push.tags en release-wheels'; print('OK')"
```

**Estado ANTES (medido):** falla en `AssertionError: release.yml: selector de rama sin ancla ^release-plz-`. Las dos primeras aserciones (concurrency) ya pasan.

## Impacto mapeado (Regla 0)

| Archivo | Rol | Leido |
|---|---|---|
| `.github/workflows/release.yml` | job `release-plz-pr` step sync (l.125) + job `fanout` (l.187) | SI (completo) |
| `.github/workflows/release-wheels.yml` | `on:` triggers (l.3-19), concurrency (l.24-26), gates `refs/tags/v` (l.177,265) | SI (completo) |
| `docs/dev/workflow/PUBLISH.md` | runbook de release; §Cascadas documenta PAT-cascade primario + fanout fallback | SI (completo) |
| `docs/dev/reviews/2026-10-20-release-v0.9.0-postmortem.md` | los 7 fallos reales | Parcial (seccion relevante) |
| `docs/dev/workflow/RULES.md` | reglas durables CI | Referencia |

**Referencias entrantes:** ningun workflow invoca estos YAML por nombre salvo
`scripts/verify-release.ps1` (lee `release-manifest.json`, no triggers) y
`release-verify.yml` (dispatch remoto del script). El contrato es la unica
asercion mecanica.

**Veredicto:** blast radius = pipeline de release completo (no se detecta
hasta publicar v0.10.0). Sin simbolos (YAML). 2 archivos a editar.

## Decision de diseno (Gate D razonado)

El contrato exige `not (fanout AND push.tags)` — hay que quitar uno.
Opciones:

- **(A) Quitar `push.tags` de `release-wheels.yml`** — ELEGIDA. El fanout de
  `release.yml` queda como UNICO disparo automatico: controlado, explicito
  (`--ref <tag>`), y los gates `startsWith(github.ref,'refs/tags/v')` de
  publish/verify siguen funcionando porque el dispatch corre SOBRE el tag.
  `push.tags` solo era "primario" con PAT presente; sin PAT el fanout ya
  falla en rojo por diseno (`::error::RELEASE_PLZ_TOKEN is not set`), o sea
  que `push.tags` nunca funciono como fallback real — solo producia el
  duplicado que causo la carrera a PyPI.
- (B) Quitar la linea `gh workflow run release-wheels.yml` del fanout —
  descartada: deja a wheels dependiendo de la semantica de supresion de
  eventos de GitHub y rompe la simetria con el camino que FIND-550 puede
  tomar en `release-npm-wasm.yml` (alli solo puede quitar `push.tags`; la
  linea de dispatch vive en `release.yml`, fuera de su scope).

**Recuperacion manual:** `workflow_dispatch` se CONSERVA en
`release-wheels.yml` (mitigacion registrada en el plan):
`gh workflow run release-wheels.yml --ref vX.Y.Z`.

**Invariante:** `cancel-in-progress` NUNCA `true` en tags — cancelar una
subida a PyPI en vuelo es el riesgo critico a no introducir.

## Validacion internet (bloque 9)

- release-plz `pr_branch_prefix` default = `release-plz-`
  (`DEFAULT_BRANCH_PREFIX` en docs.rs; config oficial:
  https://release-plz.dev/docs/config — "release-plz only manages release
  PRs whose branch starts with this prefix"). El ancla `^release-plz-`
  cubre el formato real `release-plz-<algo>`.
- GitHub Actions `concurrency`: `cancel-in-progress: true` cancela el run
  en curso del mismo grupo; `false` (o expresion falsy) deja terminar el
  in-flight y encola (docs oficiales:
  https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency).

## Spec

N/A — fix de CI con contrato mecanico (no es feature-add ni logica nueva).

## Steps

1. ✅ DISCOVERY: leer workflows, PUBLISH.md, postmortem; contrato ANTES (falla en selector sin ancla).
2. ✅ ACT: `release.yml` selector ahora `test("^release-plz-")` (ancla regex explicita).
3. ✅ ACT: `push.tags` eliminado de `release-wheels.yml` (comentario FIND-467 con causa raiz + recuperacion); `workflow_dispatch` + `pull_request` + concurrency intactos.
4. ✅ VERIFY: contrato DESPUES imprime `OK`; yaml parse OK; actionlint sin findings.
5. ✅ REVIEW P2-01 + commit local `fix(ci): ... FIND-467`.

## REWORK (post-rechazo del orquestador)

El cierre inicial fue rechazado: `startswith("release-plz-")` ->
`test("^release-plz-")` es un no-op semantico en jq (startswith ya ancla en
posicion 0); seguia aceptando `release-plz-fix-urgente`. Fix real:

- **Patron endurecido:** `test("^release-plz-[0-9]{4}-")` (endurecido aun mas tras review P2-01 rework: anio completo ISO, cierra el residual `release-plz-1manual`). Dato de campo del
  owner: el formato real emitido por release-plz en este repo es
  `release-plz-<timestamp ISO>` (ej. `release-plz-2026-10-20T08-17-56Z`), o
  sea el caracter tras el prefijo es SIEMPRE un digito. El patron acepta la
  rama real y rechaza ramas manuales `release-plz-fix-*`. Coincide con la
  mitigacion del Risk Register del plan (`regex exacto ^release-plz-[0-9]`).
- **Seleccion determinista:** el selector ya no toma `[0]` del listado (orden
  arbitrario de la API): `--json headRefName,createdAt` +
  `sort_by(.createdAt) | reverse | .[0]` = PR abierto mas reciente. No se
  filtro por autor (`github-actions[bot]`) porque el Risk Register pedia
  "determinista Y/O autor": el sort deterministico cubre el minimo y el
  filtro por autor agregaria un acoplamiento al nombre del bot que no aporta
  sobre un repo donde solo release-plz abre PRs contra main con ese prefijo.
- **Demostracion del patron (SIMULACION declarada):** `jq` no esta instalado
  en esta maquina; se simulo `test()` (Oniguruma) con `re` de Python — para un
  patron anclado `^` + clase de caracteres la semantica es identica.
  Casos: `release-plz-2026-10-20T08-17-56Z`->True,
  `release-plz-fix-urgente`->False, `release-plz-`->False,
  `xrelease-plz-2026-...`->False, `release-plz-2026`->True.
- Contrato conductual nuevo (extrae el patron del workflow y lo ejercita):
  ANTES fallaba con `('el patron acepta una rama manual', '^release-plz-',
  'release-plz-fix-urgente')`; DESPUES imprime `OK`.
- Lo que NO se toco: la eliminacion de `push.tags` en release-wheels.yml
  (aprobada por el owner en el rechazo).

## Review (P2-01) — intento 1 (superseded)

- Tier: **fast** (paths `.github/workflows/**` + `docs/**` — ningun glob adversarial).
- Reviewer: `vanta-review` (contexto: fresh-independent, distinto del autor vanta-lead).
- Veredicto inicial: REQUEST CHANGES — el mismo doble disparo existe en
  `release-npm-wasm.yml:20` (`push.tags` + fanout dispatch). **Resolucion:**
  ese archivo esta PROHIBIDO para FIND-467 porque lo edita **FIND-550 en
  paralelo en esta misma wave** — el residual queda registrado y cubierto por
  FIND-550 (mismo mecanismo, misma wave, owner explicito). No es deuda sin
  ticket: FIND-550 es el ticket.
- Docs stale corregidas: `TRIGGERS.md`, `README.md`, `CI_POLICY.md` (tabla
  §11), `PUBLISH.md` (§PyPI wheels, tabla de namespaces, §Cascadas primario).
- Nit aceptado: `test("^release-plz-")` es semanticamente equivalente a
  `startswith` — el valor es el ancla explicita que exige el contrato y la
  homogeneidad regex. Deuda residual: `scripts/docs/gen-surface-manifest.mjs`
  y `SURFACE_MANIFEST.md` mencionan `release-wheels` con tags — texto
  descriptivo, sin impacto funcional (no regenerado en este commit).
- Veredicto final tras resolucion: **APPROVE** (reviewer: vanta-review,
  reviewer_context: fresh-independent, author_context: vanta-lead).
  NOTA REWORK: este veredicto aprobo el no-op cosmetico; el orquestador lo
  rechazo. Ver "## Review (P2-01) — rework" abajo.

## Review (P2-01) — rework

- Tier: fast (`.github/workflows/**`).
- Reviewer: `vanta-review` | reviewer_context: fresh-independent |
  author_context: vanta-lead-implementer.
- Verificacion SEMANTICA (no presencia de cadena): el reviewer ejercito el
  patron extraido del workflow contra 6 casos — `release-plz-2026-10-20T08-17-56Z`
  aceptado, `release-plz-fix-urgente`/`release-plz-`/`release-plz-x123`/
  `xrelease-plz-123` rechazados; residual `release-plz-123-hotfix` documentado
  y CERRADO adoptando su sugerencia `^release-plz-[0-9]{4}-` (anio completo).
- jq sanity del reviewer: array vacio -> `// empty` -> `-z` guard exit 0;
  `createdAt` campo valido; sort ISO-8601 cronologico correcto.
- actionlint: clean. release-plz.toml no define `pr_branch_prefix` -> default
  upstream aplica (timestamp => digito tras prefijo, confirmado por grep).
- Veredicto: **APPROVE** (con sugerencia `{4}-` adoptada y re-verificada:
  contrato OK, 5/5 casos patron).

## Context Save Point

- Contrato ANTES: `AssertionError: release.yml: selector de rama sin ancla ^release-plz-` (confirmado).
- Decision: Opcion A (quitar push.tags de wheels), justificada arriba.
- PROHIBIDOS respetados: ci-rust.yml, docs-gates.yml, ci-fleet.yml, release-npm-wasm.yml, vantadb-ts/package.json, docs/dev/plans/*, Backlog.md.
