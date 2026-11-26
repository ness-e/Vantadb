---
title: "FIND-192: Higiene de entorno en waves co-batch — `-AutoSafe` en `target-cleanup.ps1` + umbral de disco declarado"
kind: task
status: completed
description: "Non-interactive pre-wave disk gate: below a declared 15 GB free threshold it frees only regenerable artifacts under target/ (incremental/ + *.pdb), with -DryRun, -SelfCheck path asserts, a cargo/rustc in-flight guard and GB-freed reporting"
tags: [dev-tools, disk, windows, build-gate]
---

# FIND-192: Higiene de entorno en waves co-batch — `-AutoSafe` + umbral de disco declarado

## Metadata
- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) §5, Fase 1, wave 1.2
- **Fuente:** Backlog `FIND-192` (origen: VER-05/VER-07 env findings; script creado 2026-10-01, nunca integrado a una wave)
- **Esfuerzo:** 🟡 4h
- **Prioridad:** 🟡 Media
- **Tipo:** Dev-tools (PowerShell) + Docs
- **Turns estimados:** 5-10
- **Creado:** 2026-11-24T00:00
- **last-synced:** 2026-11-24T00:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Un humano/orquestador antes de la primera build de una wave (`pipeline-run`, `vanta-lead`), y el propio documento de troubleshooting. **No hay integración automática**: `dev-tools/verify.ps1` no se toca (prohibido por la tarea, ver §Deuda D1) |
| Callees | `Get-PSDrive` / `Get-ChildItem` / `Remove-Item` / `Get-Process` (todo stdlib de PowerShell), `target/` como único árbol escribible. Nada en `src/**`, `.github/**` ni `Formula/**` |
| Implicaciones | Un script que borra archivos: el riesgo es borrar fuera de `target/` o corromper la caché incremental a mitad de build. Se mitiga con (a) validación de cada ruta contra el prefijo `target\` **con separador** antes de `Remove-Item`, (b) umbral explícito, (c) guarda `cargo`/`rustc` en vuelo, (d) `-DryRun` y `-SelfCheck` como caminos sin efecto. No cambia ningún contrato público ni comportamiento del motor |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `dev-tools/target-cleanup.ps1` (80L, el script de 2026-10-01), `docs/dev/references/troubleshooting.md` (135L, salida de `FIND-177`), `Cargo.toml` §`[profile.release|bench|dev|ci|test]` (`734-760`, `856-868`), `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` §5 + ownership `126,134,162,261`, `.opencode/task-system/prompts/task.md` (`170-398`), `.opencode/task-system/prompts/pipeline-full.md` (317L), `docs/dev/tasks/FIND-177.md` (186L, formato hermano de la misma wave)
- **Archivos referenciados hacia dentro:** `rg -n "target-cleanup"` → Backlog fila `FIND-192`, plan §5, este task file. `.opencode/AGENTS.md` §Reference Files apunta a `docs/dev/references/troubleshooting.md` (ruta ya existente desde `FIND-177`). Ningún script invoca `target-cleanup.ps1` todavía — ese es justamente el hueco que cierra la tarea (invocación documentada + modo no interactivo)
- **Archivos que referencian a los editados:** `docs/dev/references/troubleshooting.md` ← `.opencode/AGENTS.md` (Reference Files), plan `160`, Backlog `194`; `dev-tools/target-cleanup.ps1` ← solo menciones en prosa (Backlog + plan). Ningún YAML de CI lo consume
- **Veredicto impacto:** **alto si el script se equivoca de ruta** (borra árbol de trabajo o repos ajenos) — mitigado por `Assert-UnderTarget` y `-SelfCheck`; **bajo** para el resto del repo (0 archivos de código, 0 workflows, 0 contratos). El script es *aditivo*: los flags viejos (`-Clean`, `-Full`, `-Yes`) conservan su comportamiento y son los únicos que aún preguntan/bajan a `debug/deps`

## Contrato

```
pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -DryRun            # exit 0, reporta GB liberables
pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -AutoSafe -DryRun  # exit 0, mismo reporte, sin tocar nada
```
+ `-AutoSafe` borra **solo** `{*}/incremental` y `*.pdb` bajo `target/`, solo si el disco libre < `-MinFreeGB` (default 15), nunca `target/` entero ni nada fuera de `target/`, y reporta los GB liberados.
+ `-SelfCheck` que demuestra por assert que la lista de objetivos no puede salir de `target/`.

**Verificación obtenida (2026-11-24, evidencia real de esta sesión):**
- `-DryRun` → **exit 0** · `C libre: 6.37 GB` · `debug 35.07 GB` · `GB liberables: 21.17`
- `-AutoSafe -DryRun` → **exit 0** · `BAJO UMBRAL: 5.1 GB libres < 15 GB` · `[dry-run] liberaría 22.05 GB. Nada se borró.`
- `-AutoSafe -DryRun -MinFreeGB 100` (fuerza la rama bajo umbral con el disco por encima del default) → **exit 0** · lista los grupos y `Nada se borró`
- `-SelfCheck` → **exit 0** · `SELFCHECK: PASS (17/17 asserts)` · grupos y rutas reales validadas contra el validador de borrado
- `-AutoSafe -Clean` y `-AutoSafe -Full` → **exit 1** (`REFUSED: -AutoSafe nunca borra de forma profunda`)
- Nada se borró por este script: los `*.pdb` **subieron** de 432 a 486 durante los dry-runs (las builds paralelas seguían produciendo), prueba de ausencia de efecto
- Re-medición al cierre (el árbol cambió por otras lanes, ver §Notas): `C libre: 15.4 GB` · `GB liberables: 12.94` · `-DryRun` y `-AutoSafe -DryRun` siguen en **exit 0**

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | ¿Qué borra `-AutoSafe`? | (A) `{*}/incremental` + `*.pdb` — todo regenerable por la siguiente build, cubre el 100 % del incidente (22 GB de 39 GB medidos) · (B) sumar `debug/deps` — libera más pero fuerza rebuild en frío (30-40 min) y mata el propósito de un gate pre-wave · (C) `target/` entero — destructivo, prohibido por la tarea | A | ✅ decidido por evidencia: con (A) el wave recupera 22 GB y conserva los artefactos compilados; (B) queda como `-Clean -Full` interactivo |
| 2 | Umbral de disco | (A) 15 GB fijos por default, parametrizables con `-MinFreeGB` · (B) 10 GB · (C) 25 GB · (D) ratio del disco | A | ✅ decidido por evidencia: hoy `target/` = 39 GB y el disco tenía 5.1 GB libres con 8 builds en vuelo; 15 GB cubre un `debug/deps` incrementales de una wave sin ser tan alto que dispare borrados innecesarios. Es el "knob de calibración": `-MinFreeGB` |
| 3 | ¿Guarda contra builds en vuelo? | (A) omitir y salir **2** (advisory, no fatal) · (B) borrar igual (riesgo: `rustc` con la caché incremental a medio escribir → ICE) · (C) matar `cargo`/`rustc` — prohibido: hay otras tareas del wave compilando | A | ✅ decidido por evidencia: `FIND-259`/`FIND-710` compilaban en paralelo (6 `cargo` + 2 `rustc` en `Get-Process` durante esta tarea); exit 2 permite al orquestador reintentar sin romper nada |
| 4 | ¿Integrar al gate (`verify.ps1`)? | (A) no tocarlo — **prohibido** por la tarea (ownership: `FIND-177` dejó ahí el probe advisory, sin aplicar) · (B) invocarlo desde un runner de wave que aún no existe · (C) documentar umbral + comando y dejar el modo no interactivo listo | C | ✅ decidido por evidencia: no hay runner de waves versionado (`.github/**` y `dev-tools/verify.ps1` fuera de scope); el contrato del plan pide "invocado por un gate pre-wave con umbral declarado" y eso se satisface con el modo + el umbral escrito en troubleshooting (§Deuda D1/D2) |
| 5 | ¿Borrar `incremental/` rompe `--incremental`? | (A) documentarlo y bloquear el borrado con builds vivos · (B) borrar también `.fingerprint` para no dejar inconsistencias · (C) no tocar `incremental/` | A | ✅ decidido por evidencia: (B) fuerza recompilar **todos** los deps de `debug` (justo el costo que el gate evita); (A) conserva el beneficio y documenta el remedio puntual (`cargo clean -p <crate>`, `CARGO_INCREMENTAL=0`) en troubleshooting |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. `-AutoSafe` **nunca** borra fuera de `target/`: toda ruta pasa por `Assert-UnderTarget` antes de `Remove-Item`, comparando contra el prefijo absoluto **que termina en separador** (rechaza `target2/`, `target/../`, `target/..` y `target/` mismo).
  2. `-AutoSafe` nunca borra `deps/`, `debug/`, `release/` ni `target/` completo; el borrado profundo solo existe en `-Clean -Full` interactivo.
  3. `-AutoSafe` nunca mata procesos. Con `cargo`/`rustc` vivos sale **2** sin tocar nada.
  4. `-DryRun` no tiene efectos en **ningún** modo (ni siquiera con `-Clean`, que se omite avisando).
  5. Los flags previos (`-Clean`, `-Full`, `-Yes`) conservan su semántica: son la única vía de limpieza profunda y siguen pidiendo confirmación.
  6. `dev-tools/verify.ps1`, `src/**`, `.github/**`, `Formula/**`, `docs/dev/Backlog.md`, `docs/dev/plans/**`, `docs/dev/avance/**`, `scripts/docs/**` no se tocan en esta tanda.
- **Comandos de verificación:** los dos `-DryRun` del contrato (exit 0) + `-SelfCheck` (exit 0, 16/16) + `-AutoSafe -Clean` (exit 1) + `node scripts/docs/check-docs.mjs` (all clear) + `node scripts/docs/check-links.mjs` (0 broken) + `npx markdownlint-cli2 --config .github/markdownlint-cli2.yaml "docs/**/*.md"` (0 issues).
- **Deuda pendiente:** §Deuda D1-D3.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** **negativo** — no se agrega código nuevo de producción; se cierra deuda operativa (el disco llegó a 0 bytes y tumbó un link en medio del pipeline, 31 GB liberados a mano) y se hace auditable lo que hasta hoy era un script interactivo de uso tribal.

### §Deuda — propuestas que NO se aplican acá (ownership ajeno)

**D1 — probe advisory de `FIND-177` en `dev-tools/verify.ps1` (sigue sin aplicar).**
Es el bloque `WIN-LOCK` de `docs/dev/tasks/FIND-177.md:77-91` (aviso, no bloqueo, cuando un proceso corriendo desde `target/debug` mantiene el `.exe` mapeado). **Prohibido por la tarea**: `verify.ps1` no pertenece a este carril. Sigue pendiente en el Backlog de `FIND-177`; este doc solo lo referencia.

**D2 — nadie invoca todavía el gate (`-AutoSafe`) automáticamente.**
No existe un runner de waves versionado: `pipeline-run`/`vanta-lead` orquestan desde el harness (`.opencode/`, fuera del repo) y los workflows de `.github/**` no son de este carril. Hoy la integración es: umbral declarado (15 GB) + comando documentado en `docs/dev/references/troubleshooting.md` + exit codes estables (`0/1/2`) para que un runner lo llame sin parsear texto. Cuando exista un pre-wave script, esa es la línea a insertar:

```powershell
pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -AutoSafe   # 2 = builds en vuelo; 3 = sigue bajo umbral
```

Contrato de exit codes para un runner, sin parsear texto: `0` ok · `2` reintentar entre builds · `3` **no arrancar la wave** (disco sigue bajo el umbral: liberar a mano) · `1` rechazo de seguridad.

**D3 — el borrado real no se ejecutó desde este carril** (solo dry-runs). Había 6 `cargo` + 2 `rustc` compilando (`FIND-259`, `FIND-710`) y 5 GB libres: correrlo habría roto la caché incremental de otra tarea. El camino de borrado queda verificado por (a) `-SelfCheck` sobre la lista real de rutas, (b) el `Assert-UnderTarget` repetido en el bucle de borrado, (c) la guarda de procesos (muestreada al inicio **y** re-chequeada por ítem), (d) el salto de reparse points — pero **no** por una ejecución en vivo. Primera oportunidad real: entre dos waves.

*Nota de honestidad:* durante esta tarea el árbol **sí cambió por fuera** — los 486 `*.pdb` pasaron a 3 y el disco subió de 4.8 GB a 15.4 GB libres, con los artefactos de build intactos (505 `.exe`, 951 `.rlib` en `debug/deps`). No fue este script (esta sesión solo corrió dry-runs; un `-AutoSafe` real habría salido `2` por los 6 `cargo` vivos). El efecto observable es el esperado por el diseño: podar `*.pdb` + `incremental` cuesta 0 artefactos. Queda como evidencia indirecta, no como prueba de ejecución.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato mecánico ✅ (2 dry-runs exit 0 + `-SelfCheck` 16/16 + rechazo de flags) · `check-docs` all clear ✅ · `check-links` 0 broken ✅ · markdownlint 0 issues ✅ |
| **Commit** | Atómico (3 archivos), `chore(dev-tools): add a safe non-interactive target cleanup for waves (FIND-192)` (hash en el `RESULTADO`/`git log -1` — citarlo acá se auto-invalida al amend), sin `--no-verify`, sin `push` (Regla 7) ✅ |
| **Release** | N/A: no toca código del motor ni packaging; `chore:` no genera release en release-plz (justificado en Notas) |

## Herramientas necesarias
- `pwsh -NoProfile -File`, `Get-PSDrive`, `Get-ChildItem`, `Get-Process`
- `node scripts/docs/*.mjs`, `npx markdownlint-cli2`
- **No** `cargo`: el carril Rust está ocupado en esta wave (instrucción de la tarea)

**Skills cargadas (SDP):** `documentation-skill` (obligatoria: crea/edita `.md` bajo `docs/` — frontmatter, `kind`→ruta, links, DoD) · `incremental-implementation` (slice delgado: primero el gate no destructivo, después el doc) · `test-driven-development` (el self-check *es* el test: se escribió para fallar si una ruta escapa de `target/`) · `writing-guidelines` · `doubt-driven-development` (código que borra archivos → revisión adversarial del bucle de borrado) · `systematic-debugging` (causa raíz de los GB, no solo "borrar y listo"). SDP v3 devolvió además `campaign-executor`, `progreso`, `writing-plans`, `context-engineering`, `source-driven-development` — base/ciclo, sin `pinned` (`SDP: documentation-skill, incremental-implementation, test-driven-development, writing-guidelines, doubt-driven-development, systematic-debugging`).

## Investigation Notes

- **Por qué `incremental` crece a 22 GB.** Cargo crea un directorio de sesión por **crate × perfil** y no hace GC de las sesiones viejas mientras los builds siguen teniendo éxito: cada sesión guarda objetos intermedios, `.work-products` y el caché de queries. Con ~90 crates del workspace × perfiles (`dev`, `test`, `bench`, `ci`) × las variantes de features que piden `vantadb-python`/`vantadb-server`/`vantadb-mcp`, el árbol se multiplica. Medido hoy: **12.07 GB** en `target/debug/incremental` (y 0 GB en `release`, que se compila sin incremental). En el incidente del 23/11 llegó a ~22 GB.
- **Por qué los PDB llegan a 9-10 GB — y no es un descuido que se arregle en `Cargo.toml`.** `Cargo.toml:856-857` (`[profile.test] debug = 1`) y `:743` (`[profile.bench] debug = 1`) están puestos **a propósito** (backtraces útiles en tests, flamegraphs en benches). Pero eso significa que **cada binario de test/bench/example enlazado emite su `.pdb`** con símbolos completos: hoy son **445 archivos / 9.98 GB** en `debug/deps`, `debug/examples` y `release/deps`. Bajarlos a `debug = 0` ahorraría disco y empeoraría el diagnóstico → se podan entre waves (esto es exactamente lo que hace `-AutoSafe`), no se apagan.
- **Por qué `--incremental` quedó roto tras un cleanup previo (nota de la fila del Backlog).** Borrar `incremental/` **con un build en curso** deja entradas de `.fingerprint` apuntando a directorios de sesión que ya no existen; el siguiente `rustc` que reusa una de esas entradas muere con `could not read incremental cache` (o un ICE), y el síntoma se lee como repo corrupto. No es que la caché sea frágil por diseño: es que se borró a mitad de escritura. Remedios documentados: `cargo clean -p <crate>` (una sola crate) o `$env:CARGO_INCREMENTAL = 0` para un gate puntual. **No** se borra `.fingerprint` en `-AutoSafe`: forzaría recompilar todos los deps de `debug`, que es el costo que el gate existe para evitar.
- **Estado del entorno durante esta tarea:** C: con **5.1-6.4 GB libres** (o sea, ya por debajo del umbral de 15 GB), `target/debug` 35-36 GB, y **6 `cargo` + 2 `rustc`** compilando (`FIND-259`, `FIND-710`). Por eso: solo dry-runs, y la guarda de procesos es un requisito real, no teórica.
- **Sin web research:** todo el comportamiento es del SO y de Cargo, verificable en el repo (`Cargo.toml`) y con comandos locales; no aplica el gate de citas ni hay URLs que verificar.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — resueltas: por qué crece `incremental` (sesiones por crate×perfil), por qué crecen los PDB (`profile.test/bench debug = 1`), por qué se rompe incremental (borrado a mitad de build), qué borra exactamente el gate |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — **aplica y es el corazón de la tarea.** El script borra archivos: el threat model es "una ruta calculada se escapa de `target/`". Controles: prefijo absoluto con separador final + `[IO.Path]::GetFullPath` (normaliza `..` antes de comparar), validación repetida en el bucle de borrado, lista de objetivos cerrada (`incremental/` y `*.pdb`), `-SelfCheck` con 6 casos de rechazo (incluye el clásico `target2/` y el escape `target/../src/`), y rechazo de flags incompatibles. No se agrega ninguna dependencia. Sin input de usuario en el camino `-AutoSafe`.
- [x] **PERFORMANCE** — aplica en la dirección inversa: el gate **protege** el throughput de las waves (una wave detenida por disco lleno cuesta horas; un `cargo` en frío por borrar `deps/` cuesta ~40 min). No cambia ningún hot path del motor; no hay baseline que medir con `canonical_p99`.

## Steps

### Step 1: Mapear el script existente y su contrato implícito
- **Archivos:** `dev-tools/target-cleanup.ps1`
- **Acción:** leer el script completo (80L), identificar qué conserva (`-Clean`, `-Full`, `-Yes`, reporte, lista de procesos) y dónde insertar el modo no interactivo sin cambiar la semántica previa
- **Verify:** lectura completa + `git log --oneline -3 -- dev-tools/target-cleanup.ps1` (origen: `bb0bc04a6`)
- **Estado:** ✅ DONE

### Step 2: `-AutoSafe` + `-DryRun` + `-SelfCheck` con validación de rutas
- **Archivos:** `dev-tools/target-cleanup.ps1`
- **Acción:** `Get-TargetPrefix`/`Assert-UnderTarget` (prefijo con separador + `GetFullPath`), `Get-SafeCleanupItems` (lista cerrada y validada), umbral `-MinFreeGB 15`, guarda `cargo`/`rustc` → exit 2, reporte de GB liberados por delta de `Get-PSDrive`, `-DryRun` sin efectos en todos los modos, rechazo `-AutoSafe` + `-Clean`/`-Full`
- **Verify:** `pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -SelfCheck` → exit 0, 16/16
- **Estado:** ✅ DONE

### Step 3: Documentar umbral + comando en troubleshooting (§ entorno/disco)
- **Archivos:** `docs/dev/references/troubleshooting.md`
- **Acción:** fila en la tabla de secciones; entrada con síntoma (`LNK1140`/`LNK1104` con el disco a 0), causa raíz medida (tabla `incremental` vs `*.pdb` con los números de hoy), umbral declarado **15 GB** y los 4 comandos, tabla de exit codes, "lo que nunca hace", la nota honesta de `--incremental` roto + remedios, y el `-SelfCheck`
- **Verify:** `rg -n 'AutoSafe|LNK1140' docs/dev/references/troubleshooting.md`
- **Estado:** ✅ DONE

### Step 4: Verify del contrato + gates de docs
- **Archivos:** —
- **Acción:** los dos `-DryRun` del contrato, `-SelfCheck`, rechazo de flags, y `check-docs` / `check-links` / markdownlint
- **Verify:** ver §Contrato (evidencia real) y §Review
- **Estado:** ✅ DONE

### Step 5: Review P2-01 y commit local
- **Archivos:** los 3 de la tarea
- **Acción:** review por agente distinto del bucle de borrado; commit local sin push
- **Verify:** veredicto registrado en §Review + `git show --stat HEAD`
- **Estado:** ✅ DONE

## Dependencias
- `FIND-177` ✅ (`2dd8501af`) — creó `docs/dev/references/troubleshooting.md`, donde se documenta el umbral.
- Wave 1.2, carril C. En paralelo: `FIND-713` (docs/CI), `FIND-259` y `FIND-710` (Rust, **compilando** durante esta tarea → solo dry-runs).
- **No** tocar `dev-tools/verify.ps1`, `src/**`, `.github/**`, `Formula/**`, `docs/dev/Backlog.md`, `docs/dev/plans/**`, `docs/dev/avance/**`, `scripts/docs/**`.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (contexto fresco, leaf read-only) — invocado vía sub-agente; veredicto **❌ cambios requeridos (3 required, 6 optional, 0 críticos)** en la primera ronda, resuelto abajo
- **Tier:** **fast** (paths `dev-tools/**` + `docs/dev/**` → 0 paths adversariales de la tabla de `pipeline-full.md` §Cierre). Se pidió igual la revisión específica del bucle de borrado por blast radius
- **Enfoque:** ¿el bucle de borrado puede salirse de `target/`? ¿el umbral y la guarda de procesos cubren los dos fallos reales (disco a 0 / caché corrupta)? ¿el doc manda a hacer algo peligroso?
- **Cómo se probó:** el revisor ejecutó por su cuenta `-SelfCheck` (16/16, 488 rutas), `-DryRun`, `-AutoSafe -DryRun`, `-AutoSafe -Clean` (exit 1), `check-docs` y `check-links`, y **sondó 6 rutas límite** con la aritmética del validador (`target\.`, `target\debug\..\..\`, `target2\`, etc.). No corrió `-AutoSafe` real ni `-Clean`.
- **Hallazgos del revisor y resolución:**
  | # | Hallazgo | Resolución |
  |---|---|---|
  | **R1** | El doc afirmaba que `-AutoSafe` "nunca toca `deps/`, `debug/` ni `release/`" — **falso**: el propio dry-run lista ~10.75 GB de `*.pdb` *dentro* de `debug/deps` | ✅ corregido: ahora prohíbe borrar los **directorios** y dice explícitamente que sí borra los `*.pdb` de adentro (con el número medido) |
  | **R2** | Dos finales verdes falsos: bajo umbral sin candidatos → `exit 0`, y post-borrado sin re-chequear el umbral → `exit 0` aunque quedaran 9 GB | ✅ nuevo **exit 3** ("sigue bajo umbral") en ambos caminos + documentado en header, task file y troubleshooting; `:211` (disco indeterminable) pasó a `exit 1`; `clippy-driver`/`rustdoc` sumados a la guarda |
  | **R3** | La guarda de builds se muestreaba **una vez** y después corría un bucle de minutos sobre 12 GB: una build que arranca 200 ms después recibe su caché borrada | ✅ re-chequeo **por ítem** antes de cada `Remove-Item` (aborta con `exit 2`); más el salto de reparse points (Optional #5) |
  | **R4** | Fila **Commit** del DoD marcada ✅ sin commit, y `git show --stat HEAD` apuntando a un commit de otro carril | ✅ commit selectivo de los 3 paths al cerrar (ver abajo); la fila se cierra con el hash real |
- **Optionals aplicados:** #1 (`-SelfCheck` ahora usa `Test-Refused`, el mismo validador del borrado — 17/17 y no una re-implementación del `StartsWith`), #2 (disco elegido por longitud de `Root` desc), #3 (exit 2 sin sobrecarga semántica), #4 (assert "hay al menos un objetivo real"), #5 (reparse points omitidos), #6 (aviso de builds en vuelo también en `-Clean -Yes`). Nit de `Cargo.toml` aplicado: "test, bench and example", no "bin" (`[profile.dev] debug = 0`).
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas: los GB, tamaños y conteos citados son de esta sesión y de la sesión del revisor, ambos fechados
  - [x] No saltarse la clarificación: contrato mecánico dado por el orquestador; el único borde ambiguo (¿integrar a `verify.ps1`?) estaba resuelto por escrito en la tarea → §Deuda D1
  - [x] No declarar done sin verificar contra el contrato: 2 `-DryRun` + `-SelfCheck` + rama bajo umbral forzada + rechazo, todos ejecutados
  - [x] No ignorar fallos: el revisor tumbó 3 afirmaciones; las 3 se corrigieron en código/doc, no se argumentaron
  - [x] No single-shot: el revisor evaluó 5 alternativas de diseño (split por artefacto, `Pester`, `Get-Volume`, etc.)
  - [x] No copiar sin citar: la causa raíz de los PDB cita `Cargo.toml:743,856-857`; el umbral cita la medición
  - [x] No reintentos en bucle: 0 fallos de verify; una sola ronda de review
  - [x] Pasos conectados al objetivo: 5/5
  - [x] No degradar chequeos de seguridad: la revisión **endureció** el borrado (re-chequeo por ítem, reparse points, exit 3)
  - [x] Budget explícito: cerrado en dos pasadas (implementación + resolución del review)
  - [x] Cobertura SDP: `documentation-skill` (obligatoria por tocar `docs/**`) + las del ciclo; sin `pinned` en el output v3
- **Veredicto final:** ✅ **approve** — R1-R4 cerrados con evidencia; el propio revisor calificó la puerta de rutas como correcta tras probarla ("incluso el alias `target\.` cierra"), y los cambios pedidos fueron de veracidad del doc y de exit codes, no del mecanismo de borrado.

## Notas
- **Decisión clave:** el gate **no** borra `deps/` ni `target/` completo aunque liberaría más, porque el costo (rebuild en frío de ~40 min) es mayor que el beneficio cuando el disco solo está bajo, no agotado. La limpieza profunda sigue siendo humana e interactiva.
- **`ponytail:`** el walk de `Get-ChildItem -Recurse` sobre `target/` se paga dos veces (una para `incremental/`, otra para `*.pdb`) en vez de mantener un índice de artefactos: si el gate llega a correr en cada wave y los segundos importan, cachear el listado entre invocaciones es el upgrade path.
- **No se regeneró `docs/index.md`/`llms.txt`:** no se agregó ni borró ningún documento (solo una sección dentro de uno existente), así que el índice no cambia. `gen-index --write` arrastraría ficheros de otros carriles.
- **Release N/A:** `chore(dev-tools)` no impacta versión (`release-plz` lo ignora por diseño); no hay cambio de API ni de comportamiento del motor.
- **No se actualizó el plan file** (`docs/dev/plans/**` está prohibido en esta tarea): el orquestador marca la fila de wave 1.2 al cerrar la wave.
