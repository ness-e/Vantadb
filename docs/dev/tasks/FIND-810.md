---
title: "FIND-810 - cuatro `lib.rs` citaban `vantadb/src/lib.rs`, el prefijo que R-4 prohíbe"
kind: task
status: completed
description: "Cuatro comentarios de cabecera citaban `vantadb/src/lib.rs` (prefijo falso para la crate raíz, prohibido por api-contract.md R-4). Cambiados a `src/lib.rs`; el commit se bloqueó un rato por un probe roto de otro carril en `src/wal.rs` (el `pre-commit` compila el árbol) y se superó al desaparecer el probe."
tags: [vantadb, docs, comentarios, api-contract, ruta, bloqueado]
---

# FIND-810 - el prefijo que la norma prohíbe, en cuatro comentarios

## Contexto

R-4 (`.opencode/rules/api-contract.md:31`) lo dice literal:

> **Must not:** citar `src/sdk.rs` cuando el módulo es `src/sdk/` (EMBEDDED_SDK.md lo
> cita; no existe), ni usar el prefijo falso **`vantadb/src/...`** para la crate raíz.

Los cuatro `lib.rs` lo repetían en su comentario de cabecera, siempre la misma frase:

```text
// ERR-CORE-02: keep clippy deny on prod code; tests may `unwrap`/`expect`
// freely (same pattern as vantadb/src/lib.rs).
```

`vanta-memory/src/lib.rs:2` · `vanta-proxy/src/lib.rs:2` · `vantadb-mcp/src/lib.rs:4` ·
`vantadb-wasm/src/lib.rs:4`. Son comentarios: no rompen compilación, pero **propagan el
prefijo con el que se escriben los contratos siguientes** — el bug real que originó la
fila fue un ADR que mandaba implementar sobre un fichero inexistente.

## Contrato

Los 4 comentarios apuntan a `src/lib.rs` (o al path real del crate). Verificación:
`rg 'vantadb/src/' --glob '*.rs'` → **0**.

## Impacto mapeado (Regla 0)

**Leídos completos:** los 4 `lib.rs` (cabecera y atributos internos), R-4 en
`.opencode/rules/api-contract.md:28-33`.

**Referencias hacia dentro:** ninguna — el texto vive dentro de un `//` y no lo
consume ninguna herramienta. `ERR-CORE-02` es la regla que documenta el patrón
(`#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]`), y sigue
diciendo lo mismo.

**Referencias hacia afuera:** ninguna.

**Veredicto:** cambio de texto en 4 comentarios, 1 línea por fichero. El único riesgo
es de **herramientas** (el `pre-commit` corre clippy sobre cualquier `.rs` staged, y
este carril tiene prohibido compilar Rust): se declara abajo con `SKIP=` explícito.

## Spec - decisiones

| # | Decisión | Elegido | Alternativa descartada | Por qué |
|---|---|---|---|---|
| D1 | Texto nuevo | `same pattern as the root crate's \`src/lib.rs\`` | `same pattern as src/lib.rs` | dentro de `vanta-memory/src/lib.rs`, "`src/lib.rs`" a secas se lee como **su propio** fichero. "the root crate's" fija la crate sin repetir el prefijo prohibido. |
| D2 | ¿Se toca la norma R-4 o el `ADR-0063`? | **no** | corregir el erratum del ADR | son decisiones del owner (el ADR ya se corrigió en `7aaa6c239`); esta fila sólo arregla los comentarios, que es lo que su contrato pide. |

## Estado — el `clippy` se saltó por WIP ajeno (`FIND-823`), y por qué eso no debilita la fila

El cambio se aplicó y verificó a la primera; lo que costó fueron **tres intentos de
commit**, dos bloqueados por código roto de **otro carril** en `src/` (zona prohibida
aquí):

```text
intento 1 — src\wal.rs:135                     error: expected `;`, found `#`
                                               Vec::new() // PROBE: identifier disabled
intento 2 — src\storage\vfile_crypto.rs:377    error[E0425]: cannot find type `Path` in this scope
             src\wal.rs:920                    error[E0382]: borrow of moved value: `path`
```

El motivo era estructural: `dev-tools/verify_staged.ps1` hacía `cargo clippy -p <member>`
**sin leer `SKIP`**, y tres de los cuatro members compilan la crate raíz - cualquier rojo
ajeno en `src/**` bloqueaba el commit de **todos**. Eso se registró como **`FIND-823`** y
**ya está arreglado** (`f5ac57998`): `verify_staged.ps1` honra `SKIP=cargo-clippy|clippy`
imprimiendo `SKIPPED on request ... NOT verified` (no en silencio), y avisa cuando hay
`.rs` bajo `src/` sin commitear ("clippy compila el WORKTREE, no tu commit").

**Gate sustituido, declarado:** el commit se hizo con **`SKIP=cargo-fmt,cargo-clippy`**, y
el `clippy` se saltó **por WIP ajeno en `src/**` (`FIND-823`)**, no por comodidad:
`cargo clippy -p vantadb` no podía pasar en este árbol por código que no es de esta fila,
y `--no-verify` está prohibido (no se usó — hubo además un intento que salió con
`exit 2`, "nothing was created", sin crear nada).

**Por qué no debilita la verificación:** las cuatro líneas son **comentarios** dentro de
un `//` de cabecera, **por encima** de los atributos internos (`#![cfg_attr(...)]`): no
pueden alterar el parseo ni el lint. La verificación real de la fila es la **negativa**
—`rg 'vantadb/src/' --glob '*.rs'` → 0—, y esa no depende de compilar.

## Verificación — las dos direcciones

```text
$ rg -n 'vantadb/src/' --glob '*.rs' -g '!target'      # antes
vantadb-wasm\src\lib.rs:4:// freely (same pattern as vantadb/src/lib.rs).
vanta-proxy\src\lib.rs:2:// freely (same pattern as vantadb/src/lib.rs).
vanta-memory\src\lib.rs:2:// freely (same pattern as vantadb/src/lib.rs).
vantadb-mcp\src\lib.rs:4:// freely (same pattern as vantadb/src/lib.rs).

$ rg -n 'vantadb/src/' --glob '*.rs' -g '!target'      # despues
(exit 1: sin coincidencias)  -> 0

$ git diff --numstat -- vanta-memory/src/lib.rs vanta-proxy/src/lib.rs \
                        vantadb-mcp/src/lib.rs vantadb-wasm/src/lib.rs
1       1       vanta-memory/src/lib.rs
1       1       vanta-proxy/src/lib.rs
1       1       vantadb-mcp/src/lib.rs
1       1       vantadb-wasm/src/lib.rs
```

La dirección que importa es la segunda: el gate de la fila no busca "el texto nuevo
está", busca **"el prefijo prohibido no está"**, y eso es lo que da 0.

## Deuda

- **No se compiló Rust** (disco: **8,6 GB libres** al escribir esto, y dos carriles Rust
  vivos): `cargo check -p <crate>` queda pendiente y **el `pre-commit` corrió con
  `SKIP=cargo-fmt,cargo-clippy` declarado**. El cambio es texto dentro de un comentario
  `//` de cabecera (por encima de los atributos internos): no puede alterar el parseo ni
  el lint. La compilación la harán los carriles que ya tienen el `target/` caliente.
- **El prefijo sigue vivo fuera de `*.rs`**: el contrato de esta fila es
  `rg 'vantadb/src/' --glob '*.rs'` → 0. El barrido de `.md` (incluido el erratum del
  `ADR-0063`) es del carril de docs.

## Review

**P2-01 - `vanta-review`, contexto fresco, una ronda:** ver el `RESULTADO` del lote.

## NOTICED BUT NOT TOUCHING

- `api-contract.md:31` (R-4) menciona además `src/sdk.rs` vs `src/sdk/` en
  `EMBEDDED_SDK.md`: ese fichero está modificado en el árbol por otro carril
  (`docs/api/EMBEDDED_SDK.md`) y no es de esta fila.
- `src/**` (crate raíz) es zona prohibida de este lote.
