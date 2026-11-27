---
title: "LOTE gates y CI - 6 tareas, especificacion completa"
kind: task
status: active
description: "Especificacion ejecutable de FIND-787, 814, 815, 816, 821 y 810: contrato, verificacion, comandos, dependencias y skills por tarea."
id: LOTE-gates
---

# LOTE gates y CI — 6 tareas

**Owner:** 4 días hasta el 01-dic. Nada diferido. **Un `FIND` nuevo se resuelve en el acto.**

## Orden

```
787 (cerrar con evidencia) → 814 (gitleaks) → 815 (guard upstream) → 816 (docs-gates) → 821 (hooks .opencode) → 810 (citas .rs)
```

**Por qué:** `787` primero porque es el que hace que los demás verdes **signifiquen algo**. `810` al final porque toca `.rs` de 4 crates y el `pre-commit` correrá clippy.

---

## FIND-787 — `skipped` satisface un required check (CERRAR)

**Severidad:** 🟠 Alta · **Estado:** el fix **ya está hecho**; falta **verificar y retirar la fila**

**Los tres hechos que hay que verificar antes de retirarla:**
1. El guard existe: `f39ea167c` — job `tests-guard` (`name: Tests Executed (guard)`, `needs: [rust-scope, test, test-windows, test-macos]`, `if: always()`).
2. Está **required** en el ruleset `23692587` de `develop`: `gh api repos/ness-e/Vantadb/rulesets/23692587 --jq '.rules[] | select(.type=="required_status_checks") | .parameters.required_status_checks[].context'` ⇒ **15 contexts**, con `Tests Executed (guard)` dentro.
3. Su `fail-path` se **probó en producción**: PR #292 (rama desechable, `secret-scan` roto a propósito) ⇒ `Tests (Linux/Windows/macOS)` `skipped` ×3 ⇒ guard **`failure`** con 4 anotaciones nombrando los jobs ⇒ merge **`BLOCKED`**. PR cerrado sin merge, rama borrada.

**Contrato.** Si los tres se sostienen ⇒ **retirar la fila** con la evidencia (commit + run id + PR). Si alguno no ⇒ **no la retires** y decí cuál falla.

⚠️ **No basta con el harness local de 6 caras**: el hecho 3 es el que prueba que GitHub lo evalúa así.

---

## FIND-814 — `tests/` allowlisteado entero en gitleaks

**Severidad:** 🟠 Alta · **Esfuerzo:** 30-60 min

**Qué rompe.** `.gitleaks.toml` tiene `'''(^|/)tests/'''` en la allowlist global ⇒ **un secreto real dentro de `tests/` es invisible** al `Secret Scan`. Es la **misma clase** que el `docs/` allowlisteado entero que se detectó al cerrar `FIND-786`.

**Descubrimiento:** lo midió el carril del `fail-path` de `FIND-787` al buscar dónde poner una credencial de prueba: un probe en `tests/` **habría sido invisible**, así que el experimento habría concluido "el guard no funciona" cuando el problema era que **el escaneo no miraba**.

**Contrato.** Reemplazar el allowlist de **directorio** por allowlists **por fichero o por huella**: los fixtures que necesiten una credencial falsa la declaran con **`gitleaks:allow` en su línea**, que es explícito y auditable; el resto de `tests/` vuelve a estar escaneado.

**Verificación — las dos direcciones:**
- Un `AKIA…` con sufijo **aleatorio** dentro de `tests/` **dispara**. ⚠️ **Trampa medida**: `AKIAIOSFODNN7EXAMPLE` **no** dispara porque `EXAMPLE` es stopword del config por defecto. Usá `AKIAZ7Q2XK4M9PLR3TUV` o equivalente.
- El mismo valor con `gitleaks:allow` en la línea **no** dispara.
- El escaneo completo sigue en **0** en `HEAD`.

**Comando.** El job usa **gitleaks 8.24.3** (no la última): verificá con **esa** versión, en los dos modos (`gitleaks dir <path>` y `gitleaks git --log-opts=<rango>`).

---

## FIND-815 — El guard solo mide si `rust-scope=true`

**Severidad:** 🟠 Alta · **Esfuerzo:** 2-4 h

**Qué rompe.** El guard tiene una rama **legítima** de skip (`SCOPE=false && SCOPE_RESULT=success` ⇒ `exit 0`) porque sin cambios de Rust no hay suites que ejecutar. Pero esa misma rama significa que **un PR de solo-docs que rompa `secret-scan` no bloquea**: los tests se saltan, el guard ve `SCOPE=false`, y el candado se cierra con el escaneo en rojo.

**El caso real ya existió:** en `43e41f055` el `Secret Scan` falló y las tres suites quedaron `skipped`.

**Contrato.** El guard debe juzgar **dos** cosas:
1. si las suites tocaban ejecutarse y no ejecutaron ⇒ rojo (**lo que ya hace**);
2. si un check **upstream** que alimenta a las suites falló (`secret-scan`, `fmt`, `clippy`) ⇒ **rojo aunque el scope sea docs-only**.

La condición de skip deja de ser *"no hay Rust"* y pasa a ser *"no hay Rust **y** los upstream están verdes"*.

**Verificación.** PR docs-only con `secret-scan` roto ⇒ guard **rojo**. PR docs-only con todo verde ⇒ guard verde (sin coste de runner).
⚠️ **No apliques cambios de ruleset.** Si el arreglo los requiere, proponé el JSON exacto.

---

## FIND-816 — `docs-gates.yml` filtra el push por 21 patrones y el 18 % queda fuera

**Severidad:** 🟠 Alta · **Esfuerzo:** 2-4 h

**Qué rompe.** Filtra el `push` por **21 patrones** con `docs/**` primero ⇒ **426 de 2351 ficheros trackeados (18 %) caen fuera** del filtro. Y su `:64` **niega** lo que su `:51` declara. Efecto: si el push no dispara `docs-gates`, sus **4 required quedan ausentes** y el candado no mide la documentación.

**Dónde.** `.github/workflows/docs-gates.yml:51` · `:64`.

**Contrato.** La **misma solución que `FIND-807`** (`1c6cee680`): **el filtro se resuelve dentro del job**, no en `on.push`. `FIND-807` ya lo hizo moviéndolo a `rust-scope` con `github.event.before` ⇒ **reusá ese idioma**, no inventes otro. Y un chequeo que compare los 21 patrones contra los ficheros que los gates realmente juzgan.

**Verificación.** Un push que solo toque un fichero **fuera** de los 21 patrones **publica** los 4 contexts. Y `:64` deja de afirmar cobertura completa o la implementa.

---

## FIND-821 — `.opencode` es un submódulo sin hooks

**Severidad:** 🟠 Alta · **Esfuerzo:** 2-4 h

**Qué rompe.** Medido hoy **dos veces** al commitear `AGENTS.md`:

```
WARNING: no pre-commit hook under .../.git/modules/.opencode/hooks
         - committing WITHOUT the gate.
```

**El repositorio que define los gates no los aplica a sí mismo.** Todo lo que vive ahí —prompts del pipeline, agentes, skills, el helper, los chequeos de contrato— se commitea **sin markdownlint, sin actionlint, sin la compuerta de área staged y sin la verificación por contenido**. Y lo que vive ahí es **lo que verifica a los demás**.

Es la misma clase del día en la capa más alta: `FIND-797` (la lane no dispara con lo que certifica), `FIND-807` (11 de 15 ausentes), `FIND-816` (18 % fuera) — aquí el filtro es **el repositorio entero**.

**Contrato — elegí:**
- (a) instalar los **mismos hooks** en el submódulo (el helper ya detecta y avisa); o
- (b) que el commit del padre **verifique el SHA del submódulo** con `check-harness-contract.mjs` antes de aceptarlo.

**Verificación.** Un commit en `.opencode` con un fichero inválido **falla**. Y el helper deja de imprimir el WARNING.

---

## FIND-810 — Cuatro ficheros citan `vantadb/src/lib.rs`

**Severidad:** 🔵 Baja · **Esfuerzo:** 15-30 min

**Qué rompe.** Cuatro `lib.rs` citan `vantadb/src/lib.rs` **en comentarios**: el mismo prefijo que **`api-contract.md:31` (R-4) prohíbe** y que el propio `ADR-0063` arrastraba en su texto (ya corregido en `7aaa6c239`).

**Dónde.** `vanta-memory/src/lib.rs` · `vanta-proxy/src/lib.rs` · `vantadb-mcp/src/lib.rs` · `vantadb-wasm/src/lib.rs`.

**Contrato.** Los 4 comentarios apuntan a **`src/lib.rs`** (o al path real del crate). Son **comentarios**: no rompen compilación, pero **propagan el prefijo** con el que se escriben los contratos siguientes — el bug real de hoy fue un ADR que mandaba implementar sobre un fichero inexistente.

**Verificación.** `rg 'vantadb/src/' --glob '*.rs'` ⇒ **0**.
⚠️ Son `.rs` de 4 crates ⇒ el `pre-commit` correrá clippy. `SKIP=cargo-fmt,cargo-clippy` **declarado** + `cargo check -p <crate>` **si** el disco lo permite (13 GB, dos carriles Rust).

---

## Proceso del lote

1. **Verificar antes de tocar**: cada fila trae su medición; re-ejecutala.
2. **Un commit por fila**, con el helper y salida capturada.
3. **Review P2-01** una ronda.
4. **`FIND` nuevo**: se resuelve en el acto — los workflows son ficheros disjuntos, así que casi siempre **está en tu zona**.

## Gates de validación

| Gate | Criterio |
|---|---|
| `actionlint` | exit 0 con y sin shellcheck, **cronometrado**. ⚠️ **Cuelga** si un `run:` gana una sentencia (medido: **33 h** bloqueado con 0,8 s de CPU). Lógica nueva ⇒ **compuesta** (`case`, función) |
| gitleaks | **8.24.3**, los dos modos |
| `check-docs` / `check-links` | exit 0 |
| `check-harness-contract --self-test` | **89/89** si lo tocás |
| Commit | helper, códigos `7`/`8`/`9`, `-ExpectAbsent` declarado |

## Riesgos

- **`actionlint` cuelga**: es el riesgo #1 del lote. Cada YAML modificado, cronometrado y con tope.
- **No aflojar gitleaks** para "limpiar": quitar cobertura real es peor que el falso positivo.
- **No aplicar el ruleset**: es del owner.
- **`certification.yml` / `release-verify.yml` / `snapshot-certification.yml` son de otro carril** — no los toques.
