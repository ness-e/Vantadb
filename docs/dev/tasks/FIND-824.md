---
title: "FIND-824 — download_sift.py salía 0 cuando había fallado"
kind: task
status: completed
description: "El script DEPRECATED atrapaba su excepción y salía 0: un fallo disfrazado de éxito. Se verificó huérfano (cero invocadores) y se dejó como redirección que falla determinísticamente (exit 1, sin tocar la red) nombrando el aprovisionador real, en vez de borrarlo: datasets/README.md y scripts/verify_datasets.ps1 todavía lo señalan como la acción a tomar."
id: FIND-824
tags: [vantadb, evals, benches, datasets, dev-tools]
---

# FIND-824 — `download_sift.py`: huérfano que se tragaba su excepción

## Metadata

- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-824`)
- **Origen:** lote evals/benches 2026-11-27 (`FIND-664` §Deuda) · **Ver:** `FIND-664`
- **Esfuerzo:** 🟢 15 min
- **Estado:** ✅ COMPLETED (local, sin push — Regla 7)

## Premisa (re-medida en HEAD)

Un fallo y un éxito eran indistinguibles por exit code. Medido con la descarga forzada a fallar
(se inyecta `urlretrieve` → `OSError`, para no pegarle a la red), versión de `HEAD` vs versión nueva:

```
# HEAD
python -c "…urllib.request.urlretrieve=lambda *a,**k: (_ for _ in ()).throw(OSError('host dead'))…" <git show HEAD:dev-tools/scripts/download_sift.py>
  Downloading SIFT1M (160MB)... This may take a while depending on the FTP server.
  Error downloading or extracting: host dead
  exit=0          ← la mentira

# worktree
python dev-tools/scripts/download_sift.py
  download_sift.py is DEPRECATED: its FTP host is dead.
  SIFT-1M is provisioned by: python scripts/download_ground_truth.py --datasets sift-1m
  exit=1
```

**Huérfano, verificado con `rg` en todo el repo** (sin `target/` ni `.git`): 8 hits, **cero
invocadores**. `datasets/README.md` (×2), `docs/dev/avance/activo/ci-cd.md`, `docs/dev/tasks/FIND-664.md`,
`docs/dev/tasks/closed/TBH-01.md` y `scripts/download_ground_truth.py:57` (un comentario) lo
**mencionan**; `scripts/verify_datasets.ps1:21` lo cita como `Source` (metadato que imprime
«run «source»» cuando falta un dataset), pero nada lo ejecuta.

## Decisión: arreglarlo para que falle, no borrarlo

El contrato permitía las dos. Se eligió **no borrar** porque el fichero tiene dos referencias vivas
en textos que el lector humano sí sigue (`datasets/README.md` da el comando literal;
`verify_datasets.ps1` lo imprime como acción recomendada): borrarlo deja una instrucción que apunta
a la nada, y **arreglar las dos referencias estaba fuera de la zona** de esta fila. Un script que
responde «esto está muerto, usá este comando» y **sale 1** convierte una referencia stale en un
desvío correcto.

**Qué se fue:** `urllib.request`, `tarfile`, `os`, la URL al FTP muerto, el `try/except` que solo
imprimía y el `tar.extractall` (que además extraía sin validar rutas). **Qué queda:** `main()` que
imprime el destino y devuelve **1**. No toca la red, así que el resultado es determinista y no
depende de que el host siga caído.

## Steps

1. ✅ Verificar orfandad con `rg` sobre todo el repo (arriba).
2. ✅ Reescribir el script como redirección que falla (§Decisión).
3. ✅ Probar las dos direcciones: `HEAD` → exit 0 con la descarga fallida; nuevo → exit 1 sin red.
4. ✅ `python -m py_compile` limpio (sin dependencias nuevas, stdlib solo).

## Contrato de verificación

```
python dev-tools/scripts/download_sift.py                 → exit 1 + nombra scripts/download_ground_truth.py
<versión de HEAD con urlretrieve lanzando>                → exit 0   (el mutante: reproduce el defecto)
python -m py_compile dev-tools/scripts/download_sift.py   → exit 0
```

**Invariante:** el script nunca sale 0 y nunca escribe en `datasets/`.

## Deuda declarada

- **`scripts/verify_datasets.ps1:21` sigue diciendo `Source = "dev-tools/scripts/download_sift.py"`**
  (y `datasets/README.md` lo documenta como «this directory's downloader»): ahora el lector llega a
  un desvío que le dice el comando correcto, pero el metadato debería apuntar a
  `scripts/download_ground_truth.py --datasets sift-1m`. Fuera de zona → **fila propuesta en el
  `RESULTADO`**.
- El espejo `.sh` ya quedó arreglado por `FIND-664`; el `.ps1` no, y es el que corre en Windows.

## Review

- **Ronda P2-01:** tier **Fast** (`dev-tools/**`) — verify mecánico + spot-check.
- **Scope:** solo `dev-tools/scripts/download_sift.py`; `scripts/**` y `datasets/README.md` intactos
  (deuda declarada arriba).
