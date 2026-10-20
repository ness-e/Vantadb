---
title: Runbook - Rotacion de los tokens de publicacion
kind: runbook
status: active
description: "Rotacion de los 6 tokens de publicacion (PyPI, TestPyPI, npm, crates.io, GitHub PAT, providers) que quedaron expuestos en texto plano durante el release 0.9.0. Orden, pasos por registry, verificacion y las reglas para no volver a exponerlos."
tags: [vantadb, ci, security, publish, release, runbook]
---

# Runbook - Rotacion de los tokens de publicacion

> **Por que:** durante la sesion del release v0.9.0 (2026-10-20) los tokens de
> publicacion se escribieron en texto plano en una conversacion. **Un token
> exposure es una credencial robada**, aunque el chat sea "privado": queda en el
> historial, en los logs del agente y en cualquier export. La rotacion no es
> opcional.
> Backlog: `BIZ-18` (rotacion) · `BIZ-19` (token de crates.io con scope `publish`).
> Post-mortem: `docs/dev/reviews/2026-10-20-release-v0.9.0-postmortem.md` §7.

## 0. Regla de oro (lee esto antes de tocar nada)

**Un token no vuelve a escribirse en ningun chat, issue, PR, log ni commit.**

- Se introduce **solo** por la via oficial de cada registry: el panel web o la
  CLI del registry.
- Al repo se actualiza con `gh secret set` **desde tu shell local**, no pegando
  el valor en ningun sitio donde yo lo lea:

  ```powershell
  # PowerShell: lee del portapapeles para que el valor no quede en el historial
  Get-Clipboard | gh secret set PYPI_TOKEN --repo ness-e/Vantadb
  Clear-Clipboard
  ```

  ```bash
  # bash/macOS:同等 — no pongas el valor literal en la linea de comando
  gh secret set NPM_TOKEN --repo ness-e/Vantadb   # te pide el valor por stdin
  ```

- `gh secret list` **muestra solo el nombre y la fecha de actualizacion**, nunca
  el valor: es la unica vista de los secrets que se puede hacer sin riesgo.
- Si un token vuelve a aparecer en texto plano: **rotables**, no lo "corrijas".

## 1. Inventario de los 6 tokens

| # | Token | Registry | Secret en el repo | Quien lo emite | Se usa para |
|---|-------|----------|-------------------|----------------|--------------|
| 1 | crates.io API token (scope `publish`) | crates.io | `CARGO_REGISTRY_TOKEN` | crates.io > Account > Access Tokens | Publicar crates que **no** existian (bootstrap). El publish normal va por OIDC y no lo necesita. |
| 2 | PyPI API token | PyPI | `PYPI_TOKEN` | pypi.org > Manage > API tokens | `vantadb-py` + los 7 adapters (`vantadb-{crewai,dspy,haystack,langchain,letta,llamaindex,mem0}`) |
| 3 | TestPyPI API token | TestPyPI | `TEST_PYPI_TOKEN` | test.pypi.org > Manage > API tokens | staging de los mismos paquetes |
| 4 | npm automation token | npm | `NPM_TOKEN` | npmjs.com > Access Tokens > Automation | `vantadb`, `vantadb-wasm`, `vantadb-node` (+ los packages de providers/adapters si se publican desde npm) |
| 5 | GitHub PAT (repo / workflow) | GitHub | como secret de workflow (`RELEASE_PLZ_TOKEN` u otro) | github.com > Settings > Developer settings > PAT | Que release-plz pueda crear el tag y el release. **Es el mas peligroso**: da escritura al repo. |
| 6 | Tokens de los providers | GitHub | segun `release-providers.yml` | igual que #5 | Publicar los 3 providers desde el propio repo |

**Que NO hay que rotar:** los tokens OIDC de Trusted Publishing. No son tokens:
son un permiso por workflow, se renuevan solos y no se pueden robar de un
secret (ver `docs/dev/workflow/PUBLISH.md`, seccion de auth model).

## 2. Orden de rotacion

Se rota de menos blast radius a mas blast radius, y **en este orden** para que
un fallo a mitad no deje el release sin publicar:

1. **TestPyPI** (#3) - no publica nada real; sirve de ensayo del procedimiento.
2. **PyPI** (#2) - los 7 adapters + `vantadb-py`.
3. **npm** (#4) - `vantadb`, `vantadb-wasm`, `vantadb-node`.
4. **crates.io** (#1) - **solo si** hay que hacer el primer publish de
   `vanta-memory` (ver `BIZ-19`); el publish normal va por OIDC.
5. **GitHub PAT** (#5, #6) - el mas critico: hasta que no se rota, cualquiera con
   el token puede escribir en el repo (tags, releases, workflows).
6. **Verificacion** (seccion 4) - un publish real por canal, no una lectura de
   owners.

> **No rotes #5 antes que #2-#4**: mientras el PAT viejo siga vivo, un atacante
> podria re-publicar o mover tags. Al reves, el riesgo es que el release se
> quede sin poder publicar.

## 3. Procedimiento por registry

### 3.1 crates.io (scope `publish`)

1. crates.io > **Account > Access Tokens > New Token**.
2. Scope: **`publish`** (marca "Publish new crates" **y** el alcance de crates
   existentes que necesites). Es el scope que faltaba en el token anterior:
   con el token viejo, `cargo publish` devolvia **403**.
3. **Opcional pero recomendado**: activa "Automatic crates.io publishing" (Trusted
   Publishing) para `vantadb` y para `vanta-memory` una vez exista; entonces el
   token solo hace falta para el primer publish y se puede revocar.
4. `gh secret set CARGO_REGISTRY_TOKEN --repo ness-e/Vantadb`
5. Verifica con un publish real, **no** con `cargo owner --list` (ese comando
   lee el indice estatico de owners y **no autentica**: dio un falso "PASS" con
   el token sin scope).

### 3.2 PyPI / TestPyPI

1. pypi.org > Manage > API tokens > **Create API token**; scope por proyecto
   (o "account-wide" si prefieres no repetirlo 8 veces).
2. **Ojo**: un token de PyPI **no** se puede re-validar sin cambiarlo. Cada
   proyecto necesita su propio token; si generas uno por proyecto son 8.
3. `gh secret set PYPI_TOKEN --repo ness-e/Vantadb` y
   `gh secret set TEST_PYPI_TOKEN --repo ness-e/Vantadb`
4. Verifica: el job `Publish to PyPI (Production)` de `release-adapters.yml`
   (verifica que la version no exista antes de subir: `twine --skip-existing`).

### 3.3 npm

1. npmjs.com > Access Tokens > **Generate New Token > Automation**.
2. Necesita scope de **publicacion** y de **crear el paquete** si es el primer
   publish de uno: un token de staging **no puede crear paquetes** (ese fue el
   fallo del release 0.9.0).
3. `gh secret set NPM_TOKEN --repo ness-e/Vantadb`
4. Verifica: `npm whoami` con el token (con un `.npmrc` o `NPM_TOKEN` en el
   entorno; en PowerShell **no** se puede meter un nombre de config de npm con
   "/" en `$env:`, usa `[Environment]::SetEnvironmentVariable`).

### 3.4 GitHub PAT

1. github.com > Settings > Developer settings > **Personal access tokens >
   Fine-grained** (preferible) o Tokens (classic).
2. Permisos minimos: **Contents: Read and write** (para tags y releases) y
   **Workflows: Read and write** solo si el workflow lo necesita.
3. Actualiza el secret que lo referencia (`RELEASE_PLZ_TOKEN` y/o el que use
   `release.yml`; comprueba con `grep -rn "RELEASE_PLZ_TOKEN" .github/`).
4. **Revoca el PAT viejo** (GitHub > Settings > Developer settings > Personal
   access tokens > Delete). Un PAT exposure se cierra revocando, no borrando el
   mensaje.
5. Si el PAT lo usaba una persona para otras cosas, emite uno nuevo para el
   repo y deja el personal para lo personal.

### 3.5 Tokens de providers

Mismo procedimiento que el PAT, pero en el secret que use
`release-providers.yml`. Revoca los antiguos.

## 4. Verificacion (por canal, con un publish real)

```powershell
# 1) los secrets existen y son recientes (no se ve el valor)
gh secret list --repo ness-e/Vantadb

# 2) publish real por canal (lo que de verdad valida un token)
#    crates.io: un publish de verdad; `cargo owner --list` NO autentica
#    PyPI:     twine upload --skip-existing de un adapter
#    npm:      npm whoami  +  npm publish --dry-run

# 3) el verificador oficial del release
pwsh scripts/verify-release.ps1 -Tag v0.9.0 -Smoke
```

**Un token no se da por bueno hasta que ha publicado algo.** Los tres fallos
del release 0.9.0 fueron tokens que parecian buenos: el de crates.io (403 al
publicar), el de npm staging (no podia crear el paquete) y el de PyPI (nunca se
llego a usar con el token de staging).

## 5. Cuando termines

1. Marca `BIZ-18` como completada en `docs/dev/Backlog.md` (la fila se elimina
   y el registro va a `docs/dev/avance/`).
2. Anota en el post-mortem (§7) la fecha de rotacion y que los 6 tokens viejos
   quedaron revocados.
3. Si `vanta-memory` ya se publico, cierra tambien `BIZ-19` y la fila del
   bootstrap en `docs/dev/Backlog.md`, y saca `vanta-memory` de
   `documented_only` en `release-manifest.json`.

## 6. Checklist de una linea por token

- [ ] TestPyPI: emitido, secreto actualizado, staging ok
- [ ] PyPI: emitidos (8 proyectos), secretos actualizados, publish real ok
- [ ] npm: token Automation con permiso de crear paquetes, secreto actualizado, `npm whoami` ok
- [ ] crates.io: token con scope `publish` emitido, secreto actualizado, publish real ok (solo si hay bootstrap)
- [ ] GitHub PAT: emitido con permisos minimos, secreto actualizado, **viejo revocado**
- [ ] providers: emitidos, secretos actualizados, **viejos revocados**
- [ ] `gh secret list` sin secretos vacios ni viejos
- [ ] `verify-release.ps1 -Smoke` en verde
- [ ] `BIZ-18` y `BIZ-19` actualizadas en el backlog
