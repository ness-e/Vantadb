---
title: "FIND-743 - el recolector de temporales también barre ficheros sueltos"
kind: task
status: completed
description: "temp-collector.ps1 iteraba sólo directorios bajo %TEMP%, así que el residuo suelto de los tests (vanta_test_*.corrupt de 54/63 B, vanta-ckpt-*.jsonl) era invisible para su inventario y su reporte decía 'limpio' con la misma cara que un inventario completo"
tags: [vantadb, dev-tools, temp, cleanup]
---

# FIND-743 - el recolector de temporales también barre ficheros sueltos

## Contexto

`FIND-735` (`c73519024`) midió el residuo que deja la cuarentena de WAL:
**63 B por corrida** de `<wal>.corrupt` y **54 B por corrida** de
`vanta_test_sharded_<rand>.shard0.corrupt` (`src/wal_sharded.rs:999`). El test
borra su directorio, pero la cuarentena escribe un **hermano** del path y el
`remove_file` del final no lo toca.

El recolector de `FIND-732` (`264940d57`) itera **directorios** bajo `%TEMP%`; un
fichero suelto nunca entra en su inventario ni en su reporte. Volumen pequeño por
corrida pero **sin techo**, y el reporte es indistinguible de un inventario
completo: "limpio" significaba a la vez "no hay nada" y "no miré la mitad".

## Alcance

Escritura permitida: `dev-tools/temp-collector.ps1` + este task file.
`dev-tools/verify.ps1` y `.githooks/pre-push` **sólo se leyeron** (ambos invocan
con `-Quiet`; el cambio es aditivo y no necesita flag nuevo).
Prohibido: `src/**` (`FIND-741` en vuelo), `vanta-memory/**`, `vantadb-mcp/**`
(`MCP-43` en vuelo), `.github/**`, `.githooks/**`, `docs/dev/Backlog.md`,
`docs/dev/plans/**`, `docs/dev/avance/**`, `.opencode/**`.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `dev-tools/temp-collector.ps1` (264 líneas),
  `dev-tools/verify.ps1:87` (call site), `.githooks/pre-push:69-72` (call site),
  `dev-tools/target-cleanup.ps1` (patrón de guardas), `docs/dev/tasks/FIND-732.md`,
  `docs/dev/tasks/FIND-735.md`, `docs/dev/avance/activo/operaciones.md:955-964`.
- **Referencias hacia dentro (quién lo llama):** `dev-tools/verify.ps1` y
  `.githooks/pre-push`, ambos `-Quiet` y **advisory** (nunca bloquean). Ninguno
  depende de la forma del reporte ni del número de items.
- **Referencias hacia fuera (lo que consume):** `%TEMP%` top-level; nombres que
  produce `src/wal.rs:1743` (`<path>.corrupt`), `src/wal_sharded.rs:999`
  (`vanta_test_sharded_<rand>.shard0.corrupt`) y
  `vantadb-node/tests/cognitive-contract.test.ts:187` (`vanta-ckpt-<pid>.jsonl`).
- **Veredicto:** cambio aditivo y contenido. Firma de CLI intacta, códigos de
  salida intactos, `-Quiet`/`-DryRun`/`-SelfCheck` intactos. Nada aguas arriba se
  entera; lo único que cambia es que el inventario ahora tiene una segunda clase.

## Steps

1. ✅ **RED** - casos del self-check para ficheros sueltos antes de implementarlos
   (falló por dos motivos distintos: `FAIL` del inventario + `Invoke-Collect` inexistente).
2. ✅ **GREEN** - inventario + reporte separado + camino de borrado único.
3. ✅ **VERIFY** - contrato mecánico (3 comandos) + señuelos antes/después + pruebas de mutación.
4. ✅ **Review P2-01** (`vanta-review`, tier fast: `dev-tools/**`) - `changes-required` → F1/F2 corregidos en la misma ronda.
5. ✅ **Commit local** (sin push): `fix(tools): the temp collector also sweeps loose files, with the same two abandonment proofs (FIND-743)`.

## Investigación: ¿la sonda de exclusividad sirve sobre un fichero?

Un fichero no tiene `lock` de fjall; su señal de vida es el handle abierto. Medido
empíricamente (Windows 11 26200, PS 7.6.10):

| Handle que lo tiene abierto | Abrir con `FileShare.None` |
|---|---|
| ninguno | **OPEN** |
| `FileShare.None` | **BLOCKED** |
| `FileShare.Read` (Access=Read) | **BLOCKED** |
| `FileShare.ReadWrite` | **BLOCKED** |
| `FileShare.ReadWrite+Delete` (lo que da `std::fs::File`) | **BLOCKED** |

Conclusión: la sonda **sí** sirve para ficheros, y el bloqueo es **cross-process**
(verificado: el barrido corre como proceso hijo, con el handle del padre vivo, y el
fichero no aparece en el inventario). Se reusa `Test-LockFree` tal cual.

Segundo hallazgo, y el que justifica que la sonda sea obligatoria:
**`Remove-Item` SÍ borra un fichero abierto cuando el handle concede `DELETE`**
(medido: `DELETED-ANYWAY`), que es justo lo que concede `std::fs::File`. La sonda
es la que protege, no `Remove-Item` — igual que en `FIND-732`.

Techo conocido (`ponytail:` en el código): un handle abierto con **acceso cero**
(sólo atributos) no colisiona con `FileShare.None`; para ese caso el segundo cierre
es la inactividad de N minutos. Ningún productor del repo lo hace.

## Verificación

```
pwsh -NoProfile -File dev-tools/temp-collector.ps1 -SelfCheck
SELFCHECK: PASS (36/36 asserts)      # 19 previos + 17 nuevos
```

```
pwsh -NoProfile -File dev-tools/temp-collector.ps1 -DryRun
%TEMP% = C:\Users\Eros\AppData\Local\Temp\
dirs : 0 carpeta(s), 0 B
files: 0 fichero(s), 0 B
```

```
pwsh -NoProfile -File dev-tools/temp-collector.ps1 -MinAgeMinutes 0
%TEMP% = C:\Users\Eros\AppData\Local\Temp\
dirs : 0 carpeta(s), 0 B
files: 0 fichero(s), 0 B
```

Los dos últimos son el estado **después** del barrido: las dos clases se imprimen
siempre, así que "0 + 0" ahora es una afirmación sobre todo el inventario y no un
resumen ciego.

### Primer barrido real (antes / después)

`-DryRun` antes de tocar nada: **15 ficheros sueltos, 2,0 KB** — 4
`vanta_test_wal_healing_*.corrupt` (63 B), 8 `vanta_test_sharded_*.shard0.corrupt`
(54 B), 2 `vanta-ckpt-5296.jsonl*` (509 + 869 B) — más 1 `.tmp*` de 4,02 MB:

```
dirs : 1 carpeta(s), 4,02 MB
files: 15 fichero(s), 2,0 KB
-DryRun: liberaría 4,02 MB en 1 carpeta(s) + 15 fichero(s). Nada se borró.
```

Barrido: `recolector: 1/1 carpeta(s) + 15/15 fichero(s), 4,02 MB liberados (0 respetado(s) por duda, 0 fallo(s))`.
Ninguno de esos 15 ficheros era visible para el recolector anterior.

### Señuelos (7 casos, control positivo incluido)

| Señuelo | Esperado | Resultado |
|---|---|---|
| `vanta_test_recent_*.corrupt` (reciente, cerrado) | intacto | intacto |
| `vanta_test_held_*.corrupt` (viejo, **handle con DELETE**) | intacto | intacto |
| `Report_*.txt` (viejo, nombre ajeno) | intacto | intacto |
| `vanta_test_old_*.corrupt` (viejo, cerrado) | **borrado** | borrado |
| `vanta_test_hidden_*.corrupt` (viejo, **atributo Hidden**) | **borrado** | borrado |
| `vanta-ckpt-<pid vivo>.jsonl` (viejo, PID vivo) | intacto | intacto |
| `vanta-ckpt-<pid muerto>.jsonl` (viejo, PID muerto) | **borrado** | borrado |

`SENUELOS: PASS`. Residuo con prefijo en `%TEMP%` al terminar: **0**.

### Pruebas de mutación (el self-check tiene que fallar si se rompe)

| Mutación | Resultado |
|---|---|
| Quitar la **guarda de borrado** (`Invoke-Collect`) | `FAIL (32/36)`, exit 1 — cae en dir **y** en fichero |
| Quitar la **sonda de lock** del inventario | `FAIL (35/36)`, exit 1 |
| Quitar la **guarda de inactividad** de ficheros | `FAIL (35/36)`, exit 1 |

La primera versión de estos asserts pasaba **sin la guarda** (el handle del
self-check no concedía `DELETE`, así que `Remove-Item` fallaba solo y el assert se
satisfacía por el motivo equivocado). Corregido: los handles del self-check
conceden `DELETE` **y** hay un control positivo que comprueba que, con ese handle,
un `Remove-Item` directo **sí** borra. Si algún día deja de borrar, el control lo
delata en vez de dejar el assert vacuo.

### Gates mecánicos

| Comando | Resultado |
|---|---|
| `[Parser]::ParseFile dev-tools/temp-collector.ps1` | **OK** (0 errores) |
| `node scripts/docs/check-docs.mjs` | exit 0, `GATING: all clear` (el task file figura como *orphan* hasta que se regenere `docs/index.md`) |
| `node scripts/docs/check-links.mjs` | sin enlaces rotos nuevos (1 wikilink preexistente en `avance/historial/`, presupuestado) |
| `node scripts/docs/gen-index.mjs --check` | **ya fallaba en HEAD** por drift ajeno (`HARD-01` vs `FIND-246` en `docs/index.md`, modificado por otra tarea en vuelo); no se corrió `--write` para no pisar ese trabajo |

`dev-tools/verify.ps1` **no se corrió**: el working tree tiene cambios Rust ajenos
en vuelo (`vanta-memory/**` de `FIND-741`, `vantadb-mcp/**` de `MCP-43`) y este
ticket es Carril C (sin `cargo`). Sus dos únicos call sites (`verify.ps1:87`,
`pre-push:70`) invocan con `-Quiet`; verificado que `-Quiet` sigue saliendo 0 y
sólo imprime línea cuando borra algo o cuando respeta algo por duda.

## Review

- **Enfoque:** P2-01 tier **fast** (`dev-tools/**`), agente distinto (`vanta-review`), contexto fresco.
- **Cómo se probó:** el revisor reprodujo `-SelfCheck`, `Parser::ParseFile`, y
  corrió **mutaciones** sobre una copia del script (quitar la guarda → el
  self-check seguía en 29/29), más sondas propias de `FileShare`/`Remove-Item`.
- **Veredicto:** `changes-required` en la primera ronda → **F1** (High: el assert
  del fichero abierto pasaba por el motivo equivocado, la guarda se podía borrar
  sin que nada fallara), **F2** (Medium: los dirs se borraban por un camino
  duplicado, la guarda nueva para dirs sin cobertura), **F3** (Medium: task file
  marcado `completed` antes del gate). Corregidos en la misma ronda: handles con
  `DELETE` + control positivo, dirs por `Invoke-Collect` + caso de reapertura,
  pruebas de mutación verdes (32-35/36 al mutar), `-Force` en la enumeración del
  root (F4), `Test-PidAlive` para `vanta-ckpt-<pid>` (F5), dudas separadas de
  fallos (F6) y assert de `Assert-UnderTemp` end-to-end en vez de tautológico (F7).

## Incógnitas / deuda

- `docs/dev/references/troubleshooting.md:279` cita `SELFCHECK: PASS (19/19
  asserts)`; ahora son 36. Fuera del alcance de escritura de este ticket —
  actualizar cuando se toque ese doc.
- El prefijo `vanta*` es el documentado por el repo, pero `vanta-ckpt-<pid>.jsonl`
  (residuo real de `vantadb-node/tests/cognitive-contract.test.ts:187`) sólo se
  distingue de un fichero de usuario por la doble prueba (handle + inactividad) y,
  si lleva PID, por `Test-PidAlive`. Decisión: **ante la duda, no borrar y
  reportar** — lo que la guarda rechaza se imprime como "respetado por duda",
  también con `-Quiet`, y separado de un fallo real de borrado.
- `FIND-744` (`src/wal_sharded.rs:999`, el productor del `.shard0.corrupt`) no se
  tocó: este ticket hace que el residuo **se vea y se recoja**, no que deje de
  producirse.
- Ventana residual TOCTOU probe→delete: un proceso que abra el fichero **entre**
  la re-sonda y el `Remove-Item` no está protegido. Es irreducible sin mantener el
  handle durante el borrado, exige 10 min de inactividad previos y dura
  microsegundos; aceptado.
