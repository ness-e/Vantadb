---
title: "FIND-746 - -AutoSafe degrada a podar *.pdb en vez de negarse a liberar nada con un build en vuelo"
kind: task
status: completed
description: "El gate de disco era todo-o-nada: con cualquier cargo/rustc vivo se negaba a borrar también los *.pdb, que no participan de la cache incremental de rustc. Hoy el disco llegó a 0 bytes y 7,6 GB se liberaron a mano. Ahora degrada: poda solo *.pdb con antigüedad >= -MinAgeMinutes, no toca incremental/ y lo dice en el output"
tags: [vantadb, dev-tools, disk, cleanup]
---

# FIND-746 - `-AutoSafe` degrada a podar `*.pdb` en vez de negarse en bloque

## Contexto

Al cerrar `MCP-43` el disco llegó a **0 bytes** a mitad de tarea. El agente liberó
**7,6 GB** de `*.pdb` con antigüedad >30 min **a mano**, porque `-AutoSafe` se negó
correctamente a tocar `incremental/` (había builds de otro carril en vuelo) pero se
negó **también** a tocar los `*.pdb`, que no participan de esa cache.

El fallo de diseño no es "la guarda es demasiado estricta": es que es **todo-o-nada**.
`incremental/` podado a mitad de build deja fingerprints apuntando a sesiones que ya
no existen (rustc ICE / `could not read incremental cache`). Un `*.pdb` no entra en
esa cache; lo único que lo protege es que un linker pueda estar escribiéndolo **ahora**,
y eso se acota con antigüedad. El resultado real de la negativa en bloque era
"0 bytes libres y trabajo manual", peor que "podar lo que se puede probar seguro".

## Alcance

Escritura: `dev-tools/target-cleanup.ps1`, este task file y
`docs/dev/references/troubleshooting.md`. Ese doc **no** es el único que cita el
contrato: `docs/dev/avance/activo/operaciones.md:950` documenta los exit codes para
runner (`0 ok · 2 builds en vuelo · 3 sigue bajo umbral · 1 rechazo`) sin usar la
palabra `AutoSafe`, así que `rg 'AutoSafe' docs/` no lo encuentra. Está en
`docs/dev/avance/**`, **prohibido en este carril** → va como deuda declarada
(§Incógnitas), no se edita.

Prohibido (y respetado): `dev-tools/temp-collector.ps1` (`FIND-743`, otro carril),
`.github/**`, `docs/dev/operations/CI_POLICY.md`, `src/**`, `vanta-memory/**`,
`vantadb-mcp/**` (`FIND-748` en vuelo), `.githooks/**`, `docs/dev/Backlog.md`,
`docs/dev/plans/**`, `docs/dev/avance/**`, `.opencode/**`.

Carril C (PowerShell, sin `cargo`). Depende de `FIND-732` (el gate) y usa como
referencia de estilo el "doble prueba" de `FIND-743` (sin editarlo).

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `dev-tools/target-cleanup.ps1` (326 líneas antes del
  cambio: selección de artefactos, `Assert-UnderTarget`, conteo de builds y los tres
  `Remove-Item`), `docs/dev/tasks/FIND-743.md` (patrón de verificación),
  `docs/dev/references/troubleshooting.md:140-251` (§Disk full / build / `-AutoSafe`),
  `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md:268-287` (§9.b, de donde sale
  la fila). `temp-collector.ps1`: solo `:69` (comentario que cita este validador) —
  no se toca.
- **Referencias hacia dentro (quién lo llama):** **ninguna**. Verificado:
  `rg -n 'target-cleanup|AutoSafe' .githooks/ dev-tools/verify.ps1 dev-tools/verify_changed.ps1 justfile`
  → 0 hits. `verify.ps1` sólo invoca `temp-collector.ps1` (`FIND-743`), no este gate.
  Consecuencia directa: **cambiar un exit code no puede romper a `verify.ps1` ni al
  `pre-push`** (pre-mortem (d) del ticket, cerrado por evidencia, no por supuesto).
- **Referencias hacia fuera (lo que consume):** `target/**` (inventario propio),
  `Get-Process -Name cargo, rustc, clippy-driver, rustdoc` (detección existente,
  reusada), `Get-PSDrive` (disco del repo), `%TEMP%` (solo el sandbox del self-check).
- **Veredicto:** cambio contenido a un script sin consumidores automáticos. Firma de
  CLI aditiva (`-MinAgeMinutes` nuevo, default 30). El camino sin builds en vuelo
  queda **byte-equivalente** en comportamiento y códigos; lo único que cambia es la
  rama "hay builds en vuelo", que pasa de `exit 2` incondicional a degradar.

## Spec — contrato mecánico

| Decisión | Resuelto como | Evidencia |
|---|---|---|
| ¿Qué puede podar el modo degradado? | **Solo** grupos `Kind = 'pdb'`; el propio `Invoke-DegradedPrune` rechaza cualquier otro `Kind` con nota en el output (defensa en profundidad: el caller ya filtra) | pre-mortem (b): "degradar hasta borrar `incremental/` también" |
| ¿Qué prueba de abandono usa? | `LastWriteTime <= now - MinAgeMinutes`, default **30**, y `-MinAgeMinutes < 1` con `-AutoSafe` → `REFUSED` exit 1 | pre-mortem (a): un linker puede estar leyendo el `*.pdb` **ahora** |
| ¿Se relaja `Assert-UnderTarget`? | **No.** Sigue siendo la última puerta antes de cada `Remove-Item`, ahora con `try/catch` que **falla cerrado** (omite + reporta) en vez de tumbar el gate | el sandbox le pasa un `*.pdb` hermano de `target/` y el self-check asserta que sobrevive |
| Exit code si podó algo | `0` (aunque el disco siga bajo umbral) | contrato del ticket |
| Exit code si no podó nada y el disco sigue bajo umbral | `2` — **antes `2` significaba "hay builds en vuelo, no se tocó nada"**; ahora sólo se niega cuando no hay nada seguro que podar. Es el único cambio de contrato, y no tiene consumidores | `rg` de consumidores = 0 |
| Exit code si no podó nada pero el disco ya está sobre umbral | `0` (no hay nada que forzar) | coherencia con el `SKIP` de hoy |
| ¿`-DryRun` durante un build? | Sigue sin tocar nada, pero **avisa** que la corrida real degradaría — antes prometía los GB de `incremental/` que la real no iba a liberar | honestidad del reporte |
| Detección de builds | Se **reusa** la existente, extraída a `Get-InFlightBuilds` (misma lista de 4 procesos, 3 call sites) | el ticket prohíbe reescribirla |

## Steps

1. ✅ **RED/GREEN** — `Select-PdbItems` + `Invoke-DegradedPrune` + rama degradada en
   `-AutoSafe` + 21 asserts nuevos en `-SelfCheck` (sandbox en `%TEMP%` con `*.pdb`
   de 90 min, `*.pdb` de 1 min, `incremental/` **viejo**, un `*.pdb` fuera del
   prefijo y un junction; build simulado = `rustc.exe` real, copia de `cmd.exe`,
   para que la detección lo cuente igual que a un build de otro carril).
2. ✅ **VERIFY** — contrato mecánico + pruebas de mutación (7/7 detectadas).
3. ✅ **Docs** — `troubleshooting.md`: modo degradado, tabla de exit codes (incluido
   el segundo significado del `2`), self-check 16→40 asserts.
4. ✅ **Review P2-01** (`vanta-review`, tier **fast**): `changes-required` → F1
   (cobertura del cableado) y F2 (claim de impacto) corregidos en la misma ronda.
5. ⬜ **Commit local** (sin push): pathspec explícito.

## Verificación

```
pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -SelfCheck
SELFCHECK: PASS (40/40 asserts)      # 19 previos + 21 nuevos
```

```
pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -AutoSafe -DryRun
=== AutoSafe ===
BAJO UMBRAL: 10,63 GB libres < 15 GB.
  [dry-run] borraría target\debug\incremental (12,74 GB)
  [dry-run] borraría target\release\incremental (0 GB)
  [dry-run] borraría target\x86_64-unknown-linux-gnu\debug\incremental (1,77 GB)
  [dry-run] borraría **/*.pdb x307 (7,89 GB)
  [dry-run] liberaría 22,38 GB. Nada se borró.
  [dry-run] aviso: 5 cargo/rustc en vuelo - la corrida real DEGRADARÍA a *.pdb con antigüedad >= 30 min (incremental/ no se toca).
EXIT=0
```

(Salió por la rama bajo umbral de verdad: el disco estaba en 10,63 GB con builds de
otros carriles en vuelo — el escenario del incidente. **No** se corrió el `-AutoSafe`
real: habría podado 3,6 GB de símbolos que otros carriles están usando.)

Caso degradado real, del self-check (build simulado en vuelo + `*.pdb` viejo +
`*.pdb` reciente + `incremental/` viejo + señuelo fuera del prefijo + junction):

```
=== SelfCheck: modo degradado (build en vuelo -> solo *.pdb con antigüedad) ===
  PASS  la detección incluye el build simulado (id 17032 de 7 en vuelo)
  PASS  degradado: Select-PdbItems deja solo los grupos *.pdb (es lo que usa -AutoSafe)
DEGRADADO: 7 cargo/rustc en vuelo - incremental/ NO se toca (corrompe la cache de rustc).
DEGRADADO: solo se podan *.pdb con antigüedad >= 30 min (-MinAgeMinutes).
  borrando ...\find746-sc-2f7ed70c\target\debug\deps\old.pdb (0 MB)...
  omito ...\find746-sc-2f7ed70c\outside.pdb: REFUSED: '...\outside.pdb' no está bajo '...\target\' - AutoSafe no borra fuera de target/
DEGRADADO: podados 1 *.pdb (0 GB) con antigüedad >= 30 min.
DEGRADADO: liberados 0 GB (10,76 -> 10,76 GB libres).
  PASS  degradado: exit 0 cuando podó algo (aunque el disco siga bajo umbral)
  PASS  degradado: el *.pdb de 90 min se podó
  PASS  degradado: el *.pdb de 1 min se respetó (-MinAgeMinutes)
  PASS  degradado: incremental/ quedó intacto
  PASS  degradado: ni siquiera intentó borrar incremental/
  PASS  degradado: no sigue un junction (el destino puede estar fuera)
  PASS  degradado: el junction omitido se reporta
  PASS  degradado: no borra una ruta fuera del prefijo (Assert-UnderTarget sigue siendo la puerta)
  PASS  degradado: el rechazo de la ruta de fuera se reporta
  PASS  degradado: el output dice que está en modo degradado
  PASS  degradado: el output dice por qué no toca incremental/
DEGRADADO: 0 *.pdb podables (0 omitido(s) por antigüedad < 30 min, 0 no se pudo borrar) e incremental/ intacto; el disco sigue bajo umbral.
  PASS  degradado: la guarda interna tampoco deja tocar incremental/
  PASS  degradado: la guarda interna de Kind se anuncia en el output
  PASS  degradado: la guarda interna no llega a intentar el borrado
  PASS  degradado: -MinAgeMinutes 0 con -AutoSafe se rechaza (exit 1)
```

Segunda pasada (solo queda el `*.pdb` reciente, umbral forzado): `exit 2` y
`DEGRADADO: 0 *.pdb podables (1 omitido(s) por antigüedad < 30 min, 0 no se pudo
borrar) e incremental/ intacto; el disco sigue bajo umbral.`

### Cuánto libera el modo degradado en esta máquina (medido, 2026-11-25)

| Medición | Valor |
|---|---|
| `*.pdb` bajo `target/` | **307 ficheros / 7,89 GB** |
| `*.pdb` con antigüedad ≥ 30 min (lo podable ahora) | **132 ficheros / 3,60 GB** |
| `incremental/` (intacto en modo degradado) | 3 dirs · 12,74 + 0 + 1,77 GB |
| Total que el gate liberaría **sin** builds | 22,38 GB |
| Disco al medir | 10,36 GB libres (bajo el umbral, con 5 builds en vuelo) |

(La primera medición del día, con el `target/` menos crecido, daba 185 ficheros /
5,08 GB y **121 / 2,47 GB** podables. El incidente de `MCP-43` liberó **7,6 GB** a
mano; con este cambio esa poda es automática y `incremental/` queda intacto.)

### Pruebas de mutación (el self-check tiene que fallar si se rompe)

7/7 detectadas, contadas sobre 40 asserts:

| Mutación | Resultado |
|---|---|
| M1 quitar la guarda interna de `Kind` (tocaría `incremental/`) | `FAIL (2)` |
| M2 quitar el filtro de antigüedad (podaría el `*.pdb` de 1 min) | `FAIL (2)` |
| M3 devolver siempre `0` (nunca el `2` de "no pude podar nada") | `FAIL (1)` |
| M4 quitar `Assert-UnderTarget` del bucle degradado | `FAIL (2)` |
| M5 quitar el `REFUSED -MinAgeMinutes < 1` | `FAIL (1)` |
| M6 quitar la guarda de reparse points | `FAIL (2)` |
| M7 romper `Select-PdbItems` (el filtro que usa `-AutoSafe`) | `FAIL (7)` |

Las dos vaciedades que encontró el review, ya cerradas:

1. **El dir `incremental/` del sandbox se salvaba por edad, no por la guarda** (tenía
   `mtime` reciente). Corregido: es viejo (-120 min) y hay un assert negativo
   (`no intenta borrar incremental/`), no sólo el regex del log.
2. **El self-check llamaba `Invoke-DegradedPrune` directo, sin pasar por el cableado**
   del gate: cambiar `-eq 'pdb'` por otra cosa dejaba la feature muerta y el check en
   verde (M7 sobrevivía). Corregido: la selección vive en `Select-PdbItems` y el
   self-check llama **esa misma función**, con pdb + dir + señuelo mezclados.

### Gates mecánicos

| Comando | Resultado |
|---|---|
| `[Parser]::ParseFile dev-tools/target-cleanup.ps1` | **OK** (0 errores) |
| `node scripts/docs/check-docs.mjs` | exit 0, `GATING: all clear` (el task file figura como *orphan* hasta que se regenere `docs/index.md`) |
| `node scripts/docs/check-links.mjs` | exit 0 · 0 broken (1 wikilink presupuestado, preexistente) |
| `npx markdownlint-cli2 --config .github/markdownlint-cli2.yaml docs/dev/references/troubleshooting.md` | 0 issues. **Nota honesta:** la config **ignora** `docs/dev/tasks/**`, así que el task file no se lintea (`Linting: 1 file … 0 issues in 0 files`) |
| `node scripts/docs/gen-index.mjs --check` | **ya fallaba en HEAD** (`782 documents` commiteados vs 786 esperados; `ADR-0065` de otra lane no figura en el índice). El drift es ajeno y previo: mi task file es 1 de los 4 documentos nuevos. **No se corrió `--write`** para no pisar trabajo de otro carril |
| `dev-tools/verify.ps1` / `cargo` | **no se corren**: Carril C sin `cargo`, y el árbol tiene WIP ajeno (`nightly.yml`, `CI_POLICY.md`, `web/`, `.budget.json`) |

## Review

- **Enfoque:** P2-01 tier **fast** (`dev-tools/**`, `docs/dev/**`), agente distinto,
  contexto fresco (`vanta-review`). Corrió el script completo, 10 mutantes y el gate
  real sobre copias en `%TEMP%` con un `target/` propio y un `*.pdb` bloqueado por un
  handle.
- **Veredicto:** `changes-required` (2 Required, 0 Critical/High) → **corregido en la
  misma ronda**:
  - **F1 (Required)** — el self-check llamaba `Invoke-DegradedPrune` directo y nunca
    pasaba por el filtro del caller: M7 (romper `-eq 'pdb'`) dejaba la feature inerte
    con el check en verde. → `Select-PdbItems` compartida + caso que la ejercita.
  - **F2 (Required)** — el claim "`troubleshooting.md` es el único doc que cita el
    comportamiento del gate" era **falso**: `docs/dev/avance/activo/operaciones.md:950`
    documenta los exit codes sin usar la palabra `AutoSafe`. → claim corregido y deuda
    declarada (path prohibido en este carril, ver §Incógnitas).
  - **Optional aplicados:** F3 (assert negativo "no intenta borrar `incremental/`"),
    F4 (`-ErrorAction Stop` para que el `catch` de un fichero bloqueado no sea código
    muerto), F5 (contador `$failed` en el mensaje), F6 (segundo significado del `2` en
    la tabla de `troubleshooting.md`), F7 (delta negativo del disco), F8 (assert de
    `-MinAgeMinutes 0` vía script hijo + caso de junction para M6).
  - **Nits:** conteo de asserts corregido (21, no 11), tabla de mutaciones con los
    números reales, fila de markdownlint honesta (la config **ignora**
    `docs/dev/tasks/**`, así que el task file no se lintea), `-MinAgeMinutes 45`
    coherente con el texto, `Stop-Process` sólo si el PID sigue siendo `rustc`, y el
    sandbox falla como `FAIL` (no como excepción) en un disco a 0 bytes.

## Incógnitas / deuda

- **`docs/dev/avance/activo/operaciones.md:950` sigue diciendo `2 = builds en vuelo`**,
  que ya no es el contrato (ahora `2` = degradado que no pudo liberar nada). Path
  **prohibido** en este carril (`docs/dev/avance/**`) → lo aplica el `progreso` del
  orquestador o `vanta-lead`. Mientras no se toque, ese doc miente en una fila.
- **¿Otros artefactos podables en la misma categoría?** `target/<profile>/deps/*.d`
  (dep-info) es seguro igual que el `*.pdb`: lo escribe el compilador por target y no
  participa de la cache incremental. Medido hoy: pocos MB en cientos de ficheros —
  **no se amplía el scope** aquí; si el disco vuelve a apretar, es la siguiente
  candidata con la misma prueba de antigüedad.
- `target/<profile>/deps/*.rlib` **no** es podable con seguridad: forzaría recompilar
  los deps de `debug`, que es exactamente el coste que el gate existe para evitar.
- El gate sigue **sin invocador automático** (ni `verify.ps1` ni `pre-push`). Eso es
  de `FIND-192`, no de esta fila; se verificó expresamente para no romper nada.
- Un `REFUSED` de `Assert-UnderTarget` dentro del modo degradado ahora **omite y
  reporta** en vez de abortar el gate con exit 1 (antes, con `$ErrorActionPreference
  = 'Stop'`, una ruta escapada mataba el proceso). La propiedad de seguridad no
  cambia —nada fuera de `target/` se borra— pero el diagnóstico es una nota, no un
  fallo del gate. Deliberado: en modo degradado se prefiere podar el resto.
- Ventana TOCTOU irreducible: un linker que abra el `*.pdb` **entre** el `Get-Item`
  y el `Remove-Item`. La acota la antigüedad (30 min por defecto) y, si el handle
  impide borrar, el fichero se cuenta como **no liberado** y se sigue (nunca al revés).
