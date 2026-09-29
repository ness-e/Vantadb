---
title: "`gate-api-docs.yml` — GATE: API — surface vs documentation"
kind: runbook
status: active
description: Falla el PR que cambia la superficie pública de la API sin cambiar un solo documento
tags: [vantadb, ci, gate-api-docs, documentation, api]
related: [.github/workflows/gate-api-docs.yml, scripts/docs/check-api-docs.mjs, RULES.md, gate-docs-links.md, TRIGGERS.md]
---

# `gate-api-docs.yml` — GATE: API — surface vs documentation

## ¿Qué hace?

Una regla, y es la regla completa: **si la superficie pública de la API cambia y
`docs/` no cambia, el PR falla.**

El patrón es `NeedsDocumentation.yml` de DuckDB. Es la barrera más barata contra
la deriva de API, y de las que este repositorio no tenía: todos los gates que ya
existen en `scripts/docs/` comprueban que los documentos estén **bien formados**.
Ninguno comprueba que sean **verdaderos**.

## ¿Por qué hace falta? Deriva ya verificada

| Documento | Afirma | Realidad |
|---|---|---|
| `llms.txt` | `db.search_memory()` | El método no existe |
| `docs/api/PYTHON_SDK.md` | excepción `VantaError` | Se llama `Error` |
| `docs/api/PYTHON_SDK.md` (Quick Start) | `x.get(...)` sobre algo | Ese algo devuelve una lista |

Nada en CI lo detectó. La causa no es descuido puntual: `llms.txt` se genera del
frontmatter, el frontmatter se genera a mano, y nadie compara lo generado con la
superficie real del código.

Además, un PR que edita `docs/` y deja `llms.txt` viejo publica una superficie
que su propio índice no menciona — que es el segundo modo de fallo de la tabla.

## La regla, y los dos presupuestos

| Situación | Veredicto |
|---|---|
| La superficie no cambió | Pasa |
| La superficie cambió y cambió `docs/` **y** `llms.txt` | Pasa |
| La superficie cambió y cambió `docs/` pero **no** `llms.txt` | Falla — presupuesto `llms-stale` |
| La superficie cambió y **nada** bajo `docs/` cambió | Falla — presupuesto `undocumented` |
| No se pudo evaluar (ref base ausente) | **Exit 3**, falla el job |

Son **dos** presupuestos y no uno porque son dos fallos con dos arreglos
distintos: "cambiaste la API y no escribiste nada" se arregla escribiendo;
"escribiste docs pero no regeneraste `llms.txt`" se arregla corriendo el
generador. Un número para los dos obliga a subir el otro para trabajar alrededor
de este, y así es como un presupuesto deja de ser un presupuesto.

```bash
node scripts/docs/check-api-docs.mjs --changed origin/main..HEAD --max-undocumented=0 --max-llms-stale=0
```

Los dos valen **0** por defecto. Un gate que nace rojo se apaga, así que el
defecto es el número estricto. Si alguna vez hace falta holgura, se sube **uno**
con el flag y se fecha el motivo, igual que hace `check-links.mjs`.

### Por qué `llms.txt` cuenta aparte

`llms.txt` vive en la **raíz** del repo, no bajo `docs/`. Lo genera
`gen-index.mjs` a partir del frontmatter, y es lo que un agente de IA lee en
lugar de leer el árbol de docs. Por eso se cuenta aparte y lleva su propio
presupuesto.

## Qué NO comprueba

- **Firmas.** Un cambio de firma con el mismo nombre es invisible aquí. Ese carril
  lo tiene `tests/api/public_api.rs` (`cargo public-api`, nightly) en el job
  `public-api-snapshot` de `ci-rust.yml`. Los dos carriles no se solapan: éste es
  el barato, aquél es el exacto.
- **Exactitud.** El gate no puede distinguir un buen diff de doc de uno cosmético.
  Solo mide si existe.
- **Calidad de la documentación.** Reescribir una línea para que el diff exista
  technically pasa. La confianza en el diff es del revisor.
- **Nombres inventados.** `llms.txt` afirma `db.search_memory()`. Este gate
  demuestra que la API *cambió sin docs*, no que un doc existente diga la
  verdad. Para eso haría falta extraer identificadores del `.md` y cruzarlos con
  la superficie; es un segundo gate, no éste.

## Cómo se mide la superficie

Sin dependencias, sin build, sin red. Un *fingerprint* normalizado y ordenado de
`lenguaje \t tipo \t nombre` — solo nombres y tipos, sin firmas ni cuerpos.

| Superficie | Método | Qué captura | Qué NO captura |
|---|---|---|---|
| **rust** (1300) | Parse estático de `src/lib.rs`: `pub mod` alcanzable transitivamente, `pub fn/struct/enum/trait/type/const` de primer nivel, métodos de `impl` inherentes, y los `pub use` de la raíz del crate | Los nombres públicos con su tipo | Aridad de genéricos, tipos de firma, `#[cfg]` bajo features no-default, items generados por macro, `include!` |
| **python** (378) | Los *stubs* `.pyi` + las listas `__all__` | `Client`, `AsyncClient`, sus métodos, propiedades, excepciones, reexports | Firmas (van en el stub, no en el fingerprint) |
| **ts** (484) | Parse estático de `vantadb-ts/src/*.ts` | `export` decls, miembros de clase e interfaz, propiedades de object-literal | Genéricos, tipos de parámetro, todo lo tras `export *` de otro módulo |

Nótese `llms.txt` no entra en la superficie: es documentación, no API.

### Los stubs `.pyi` como introspección de Python

La pregunta obvia es por qué no se importa el módulo y se usa `dir()`. La
respuesta: `vantadb_py` es una extensión compilada (`.pyd`); importarla exige
Python, la wheel construida, y la máquina correcta. Los *stubs* son la
superficie **declarada**, y `vantadb-python/tests/test_stub_drift.py` ya afirma
que coinciden con el módulo compilado vía `inspect.signature` (conjunto de
métodos, nombres de parámetro, requiredness). Leer el stub es, por tanto, leer
la superficie — y el eslabón que la une al binario ya está gateado.

### El parse de Rust es aproximado, y está marcado

Un parser de texto no es un compilador. Lo que se pierde: tipos de firma,
aridad de genéricos, items de macros, `include!`, y features no-default. Lo que
gana: corre en **0.3 s** sin *toolchain*, sin `cargo`, sin red — frente a los
minutos que necesita el build de rustdoc JSON. Para una pregunta de *¿cambió el
nombre de algo?* es exacto. Para *¿cambió esta firma?* no sirve, y por eso el
otro job existe.

Se declara explícitamente en el propio archivo (`ponytail:`), no en un README.

## Cómo ejecutarlo en local

```bash
# ¿El gate sigue siendo correcto?
node scripts/docs/check-api-docs.mjs --self-test

# ¿Qué ve ahora mismo?
node scripts/docs/check-api-docs.mjs --api-surface

# ¿Qué veía en otro momento?
node scripts/docs/check-api-docs.mjs --api-surface 8ac202a1

# El gate, con cambios sin commitear
node scripts/docs/check-api-docs.mjs --changed origin/main..WORKTREE

# machine-readable
node scripts/docs/check-api-docs.mjs --changed origin/main..HEAD --json
```

`WORKTREE` como `<head>` compara contra ficheros sin commitear, así que se puede
ver el veredicto **antes** de hacer commit. Tarda ~6 s con dos refs.

## Añadir una API nueva, bien

En el **mismo PR**:

1. Escribe el código.
2. Actualiza la página de referencia de esa superficie
   (`docs/api/RUST_API.md`, `docs/api/PYTHON_SDK.md`, `docs/api/TS_SDK.md`).
3. Regenera los índices:
   ```bash
   node scripts/docs/gen-index.mjs --write
   ```
   Eso reescribe `llms.txt` y los índices de sección, y es lo que hace que el
   job pase.
4. Si es un cambio de firma, actualiza además el snapshot de `cargo public-api`
   (`VANTADB_PUBLIC_API_UPDATE=1 cargo nextest run -p vantadb --test public_api`).
   Ese carril no lo cubre este gate.

Los pasos 2 y 3 no son opcionales. El paso 3 es el que casi todo el mundo se
 salta, y es exactamente el segundo fallo de la tabla.

## Falso positivo

Tres formas, en orden de probabilidad:

**"Cambié algo interno y saltó."** Sólo se miran los items `pub` alcanzables
desde `src/lib.rs`. Si saltó, algo público cambió de verdad. Comprueba con
`--json` y mira el `kind`: si es `mod` o `reexport`, puede ser un
re-ordenamiento. **Fallo real de la herramienta**: los reexports se normalizan a
su último segmento, así que `pub use sdk::GroupByConfig` y
`pub use sdk::{GroupByConfig, ..}` son el mismo nombre; si aparece duplicado, es
un bug y hay que arreglarlo.

**"Es un comentario o una cadena."** El extractor de Rust quita comentarios y
literales antes de buscar `pub`. Si un `pub` dentro de un *doc comment* cuenta,
eso es un bug del extractor.

**"De verdad no es público."** Entonces el código debería decir `pub(crate)`. Si
`pub fn` no es parte del contrato, no es `pub fn`; cambiar el modificador es la
corrección, no subir el presupuesto.

En ningún caso la respuesta es `--max-undocumented=99`. Ese es el movimiento que
convierte un presupuesto en una puerta giratoria. Si hay que subirlo, se sube
**un** presupuesto, con el número y la fecha al lado, y una tarea que lo vuelva a
bajar.

## Estrategia del ref base

`actions/checkout` con `fetch-depth: 0`. No es costumbre, es carga útil.

Un checkout por defecto es un clon de profundidad 1 de un *merge ref* sintético
cuyos padres **no están presentes**. `git diff base..head` falla, y el script
reporta DEGRADED. Con historia completa, `github.event.pull_request.base.sha`
siempre resuelve. La alternativa —un segundo paso `git fetch` del base— deja el
ref base como un paso que puede fallar por red, y un gate cuyo ref base depende
de una llamada de red es un gate que un día no se ejecuta.

El workflow resuelve el base explícitamente **antes** del gate e imprime lo que
eligió. Si `base.sha` no estuviera en el clon, avisa con `::warning::` y cae a
`origin/<base.ref>` en vez de fallar en silencio.

En `push` y `workflow_dispatch` no hay base, así que `BASE` queda vacío a
propósito: el job reporta la superficie y **no simula haber gateado**.

### Exit 3: DEGRADED, y falla

Distinto de exit 1 a propósito. Exit 3 significa "no pude mirar", y un check
verde para "no miré" es exactamente el modo de fallo que la regla 6 de
RULES.md existe para impedir. El workflow trata el 3 como fallo, no como
`continue-on-error`.

## Invariantes

- El script **nunca** muta el árbol de trabajo. Los refs se leen con
  `git cat-file --batch` y `git ls-tree`; no hay `checkout`, ni `stash`, ni
  escritura. Un gate que muta el repo para mirar la historia es un gate que no
  se puede confiar en ejecutar dos veces.
- Los reexports se normalizan a su último segmento (`sdk::Foo` ≡ `Foo`).
- Un *rename* **no** es detectable con un fingerprint de nombre: quitar `foo` y
  añadir `bar` del mismo tipo es byte a byte idéntico a un borrado más un alta
  no relacionada. Por eso el informe dice `rename CANDIDATES`, nunca "renombrado".
- `--self-test` usa sólo fixtures sintéticos: sin ficheros del repo, sin git, sin
  red. Un gate que siempre devuelve "sin cambios" pasa su propio test para
  siempre — por eso los tests cubren también los tres extractores.
- El fingerprint es de **nombres**, no de firmas. Ver "Qué NO comprueba".

## Scripts

| Script | Para qué |
|---|---|
| `scripts/docs/check-api-docs.mjs` | `--api-surface` (extrae) y `--changed` (gatea). Sin dependencias, Node >= 18 |
| `scripts/docs/lib.mjs` | `abs()` reutilizado del resto del toolchain de docs |

Reutiliza `abs()` de `lib.mjs` en vez de recalcular la raíz del repo: una raíz
definida en varios sitios acaba siendo distinta en cada uno, y un gate que lee
del sitio equivocado no falla — informa.
