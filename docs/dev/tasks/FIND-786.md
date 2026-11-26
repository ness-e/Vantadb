---
title: "FIND-786 — Secret Scan rojo por 9 hashes SHA-256: allowlist de path en `.gitleaks.toml`, no del valor"
kind: task
status: completed
description: "Los 9 `generic-api-key` del Secret Scan son los SHA-256 de `tokenizer.json` en `embeddings/manifest.lock`, marcados por la keyword `token` del nombre más la entropía del hash. El fichero nunca cambió (diff neto cero). Se cierra con allowlist de PATH anclada al fichero: el gitleaks del job (8.24.3) sólo admite la allowlist global legacy en OR, sin el AND que estrecharía el alcance."
tags: [vantadb, ci, gitleaks, secrets, allowlist, fp]
---

# FIND-786 — el hallazgo era un hash, y el disparador era el nombre

> **Dependencias:** `FIND-778` / `FIND-784` / `FIND-753` (la cadena del wave). El otro extremo de la
> misma cadena —que `skipped` cuente como required satisfecho— no tiene fila propia en el Backlog al
> cerrar esta tarea (verificado: 0 ocurrencias de `FIND-787` en `HEAD` y en `HEAD~1`; la fila que
> apareció en un working tree intermedio nunca se commiteó): lo registra el carril que lo arregle.
> **Carril:** CI, sin compilar. Escritura: `.gitleaks.toml` (exclusivo), este task file, la fila del
> Backlog y `docs/dev/avance/activo/ci-cd.md`. Ni un `.rs`, ni un workflow.

## Contexto

`Secret Scan (gitleaks + secretlint)` reportó **9** `generic-api-key` en `embeddings/manifest.lock`,
entropía 3.74-3.88 (**cifra del monitor del push**; la medición local da 3.638-3.939 — ver
§Verificación), `Finding: "tokenizer.json": "<sha256>"`. Y el fallo arrastraba a `skipped` los
tres `Tests (Linux/Windows/macOS)`, que GitHub cuenta como required **satisfecho**.

**Re-medido acá, no heredado de la fila.** El diff neto del rango es cero:

```bash
git diff c8e73e412..43e41f055 -- embeddings/          # vacío
git log --oneline -3 -- embeddings/manifest.lock
# db6ad7f79, 290df23d0 (repo-integrity), 19f759980 (restauración)
```

Los hallazgos salen porque el path se borró y se restauró dentro del rango: `72643d9df` **borró**
2365 paths y la restauración los devolvió — **2363** en `19f759980` (`A=2363`; el subject del commit
dice 2365, que es el número de borrados de `72643d9df`, `D=2365`) y otra vez en `db6ad7f79`. gitleaks
escanea **commits del rango**, así que las 9 líneas restauradas cuentan como **adiciones** en cada una
de las dos restauraciones. El **contenido** del fichero no cambió (diff neto cero); el **historial**
sí.

**Qué dispara el FP — verificado, porque cambia el allowlist.** En el mismo fichero conviven dos
estructuras idénticas:

```json
"onnx/model.onnx": "828e1496d7fabb79cfa4dcd84fa38625c0d3d21da474a00f08db0f559940cf35",   // no se marca
"tokenizer.json":  "d241a60d5e8f04cc1b2b3e9ef7a4921b27bf526d9f6050ab90f9267a1f9e5c66"    // sí se marca
```

Mismo tipo de valor, misma longitud, entropía parecida: la única diferencia es la keyword `token`
en el **nombre** del artefacto. El `Secret` que reporta gitleaks es el **hash**, pero el disparador
es el **nombre**. Con evidencia en las dos direcciones (§Verificación, sondas `p5`/`p8`): la misma
forma de valor **fuera** de este fichero se sigue marcando, o sea que no es el hash lo que hay que
declarar inocuo, es el fichero.

## Contrato

1. `Secret Scan` vuelve a verde **declarando** que esos valores son hashes de integridad, no
   silenciando la clase `generic-api-key`.
2. Una sola vía entre (a) allowlist de **path** con el motivo escrito, (b) allowlist de **valor**
   (más ancha: silencia cualquier secreto con esa forma en todo el repo), (c) `# gitleaks:allow`
   por línea (frágil en un JSON generado de 96 líneas).
3. Prohibido `--no-verify`, borrar el job o `continue-on-error`.
4. Prueba de no-vacuidad: un token falso con forma de credencial **fuera** del allowlist tiene que
   seguir saliendo rojo.

## Impacto mapeado (Regla 0)

| Archivo | Leído | Referencias hacia dentro | Impacto |
|---|---|---|---|
| `.gitleaks.toml` | completo (116 L → 131 L) | `.github/workflows/ci-rust.yml:212` (`GITLEAKS_CONFIG`), `.secretlintignore` (lista paralela), `ci-rust.yml:25,170` (`on.push.paths` y el gate de scope) | **se edita**: una entrada de path en la `[allowlist]` global, con el motivo escrito. Formato **legacy** conservado a propósito (decisión 1) |
| `embeddings/manifest.lock` | completo (96 L) | tooling de `embeddings/` | **sólo lectura** — el fichero está bien: cada valor es el SHA-256 del artefacto |
| `.github/workflows/ci-rust.yml` | job `secret-scan` (192-220) y `on.push.paths` (22-28) | — | **sólo lectura** — prohibido por el contrato (otro carril está en `snapshot-certification.yml`) |
| `.secretlintignore` | completo | `.secretlintrc.json` | **sólo lectura y sin cambios**: `secretlint` sobre `embeddings/manifest.lock` sale **exit 0** (§Verificación), así que no necesita espejo |
| `docs/dev/Backlog.md` | sección 880-884 | — | **se retira la fila `FIND-786`** (y su encabezado de sección: era la única fila, verificado) |
| `docs/dev/avance/activo/ci-cd.md` | entradas de la ola | — | **se añade una entrada** al final |

## Spec — decisiones

| # | Decisión | Alternativas rechazadas |
|---|---|---|
| 1 | Se **conserva la `[allowlist]` legacy** (no se migra a `[[allowlists]]`), y la entrada nueva va dentro de ella | **`[[allowlists]]`**: el job no usa "un gitleaks", usa el que instala `gitleaks-action@v2.3.8`, que por defecto es **8.24.3** (`dist/index.js`: `process.env.GITLEAKS_VERSION \|\| "8.24.3"`). En 8.24.3 el `ViperConfig` global **no tiene** el array `Allowlists` ni campo `Condition` (`config/config.go`, structs `ViperConfig.Allowlist` y `viperRuleAllowlist`): una tabla `[[allowlists]]` se **ignora en silencio**. Medido: con ella puesta, 8.24.3 seguía reportando **11** hallazgos en la sonda `p4` (los 9 + 2 plantados). Habría sido un commit que *parece* arreglar y deja el rojo. Y 8.30.1 **aborta** si conviven las dos formas: `FTL Failed to load config "[allowlist] is deprecated, it cannot be used alongside [[allowlists]]"` |
| 2 | **Allowlist de PATH** (opción (a) del contrato), anclada `(^\|/)embeddings/manifest\.lock$`, con el motivo escrito al lado | **(b) regex sobre el valor** (`[0-9a-f]{64}`): en 8.24.3 la allowlist global **no tiene `condition`**, se evalúa en **OR**, así que una regex de valor entra sin acotar por fichero → valdría para **todo** el repo y silenciaría cualquier secreto de 64 hex. Medido en 8.30.1 (mismo OR por defecto): una entrada con `paths` **y** `regexTarget=line` + regex silenció además una línea `"api_key": "sk-proj-…"` **dentro** del fichero permitido — la forma exacta del pre-mortem (a). El "AND path+regex" que estrecharía esto **no es expresable** en la allowlist global de 8.24.3. **(c)** `# gitleaks:allow`: 9 líneas de un JSON **generado** (se regenera y las marcas se pierden), y `dev-tools/floor-guard.ps1:59` cuenta `gitleaks:allow` como supresión |
| 3 | El motivo escrito dice **qué** se silencia y **cuánto** | Un allowlist sin techo es una promesa sin fecha: la entrada declara que el alcance es un fichero generado y que un secreto pegado a mano ahí **no** se detectaría (Deuda) |
| 4 | No se toca `.secretlintignore` | `secretlint` con el preset recomendado sale **exit 0** sobre `embeddings/manifest.lock` (medido). Añadir el path habría apagado un gate que hoy no molesta, y el fichero está fuera de la lista de escritura |
| 5 | Anclaje `$` + prefijo `(^\|/)` en la ruta | Medido: `embeddings/manifest.lock.bak` y `embeddings/models/x/manifest.lock` **siguen** marcándose (sondas `p6`/`p7`), o sea que no se desborda a hermanos ni a subdirectorios. **Borde declarado (review L2):** `(^\|/)` acepta cualquier padre, así que un `foo/embeddings/manifest.lock` **sí** queda cubierto (medido, sonda `p11` → 0); se conserva porque es el criterio del resto del fichero y porque una ruta **absoluta** (`--source` fuera de la raíz del repo) dejaría de casar con un `^embeddings/…`. Residual: 1 path, declarado en la entrada |

## Pasos

| # | Step | Estado |
|---|---|---|
| 1 | Reproducir el rojo con el gitleaks **del job** (8.24.3), no con el último | ✅ 9 en `dir embeddings`, 18 en el rango de commits |
| 2 | Aislar el disparador (hash vs. nombre) con las líneas hermanas y sondas cruzadas | ✅ keyword del nombre + entropía; el valor aislado no basta |
| 3 | Elegir la forma de allowlist que **funciona en 8.24.3** (no la que funciona en la última) | ✅ legacy `[allowlist]`; `[[allowlists]]` se ignora en silencio |
| 4 | Escribir la entrada con motivo, alcance y anclaje | ✅ |
| 5 | Verificar 9 → 0 en `dir` y `git` (el mecanismo real del job) con 8.24.3 y 8.30.1 | ✅ |
| 6 | Probar que sigue cazando: 6 sondas fuera del allowlist, incluida la misma forma | ✅ 6/6 detectadas |
| 7 | Cierre de docs: task file, fila del Backlog, `ci-cd.md` | ✅ |

## Verificación

Gitleaks **8.24.3** es el binario que instala el job (`gitleaks-action@v2.3.8`, default
`GITLEAKS_VERSION`), así que es el que manda; 8.30.1 se usó además para el contraste. Nota de
sintaxis: `detect --no-git` **sigue aceptado** en las dos versiones (verificado con `detect --help`:
*"treat git repo as a regular directory and scan those files"*), pero está deprecado; la forma
moderna del comando del contrato es `gitleaks dir` y la del job `gitleaks git --log-opts <rango>`.

| Check | Antes | Después |
|---|---|---|
| `gitleaks dir embeddings -c .gitleaks.toml` (8.24.3) | ❌ **9** leaks | ✅ **0** |
| `gitleaks dir embeddings -c .gitleaks.toml` (8.30.1) | ❌ 9 | ✅ **0** |
| `gitleaks git --log-opts c8e73e412..43e41f055` (8.24.3, **el mecanismo del job**) | ❌ **18** (56 commits escaneados por gitleaks, 102.34 MB) | ✅ **0** |
| `secretlint embeddings/manifest.lock` (preset recommend) | exit **0** | exit 0 (sin cambios) |

Los 9 del `dir`, todos `generic-api-key`, `File: embeddings/manifest.lock`, líneas
`10,19,28,37,46,55,64,74,84`, entropía **3.638-3.939** (medición local; el monitor del push reportó
3.74-3.88, un subconjunto redondeado de los mismos 8 valores únicos). `Secret` = el hash de 64 hex.
Los 18 del rango son esas 9 líneas **× 2 restauraciones**, no "borrado + restauración": el report
JSON los agrupa por commit en `19f759980` → 9 y `db6ad7f79` → 9, los dos con `git diff-tree` en
**`A`** (adiciones); el commit de borrado (`72643d9df`, `D=2365`) **no** produce findings — gitleaks
no marca lo que desaparece. El CI reportó 9 sobre su rango (19 commits / 51,08 MB), que es un
subconjunto del local — mismo fichero, misma regla.

**Prueba de que sigue cazando** (ficheros temporales en `%TEMP%`, config final, **8.24.3**):

| Sonda | Contenido | Resultado |
|---|---|---|
| `p1` | `aws_key = "AKIAZ7Q2XK4M9PLR3TUV"` | ✅ `aws-access-token` |
| `p2` | `openai = "sk-proj-…"` (60 chars) | ✅ `openai-api-key` |
| `p5` | la **misma** línea `"tokenizer.json": "<hash real>"` fuera de `embeddings/` | ✅ `generic-api-key` |
| `p8` | `api_token = "<hash real>"` en un `.txt` cualquiera | ✅ `generic-api-key` |
| `p6` | `embeddings/manifest.lock.bak` (el allowlist no debe cubrirlo) | ✅ 9 |
| `p7` | `embeddings/models/x/manifest.lock` (ni prefijos ni profundidad) | ✅ 9 |
| `p3` | copia en **la** ruta permitida | ✅ 0 (lo único silenciado) |
| `p11` | `foo/embeddings/manifest.lock` (borde del `(^\|/)`: **sí** lo cubre, review L2) | ❌ 0 — declarado |
| `p9`/`p10` | hash solo y `"random.json": "<hash>"` en otro fichero | ✅ 0 findings: sin keyword no hay hallazgo |

**Trampa encontrada, y por qué importa:** la primera sonda usó el ejemplo canónico
`AKIAIOSFODNN7EXAMPLE` y **no** salió. No era mi allowlist: el config por defecto de gitleaks lleva
`EXAMPLE` como stopword de `aws-access-token`. Con esa sonda, la conclusión "el allowlist está
apagando todo" habría sido falsa y con la evidencia a favor. La prueba buena necesita un valor
**no** de ejemplo — que es lo que hacen `p1` y `p2`.

## Deuda

- **El alcance declarado del allowlist es el fichero entero**: un secreto real pegado a mano en
  `embeddings/manifest.lock` no se detectaría. Se acepta porque el fichero es generado y cada valor
  se recomputa contra el artefacto real; el techo queda escrito al lado de la entrada, no en un
  ticket que nadie lee.
- **El formato legacy está deprecado, pero no roto.** `gitleaks-action` fija `8.24.3`; la `[allowlist]`
  suelta **sigue funcionando** en 8.30.1 (verificado: este config parsea y da **0** ahí también). El
  **error fatal** aparece sólo si se **añade** `[[allowlists]]` sin borrar la legacy
  (`"[allowlist] is deprecated, it cannot be used alongside [[allowlists]]"`), o sea que la migración
  tiene que ser **atómica**. Cuando se quiera la forma estrecha, el cambio es: borrar la `[allowlist]`
  y poner en `[[allowlists]]` `condition = "and"` + `paths` + `regexTarget = "line"` + la regex
  `^\s*"[^"]{1,80}"\s*:\s*"[0-9a-fA-F]{64}"\s*,?\s*$` — el AND que en 8.24.3 es inexpresable. Queda
  escrito acá para que el bump no se lea como un rojo misterioso.
- **`--source .` completo no es reproducible en este disco.** `target/` son 52 GB en 120.563
  ficheros y `embeddings/models/` 8 GB de binarios: el `dir` sobre la raíz no termina en un tiempo
  razonable (se abortó). La medición se hizo donde el hallazgo vive (`embeddings/`) **y** con el
  mecanismo real del job (rango de commits), que es el que produjo el rojo. En CI el árbol es limpio
  y el escaneo es por commits, así que la diferencia no afecta al veredicto — pero consta.

## Review

**P2-01 — `vanta-review`, contexto fresco, una ronda → `changes-required`: 1 High + 3 Medium + 4 Low
+ 4 nits, todos cerrados.** El revisor **re-ejecutó** la medición en vez de heredarla (mismos
binarios, mismas sondas) y el veredicto sobre lo técnico fue explícito: *"el cambio técnico
(`.gitleaks.toml`) está **aprobado y verificado**; el cierre documental no"*.

| # | Sev | Hallazgo del revisor | Estado |
|---|---|---|---|
| H1 | **High** | `status: completed` con `## Review` vacío (gate P2-01) y sin la línea `**Review P2-01:**` en la entrada de `ci-cd.md` | ✅ esta sección + la línea en `ci-cd.md` |
| M1 | Medium | La Deuda decía "a partir de 8.30 la `[allowlist]` suelta es error fatal", y **contradecía** la entrada de avance. Medido por el revisor: legacy sola en 8.30.1 → parsea y da 0; el fatal es sólo al **convivir** con `[[allowlists]]` | ✅ reescrito: "deprecado, no roto; la migración tiene que ser atómica" |
| M2 | Medium | "en 8.24.3+ ya no existe `detect --no-git`" — **falso**: `detect --help` lo lista en 8.24.3 y 8.30.1 (*"treat git repo as a regular directory…"*). Confirmado por mí con el binario | ✅ "sigue aceptado (deprecado); la forma moderna es `gitleaks dir`" |
| M3 | Medium | "18 = 9 líneas × 2 (borrado + restauración)" — atribución **falsa**. El report agrupa `19f759980` → 9 y `db6ad7f79` → 9, los dos **adiciones** (`git diff-tree`: `A`); el borrado no produce findings | ✅ reescrito: "× 2 **restauraciones**" |
| L1 | Low | `19f759980` restauró **2363** paths, no 2365 (`git diff-tree`: `A=2363`; `72643d9df`: `D=2365`, y el subject del commit dice 2365 = los borrados) | ✅ corregido con las dos cifras y su origen |
| L2 | Low | `(^\|/)` no es "ruta exacta": `foo/embeddings/manifest.lock` también se silencia (`p11` → 0 medido por el revisor) | ✅ **declarado** en el comentario del config y en la decisión 5; se conserva el prefijo (es el criterio del resto del fichero y una ruta absoluta dejaría de casar con `^embeddings/…`) |
| L3 | Low | Cita `FIND-787` como fila abierta, y **no existe** en el Backlog ni como task file | ✅ verificado (`git grep` en `HEAD` y `HEAD~1`: 0); las referencias ya no afirman que haya fila |
| L4 | Low | `git diff --check` en `Backlog.md`: *new blank line at EOF* | ✅ blanco final eliminado |
| N1 | Nit | `.gitleaks.toml` "110 L" stale | ✅ 116 → 131 |
| N2 | Nit | "el fichero nunca cambió" → el **contenido** no, el **path** sí | ✅ precisado |
| N3 | Nit | "56 commits" vs `git rev-list` = 58 | ✅ "56 escaneados por gitleaks" (58 es el raw del rango) |
| N4 | Nit | Dos rangos de entropía sin fuente | ✅ cada cifra etiquetada con su origen |

**Lo que resistió sin cambios (re-ejecutado por el revisor):** la action es `v2.3.8` y su
`dist/index.js` fija `GITLEAKS_VERSION \|\| "8.24.3"` (el workflow no la define) · 8.24.3 tiene
`ViperConfig.Allowlist` y **no** `Allowlists`/`Condition` global, y una `[[allowlists]]` se ignora en
silencio (medido: `p3` seguía con 9) · 8.30.1 con las dos formas → FTL exacto · el anclaje (`.bak` →
9, `embeddings/models/x/…` → 9, ruta exacta → 0) · la no-vacuidad 6/6 · los números 9→0 / 18→0 /
102,34 MB / `secretlint` exit 0 / diff neto vacío · la higiene de carril (4 paths, ningún path
prohibido) · la convención de cierre de `FIND-784`. También validó las tres alternativas
descartadas (`[[allowlists]]`, regex de valor, `# gitleaks:allow`) como **bien** rechazadas.

## NOTICED BUT NOT TOUCHING

- **`docs/` está en la allowlist global entera.** Es anterior a esta tarea y es el mismo patrón que
  acá se acota: cualquier secreto commiteado bajo `docs/` no se detecta. No se toca (fuera de
  alcance y con 26 ficheros que citan ese allowlist), pero es el siguiente que hay que revisar.
- **La semántica `skipped` == required satisfecho** es el otro extremo de este rojo y multiplica su
  daño: con el `Secret Scan` verde, `skipped` deja de ser la vía de escape **de este** fallo, no del
  siguiente. No tiene fila en el Backlog al cerrar acá (verificado: 0 ocurrencias de `FIND-787` en
  `HEAD` y `HEAD~1`); lo registra el carril que lo arregle.
- **`ci-rust.yml:170`** lista `.gitleaks.toml` en el gate de scope del job Rust: tocar el config
  dispara esa lane. Es correcto (el config es gate) y no se cambió.
- **`embeddings/models/**`** (8 GB de binarios) no está en la allowlist y no genera hallazgos: la
  hipótesis "hay que permitir `embeddings/`" era falsa en las dos direcciones.
