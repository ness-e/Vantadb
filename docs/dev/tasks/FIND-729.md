---
title: "FIND-729: `vantadb-ts` no tiene typecheck ni lint en ningún gate"
kind: task
status: completed
description: "`tsc --noEmit` falla con 4 errores porque `vantadb-wasm/pkg/vantadb_wasm.d.ts` quedó atrás de la fuente escrita a mano, y `npm run lint` con 6 pre-existentes; su único gate es PR-only y el `pre-push` sólo hacía `npm test`. Sincronizar los tipos, arreglar los 6 de eslint y cablear los tres en el pre-push."
id: FIND-729
---

# FIND-729: `vantadb-ts` no tiene typecheck ni lint en ningún gate

## Metadata

- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) §6.b
- **Fuente:** cerrada `FIND-728` (el `pre-push` reparado por `FIND-718` destapó los 4 rojos de `npm test`); al medir el área TS se vio que el hook sólo corre `npm test` y que `tsc`/`eslint` nunca tuvieron gate.
- **Esfuerzo:** 🟢 2-3 h
- **Prioridad:** 🟡 Media-alta — el SDK npm se publica sin typecheck; un `.d.ts` desincronizado llega al consumidor.
- **Tipo:** build/CI (gates del ecosistema JS)
- **Carril:** C
- **Creado:** 2026-11-25

## Contrato

```sh
cd vantadb-ts && npx tsc --noEmit && npm run lint && npm test   # exit 0
node dev-tools/build-wasm-types.mjs --check                      # exit 0
```

y el `pre-push` corre los tres cuando el rango toca `vantadb-ts/**` (hoy sólo corre `npm test`).

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `.githooks/pre-push` lo usa **todo el repo** (un comando mal puesto bloquea pushes legítimos). `vantadb-ts/src/native.ts` lo re-exporta `vantadb.ts` (`export * from "./native.js"`). |
| Callees | `vantadb-ts/src/vantadb.ts` → `vantadb-wasm` (`file:../vantadb-wasm/pkg`, junction), `vantadb-node`. `dev-tools/build-wasm-types.mjs` copia `vantadb-wasm/src/vantadb_wasm.d.ts` sobre `pkg/vantadb_wasm.d.ts`. |
| Implicaciones | `vantadb-wasm/pkg/` es artefacto **gitignored** (`.opencode/rules/js-ecosystem.md` R-2, y `vantadb-wasm/pkg/.gitignore` con `*`): el fix no puede ser commitear el `.d.ts`, tiene que ser regenerarlo con el script del repo. El emisor npm (`release-npm-wasm.yml:81`, `ci-fleet.yml:237`) ya corre el script en write-mode antes de compilar el TS, así que la deriva era **local**: nadie lo corría en la máquina. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (extensión necesaria):** `.githooks/pre-push` (154 L, completo), `dev-tools/build-wasm-types.mjs` (67 L, completo), `vantadb-ts/package.json` (84 L, completo), `vantadb-ts/eslint.config.js` (25 L, completo), `vantadb-ts/src/native.ts` (L860-885 y L940-985), `vantadb-ts/src/vantadb.ts` (L1-40, L430-460, L1870-1889), `vantadb-wasm/src/vantadb_wasm.d.ts` (grep de los 4 tipos + L605-625), `.opencode/rules/js-ecosystem.md`, `ci-fleet.yml` (L215-290).
- **Referencias hacia dentro:** `native.ts` lo importan `vantadb.ts` y `src/__tests__/*`; las métodos con params no usados (`findNodesByLabel`, `graphShortestPath`) están declarados en la interfaz `src/native.ts:131,151` — el nombre del parámetro **no** forma parte del tipo, así que renombrarlo a `_x` no es cambio de contrato.
- **Referencias hacia fuera:** `vantadb-ts` consume `vantadb-wasm` (junction a `pkg/`) — sin `pkg/` sincronizado, `tsc` no resuelve 4 tipos.
- **Veredicto de impacto:** **bajo en código, alto en fricción**: 3 renombres de parámetro + 1 import redundante + 2 casts `any` fuera, 1 artifact regenerado (gitignored), ~12 líneas en el hook. Cero Rust.

## Diagnóstico — los 4 de `tsc`, medidos

```
src/types.ts(56,15):    error TS2305: Module '"vantadb-wasm"' has no exported member 'AstJson'.
src/vantadb.ts(3,15):   error TS2305: ... 'ListOptionsInput'.
src/vantadb.ts(3,33):   error TS2305: ... 'MemoryRecordInput'.
src/vantadb.ts(3,52):   error TS2305: ... 'SearchRequestInput'.
```

**Causa raíz (una sola):** `dev-tools/build-wasm-types.mjs` sustituye el `.d.ts` que genera `wasm-pack` (con `any`) por la copia escrita a mano de `vantadb-wasm/src/vantadb_wasm.d.ts` **= 956 líneas**. El `pkg/` local tenía **390 líneas** (el que deja `wasm-pack` a pelo): nadie corrió el script tras `FIND-156`/`FIND-722`, que añadieron `AstJson`, `MemoryRecordInput`, `ListOptionsInput` y `SearchRequestInput` a la fuente. Los 3 primeros están en `src/vantadb_wasm.d.ts:121,149,168,199`; en `pkg/` sólo aparecía `AstJson` en un comentario.

No se edita a mano: se corre el script (write-mode), que es exactamente lo que hacen CI y el release.

## Diagnóstico — los 6 de `eslint`, medidos

```
src/native.ts
  871:26  'label' is defined but never used               @typescript-eslint/no-unused-vars
  958:5   'maxDepth' is assigned a value but never used
  959:5   'direction' is assigned a value but never used
src/vantadb.ts
   20:3   'ForgetMode' is defined but never used
  448:33  Unexpected any                                   @typescript-eslint/no-explicit-any
  449:30  Unexpected any
```

**Causa raíz:** el único gate que corre `npm run lint` es `release-npm-wasm.yml:104-110` (con `npm run build`, que es `tsc`, en `:100-102`), y **ese workflow sólo dispara en `pull_request`** (`:20-24`): los pushes directos a `develop` —el camino normal de este repo— lo esquivan. `ci-fleet.yml:230-276`, que sí corre en push, sólo ejecuta `vitest run src/__tests__/parity.test.ts`. Resultado: la deuda se acumuló sin que nada se pusiera rojo en `develop`.

**¿Bug real o estilo?** Ninguno de los 6 es un bug, y se arreglan en el código — **cero `eslint-disable`**:

1. `label` (`native.ts:871`) y `maxDepth`/`direction` (`native.ts:958-959`) son parámetros de métodos que **fallan cerrado a propósito** (`FIND-539` label-index inexistente, `FIND-537` `graphBfs` sin parents). Existen por paridad de firma con `vantadb-node`. Convención `_x` + `argsIgnorePattern: "^_"` en la config (patrón estándar, no un silencio puntual).
2. `ForgetMode` (`vantadb.ts:20`) es un import **redundante**: el tipo ya se re-exporta en `vantadb.ts:1882` (`export type { … ForgetMode … } from "./types.js"`). Borrar la línea del import no cambia la superficie pública.
3. Los dos `any` de `closeAsync` (`vantadb.ts:448-449`) son casts defensivos de cuando el `.d.ts` no declaraba `closeAsync`; hoy sí lo declara (`src/vantadb_wasm.d.ts:620`). Se quita el cast y **se conserva** el guard `typeof … === "function"` (protege contra un `pkg/` viejo).

## Steps

1. ✅ Sync de tipos generados (`node dev-tools/build-wasm-types.mjs`) → `tsc --noEmit` y `--check` verdes.
2. ✅ 6 errores de eslint arreglados en el código (3 renombres + 1 import + 2 casts) + `argsIgnorePattern` → `npm run lint` verde.
3. ✅ `.githooks/pre-push`: `tsc --noEmit` y `npm run lint` en la rama TS (guardados por `node_modules` y `pkg/`), y `build-wasm-types --check` en un bloque propio con gatillo `^(vantadb-wasm|vantadb-ts)/` (no necesita `node_modules`).
4. ✅ Verify del contrato + el hook en un worktree descartable; medido el coste añadido.

## Resultado

### Contrato

| Comando | Resultado |
|---|---|
| `cd vantadb-ts && npx tsc --noEmit` | exit 0 (4 errores TS2305 → 0) |
| `cd vantadb-ts && npm run lint` | exit 0 (6 errores → 0) |
| `cd vantadb-ts && npm test` | exit 0 — 26 ficheros / 394 tests |
| `node dev-tools/build-wasm-types.mjs --check` | exit 0 |

### El hook, medido en un worktree descartable

Worktree `tmp/find729-hooktest` sobre `1aba69d58` + junctions de `vantadb-ts/node_modules` y `vantadb-wasm/pkg` (regenerables), `dev-tools/verify.ps1` **stubbeado a `exit 0`** para aislar la rama TS (la etapa Rust es de `FIND-718` y no cambia acá), stdin de push simulado.

| Caso | Resultado |
|---|---|
| Happy path (commit toca `vantadb-ts/**`) | corre `build-wasm-types --check` → `tsc --noEmit` → `npm run lint` → `npm test`; `pre-push: barrier passed. Push allowed.` |
| Happy path, commit toca **sólo** `vantadb-wasm/src/vantadb_wasm.d.ts` | corre `build-wasm-types --check` y **nada más** (no exige `node_modules`) |
| `pkg/` desincronizado (`.d.ts` a mano editado) | BLOCKED: `pkg/vantadb_wasm.d.ts is stale` + `fix: node dev-tools/build-wasm-types.mjs` |
| Error de tipo (`TS2322`) | BLOCKED: `vantadb-ts typecheck FAILED` |
| Variable sin usar (lint) | BLOCKED: `vantadb-ts lint FAILED` |

**Coste añadido al `pre-push`** (máquina del owner, `pkg/` caliente): `build-wasm-types --check` **1,3 s** + `tsc --noEmit` **4,4 s** + `npm run lint` **6,5 s** ≈ **12 s**. `npm test` ya corría (5,3 s). El hook sigue muy por debajo del techo de 1 h — lo que domina el tiempo es `verify.ps1`, no esto.

### Deuda / no tocado

- **`release-npm-wasm.yml` sigue siendo PR-only**: el gate de typecheck/lint existe en CI, pero sólo en PRs. Un push directo a `develop` (o un `--no-verify`) sigue sin typecheck salvo por este hook local. La causa raíz del agujero (gatillo `pull_request` sin `push: branches: [develop]`) es de `release-ci` y quedaba fuera del contrato → fila `FIND-*` sugerida.
- El hook sigue corriendo `verify.ps1` (etapa Rust completa) aunque el push sólo toque `vantadb-ts/` — diseño pre-existente de `FIND-718`, fuera de alcance.

### Review P2-01

- **Revisor:** `vanta-review` (sub-agente, contexto fresco; autor = `vanta-worker`) — `ses_dee35f1f4ffeHgZ00NH0MkzOwj`.
- **Veredicto:** `CHANGES-REQUIRED` sobre la evidencia, no sobre el código («Código correcto; sólo los dos textos de justificación mienten sobre CI»).
- **Contrato re-verificado por el revisor:** `build-wasm-types --check` 0 · `tsc` 0 · `lint` 0 · `npm test` 394/394.
- **Resuelto en esta ronda:**
  1. (required) «sin gate en ningún lado» era **falso**: `release-npm-wasm.yml:100-110` corre `tsc`+`lint` en PR. Reescrito el comentario del hook y esta sección de diagnóstico.
  2. (optional) el `--check` sólo corría dentro de la rama `vantadb-ts/`: ahora es un bloque propio con gatillo `^(vantadb-wasm|vantadb-ts)/` y sin exigir `node_modules` → editar sólo la fuente a mano ya no escapa, y probado en el worktree.
- **Descartado con motivo:** `npx --no-install` (el `npx` local resuelve `typescript` del `node_modules`; `--no-install` está deprecado en npm ≥10 y el guard ya exige `node_modules`); colapsar `exit 1/2/3` del script en un solo mensaje (el script imprime su propio diagnóstico arriba).
- **Confirmado sin cambio:** nombres de parámetros no son contrato (`native.ts:131,151-156` intactas), el re-export de `ForgetMode` preserva la superficie, y ninguno de los 6 errores era un bug real.

### NOTICED BUT NOT TOUCHING

- `.github/workflows/ci-rust.yml` y `web/` aparecen modificados/untracked en `git status`: son de `FIND-730` y de otra sesión. No se commitean acá.

## Invariantes

- `vantadb-wasm/pkg/` **no se commitea** (gitignored); el fix no puede depender del artefacto versionado.
- Nada de `// eslint-disable` ni de bajar reglas a `warn`.
- El `pre-push` debe seguir siendo **<1 h** y no bloquear pushes legítimos: los comandos nuevos se saltan con mensaje (como hoy) si falta `node_modules` o `pkg/`.
- `FIND-730` toca `.github/workflows/ci-rust.yml` en paralelo: no se toca ese fichero.
